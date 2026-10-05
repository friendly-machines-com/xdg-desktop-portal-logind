//! Kill only the application client, and verify release without service shutdown.

use std::{
    env,
    error::Error,
    io::{BufRead, BufReader},
    process::{Child, Command, ExitCode, Stdio},
    thread,
    time::Duration,
};

use xdg_desktop_portal_logind::{LoginManagerProxy, backend::BUS_NAME};
use zbus::{Connection, fdo};

const FRONTEND: &str = "org.freedesktop.portal.Desktop";

struct Client(Child);

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn owners(bus: &fdo::DBusProxy<'_>) -> Result<(String, String), Box<dyn Error>> {
    Ok((
        bus.get_name_owner(FRONTEND.try_into()?).await?.to_string(),
        bus.get_name_owner(BUS_NAME.try_into()?).await?.to_string(),
    ))
}

async fn verify(what: &str) -> Result<(), Box<dyn Error>> {
    let session = Connection::session().await?;
    let system = Connection::system().await?;
    let manager = LoginManagerProxy::new(&system).await?;
    let bus = fdo::DBusProxy::new(&session).await?;
    // The public client waits for the frontend to become ready and then holds
    // its request. Spawn the sibling example executable, not cargo itself.
    let executable = env::current_exe()?.with_file_name("portal");
    let mut client = Client(
        Command::new(executable)
            .args([what, "300"])
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?,
    );
    let stdout = client.0.stdout.take().ok_or("missing client stdout")?;
    let (ready_tx, ready_rx) = async_channel::bounded(1);
    let output = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let line = line?;
            println!("{line}");
            if line.starts_with("Holding flags=") {
                let _ = ready_tx.send_blocking(());
            }
        }
        Ok::<_, std::io::Error>(())
    });
    futures_lite::future::race(
        async {
            ready_rx
                .recv()
                .await
                .map_err(|_| "client exited before acquiring an inhibitor")
        },
        async {
            async_io::Timer::after(Duration::from_secs(45)).await;
            Err("timed out waiting for client readiness")
        },
    )
    .await?;

    let before = owners(&bus).await?;
    let backend_pid = bus
        .get_connection_unix_process_id(before.1.as_str().try_into()?)
        .await?;
    let locks = manager.list_inhibitors().await?;
    let active: Vec<_> = locks
        .iter()
        .filter(|entry| entry.5 == backend_pid)
        .collect();
    if active.len() != 1 || active[0].3 != "block" || active[0].0 != what {
        return Err(format!(
            "expected one {what}/block inhibitor from backend PID {backend_pid}, got {active:?}"
        )
        .into());
    }
    if client.0.try_wait()?.is_some() {
        return Err("client exited before it could be killed".into());
    }
    println!(
        "Verified active inhibitor. Frontend={}, backend={} (PID {backend_pid}).",
        before.0, before.1
    );
    println!("Sending SIGKILL to client PID {} only.", client.0.id());
    client.0.kill()?; // std::process::Child::kill sends SIGKILL on Unix.
    client.0.wait()?;
    output
        .join()
        .map_err(|_| "client output thread panicked")??;

    for _ in 0..500 {
        if owners(&bus).await? != before {
            return Err("frontend or backend exited or changed owner during cleanup".into());
        }
        let locks = manager.list_inhibitors().await?;
        if !locks.iter().any(|entry| entry.5 == backend_pid) {
            // Check again after observing release, so service shutdown cannot
            // masquerade as successful client-disconnect cleanup.
            if owners(&bus).await? != before {
                return Err("frontend or backend changed owner after release".into());
            }
            println!(
                "PASS: client death released inhibitor; frontend and backend owners unchanged."
            );
            return Ok(());
        }
        async_io::Timer::after(Duration::from_millis(10)).await;
    }
    Err("inhibitor survived client death for more than 5 seconds".into())
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() > 1 {
        return Err("usage: client_death [idle|sleep|sleep:idle]".into());
    }
    let what = args.first().map(String::as_str).unwrap_or("idle");
    if !matches!(what, "idle" | "sleep" | "sleep:idle") {
        return Err("usage: client_death [idle|sleep|sleep:idle]".into());
    }
    futures_lite::future::block_on(verify(what))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("client-death test: {error}");
            ExitCode::FAILURE
        }
    }
}
