//! Tests use a private dbus-daemon and mock logind; never the desktop/system bus.

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    os::{fd::OwnedFd as StdOwnedFd, unix::net::UnixStream},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};

use xdg_desktop_portal_logind::backend::{self, BUS_NAME, DESKTOP_PATH};
use zbus::{
    Connection, fdo,
    zvariant::{OwnedFd, OwnedObjectPath, OwnedValue},
};

struct PrivateBus {
    child: Child,
    address: String,
    _runtime: tempfile::TempDir,
}

impl PrivateBus {
    fn start() -> Self {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::create_dir(runtime.path().join("cache")).unwrap();
        let mut child = Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--print-address=1"])
            .env("XDG_RUNTIME_DIR", runtime.path())
            .env("XDG_CACHE_HOME", runtime.path().join("cache"))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .expect("install dbus-daemon to run the isolated portal tests");
        let mut address = String::new();
        BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut address)
            .unwrap();
        assert!(!address.is_empty());
        Self {
            child,
            address: address.trim().to_owned(),
            _runtime: runtime,
        }
    }

    async fn connect(&self) -> Connection {
        zbus::connection::Builder::address(self.address.as_str())
            .unwrap()
            .build()
            .await
            .unwrap()
    }
}

impl Drop for PrivateBus {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Call {
    what: String,
    who: String,
    why: String,
    mode: String,
    pid: u32,
    peer: UnixStream,
}

type Gate = (async_channel::Sender<()>, async_channel::Receiver<()>);

#[derive(Default)]
struct MockState {
    calls: Mutex<Vec<Call>>,
    gate: Mutex<Option<Gate>>,
}

struct MockLogind(Arc<MockState>);

#[zbus::interface(name = "org.freedesktop.login1.Manager")]
impl MockLogind {
    async fn inhibit(
        &self,
        what: &str,
        who: &str,
        why: &str,
        mode: &str,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) -> fdo::Result<OwnedFd> {
        if why == "deny" {
            return Err(fdo::Error::AccessDenied("test denial".into()));
        }
        let gate = self.0.gate.lock().unwrap().clone();
        if let Some((entered, release)) = gate {
            entered.send(()).await.unwrap();
            release.recv().await.unwrap();
        }
        let bus = fdo::DBusProxy::new(connection).await.unwrap();
        let pid = bus
            .get_connection_unix_process_id(header.sender().unwrap().clone().into())
            .await
            .unwrap();
        let (fd, peer) = UnixStream::pair().unwrap();
        peer.set_nonblocking(true).unwrap();
        self.0.calls.lock().unwrap().push(Call {
            what: what.into(),
            who: who.into(),
            why: why.into(),
            mode: mode.into(),
            pid,
            peer,
        });
        Ok(StdOwnedFd::from(fd).into())
    }

    fn list_inhibitors(&self) -> Vec<xdg_desktop_portal_logind::InhibitorEntry> {
        self.0
            .calls
            .lock()
            .unwrap()
            .iter()
            .filter(|call| !peer_closed(&call.peer))
            .map(|call| {
                (
                    call.what.clone(),
                    call.who.clone(),
                    call.why.clone(),
                    call.mode.clone(),
                    1000,
                    call.pid,
                )
            })
            .collect()
    }
}

fn released(state: &MockState, index: usize) -> bool {
    let calls = state.calls.lock().unwrap();
    peer_closed(&calls[index].peer)
}

fn peer_closed(mut peer: &UnixStream) -> bool {
    let mut byte = [0];
    match peer.read(&mut byte) {
        Ok(0) => true,
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => false,
        result => panic!("unexpected inhibitor peer read: {result:?}"),
    }
}

async fn wait_until(mut predicate: impl FnMut() -> bool) {
    for _ in 0..200 {
        if predicate() {
            return;
        }
        async_io::Timer::after(Duration::from_millis(10)).await;
    }
    panic!("condition did not become true");
}

async fn inhibit(client: &Connection, path: &str, flags: u32, reason: &str) -> zbus::Result<()> {
    let options = HashMap::from([(
        "reason",
        OwnedValue::from(zbus::zvariant::Str::from(reason)),
    )]);
    client
        .call_method(
            Some(BUS_NAME),
            DESKTOP_PATH,
            Some("org.freedesktop.impl.portal.Inhibit"),
            "Inhibit",
            &(
                OwnedObjectPath::try_from(path).unwrap(),
                "test.application",
                "",
                flags,
                options,
            ),
        )
        .await?;
    Ok(())
}

async fn close(client: &Connection, path: &str) -> zbus::Result<()> {
    client
        .call_method(
            Some(BUS_NAME),
            path,
            Some("org.freedesktop.impl.portal.Request"),
            "Close",
            &(),
        )
        .await?;
    Ok(())
}

async fn exercise_service(bus: &PrivateBus) {
    let state = Arc::new(MockState::default());
    let mock = bus.connect().await;
    mock.object_server()
        .at("/org/freedesktop/login1", MockLogind(state.clone()))
        .await
        .unwrap();
    mock.request_name("org.freedesktop.login1").await.unwrap();
    let backend_session = bus.connect().await;
    let backend_system = bus.connect().await;
    let server_session = backend_session.clone();
    let server_task = backend_session.executor().spawn(
        async move { backend::serve(server_session, backend_system).await },
        "portal test server",
    );
    let frontend = bus.connect().await;
    frontend
        .request_name("org.freedesktop.portal.Desktop")
        .await
        .unwrap();
    let observer = fdo::DBusProxy::new(&frontend).await.unwrap();
    for _ in 0..200 {
        if observer
            .name_has_owner(BUS_NAME.try_into().unwrap())
            .await
            .unwrap()
        {
            break;
        }
        async_io::Timer::after(Duration::from_millis(10)).await;
    }
    assert!(
        observer
            .name_has_owner(BUS_NAME.try_into().unwrap())
            .await
            .unwrap()
    );

    let path = "/org/freedesktop/portal/desktop/request/test/one";
    let second = "/org/freedesktop/portal/desktop/request/test/two";
    let stranger = bus.connect().await;
    assert!(inhibit(&stranger, path, 8, "idle").await.is_err());
    assert!(inhibit(&frontend, path, 1, "logout").await.is_err());
    assert!(inhibit(&frontend, path, 0, "empty").await.is_err());
    assert!(inhibit(&frontend, path, 16, "unknown").await.is_err());
    assert!(
        inhibit(&frontend, "/bad/path", 8, "bad path")
            .await
            .is_err()
    );
    assert!(inhibit(&frontend, path, 8, &"x".repeat(257)).await.is_err());
    assert!(state.calls.lock().unwrap().is_empty());

    // Logind failure must be returned, and the failed request must be removed.
    assert!(inhibit(&frontend, path, 8, "deny").await.is_err());
    inhibit(&frontend, path, 8, "idle").await.unwrap();
    assert!(!released(&state, 0));
    assert!(inhibit(&frontend, path, 8, "duplicate").await.is_err());
    assert!(close(&stranger, path).await.is_err());
    assert!(!released(&state, 0));
    inhibit(&frontend, second, 12, "combined").await.unwrap();
    {
        let calls = state.calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(
            (
                &*calls[0].what,
                &*calls[0].who,
                &*calls[0].why,
                &*calls[0].mode
            ),
            ("idle", "test.application", "idle", "block")
        );
        assert_eq!(calls[1].what, "sleep:idle");
    }
    close(&frontend, path).await.unwrap();
    wait_until(|| released(&state, 0)).await;
    assert!(!released(&state, 1));
    close(&frontend, second).await.unwrap();
    wait_until(|| released(&state, 1)).await;

    // Close while system-bus acquisition is pending: no late FD may survive.
    let (entered_tx, entered_rx) = async_channel::bounded(1);
    let (release_tx, release_rx) = async_channel::bounded(1);
    *state.gate.lock().unwrap() = Some((entered_tx, release_rx));
    let caller = frontend.clone();
    let pending = frontend.executor().spawn(
        async move { inhibit(&caller, path, 4, "pending").await },
        "pending inhibition",
    );
    entered_rx.recv().await.unwrap();
    close(&frontend, path).await.unwrap();
    release_tx.send(()).await.unwrap();
    pending.await.unwrap().unwrap();
    wait_until(|| released(&state, 2)).await;
    *state.gate.lock().unwrap() = None;
    assert_eq!(state.calls.lock().unwrap()[2].what, "sleep");

    // Losing the frontend name releases its locks even if the connection lives.
    inhibit(&frontend, path, 8, "name loss").await.unwrap();
    frontend
        .release_name("org.freedesktop.portal.Desktop")
        .await
        .unwrap();
    wait_until(|| released(&state, 3)).await;
    assert!(inhibit(&frontend, second, 8, "no owner").await.is_err());
    frontend
        .request_name("org.freedesktop.portal.Desktop")
        .await
        .unwrap();

    // A new frontend connection disappearing also releases its requests.
    drop(observer);
    inhibit(&frontend, path, 8, "disconnect").await.unwrap();
    frontend.close().await.unwrap();
    wait_until(|| released(&state, 4)).await;

    // logind loss terminates the backend instead of retaining invalid locks.
    mock.release_name("org.freedesktop.login1").await.unwrap();
    assert!(server_task.await.unwrap().is_err());
    backend_session.close().await.unwrap();
}

struct FrontendProcess(Child);

impl Drop for FrontendProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
#[ignore = "requires dbus-daemon and real xdg-desktop-portal; still uses mock logind"]
fn real_frontend_on_private_bus() {
    exercise_runner(&["idle", "0"], "Request.Close acknowledged");
}

#[test]
#[ignore = "requires dbus-daemon and real xdg-desktop-portal; still uses mock logind"]
fn client_death_keeps_frontend_and_backend_alive() {
    exercise_runner(
        &["--client-death", "idle"],
        "PASS: client death released inhibitor; frontend and backend owners unchanged.",
    );
}

fn exercise_runner(arguments: &[&str], expected_output: &str) {
    let bus = PrivateBus::start();
    // Exercise actual D-Bus activation of the installed executable, including
    // both key-file and argv escaping.
    let install_parent = tempfile::Builder::new()
        .prefix("portal install \"quote\\dollar$` ")
        .tempdir()
        .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    futures_lite::future::block_on(async {
        let state = Arc::new(MockState::default());
        let mock = bus.connect().await;
        mock.object_server()
            .at("/org/freedesktop/login1", MockLogind(state.clone()))
            .await
            .unwrap();
        mock.request_name("org.freedesktop.login1").await.unwrap();
        // Exercise the same runner used on the desktop: real frontend, real
        // daemon executable, descriptor selection and D-Bus activation. Only
        // the system bus is replaced with our private mock logind bus.
        let mut client = FrontendProcess(
            Command::new("sh")
                .current_dir(root)
                .arg("scripts/test-portal.sh")
                .args(arguments)
                .env("DBUS_SYSTEM_BUS_ADDRESS", &bus.address)
                .env("TMPDIR", install_parent.path())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        let mut status = None;
        for _ in 0..600 {
            status = client.0.try_wait().unwrap();
            if status.is_some() {
                break;
            }
            async_io::Timer::after(Duration::from_millis(100)).await;
        }
        let mut stdout = String::new();
        let mut stderr = String::new();
        if status.is_some() {
            client
                .0
                .stdout
                .take()
                .unwrap()
                .read_to_string(&mut stdout)
                .unwrap();
            client
                .0
                .stderr
                .take()
                .unwrap()
                .read_to_string(&mut stderr)
                .unwrap();
        }
        assert!(
            status.is_some_and(|status| status.success()),
            "runner failed: {status:?}\n{stdout}\n{stderr}"
        );
        assert!(stdout.contains("Portal Response=0"));
        assert!(
            stdout.contains(expected_output),
            "unexpected output: {stdout}"
        );
        assert_eq!(state.calls.lock().unwrap().len(), 1);
        wait_until(|| released(&state, 0)).await;
    });
}

#[test]
#[ignore = "requires dbus-daemon; uses only a private bus and mock logind"]
fn portal_lifecycle_on_private_bus() {
    let bus = PrivateBus::start();
    futures_lite::future::block_on(futures_lite::future::race(exercise_service(&bus), async {
        async_io::Timer::after(Duration::from_secs(15)).await;
        panic!("portal lifecycle test timed out");
    }));
}
