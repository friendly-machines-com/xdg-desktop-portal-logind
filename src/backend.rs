//! Session-bus portal service. Inhibitor FDs belong to individual requests.

use std::{collections::HashMap, sync::Arc};

use async_lock::Mutex;
use futures_lite::StreamExt;
use zbus::{
    Connection, ObjectServer, fdo,
    message::Header,
    zvariant::{OwnedFd, OwnedObjectPath, OwnedValue},
};

use crate::LoginManagerProxy;

pub const BUS_NAME: &str = "org.freedesktop.impl.portal.desktop.logind";
pub const DESKTOP_PATH: &str = "/org/freedesktop/portal/desktop";
const FRONTEND_NAME: &str = "org.freedesktop.portal.Desktop";
const REQUEST_PREFIX: &str = "/org/freedesktop/portal/desktop/request/";

type Options = HashMap<String, OwnedValue>;

/// Strict mapping: never silently discard unsupported inhibition flags.
fn inhibitor_types(flags: u32) -> fdo::Result<&'static str> {
    match flags {
        4 => Ok("sleep"),
        8 => Ok("idle"),
        12 => Ok("sleep:idle"),
        0 => Err(fdo::Error::InvalidArgs(
            "at least one flag is required".into(),
        )),
        flags if flags & !15 != 0 => {
            Err(fdo::Error::InvalidArgs("unknown inhibition flags".into()))
        }
        _ => Err(fdo::Error::NotSupported(
            "logout and user-switch inhibition are not supported".into(),
        )),
    }
}

fn failed(error: impl std::fmt::Display) -> fdo::Error {
    fdo::Error::Failed(error.to_string())
}

fn sender(header: &Header<'_>) -> fdo::Result<String> {
    header
        .sender()
        .map(|name| name.to_string())
        .ok_or_else(|| fdo::Error::AccessDenied("missing D-Bus sender".into()))
}

async fn authorize(connection: &Connection, owner: &str) -> fdo::Result<()> {
    let bus = fdo::DBusProxy::new(connection).await.map_err(failed)?;
    let name = FRONTEND_NAME.try_into().map_err(failed)?;
    let current = bus
        .get_name_owner(name)
        .await
        .map_err(|_| fdo::Error::AccessDenied("the portal frontend is not running".into()))?;
    if current.as_str() != owner {
        return Err(fdo::Error::AccessDenied(
            "only the portal frontend may call this backend".into(),
        ));
    }
    Ok(())
}

/// A request may be closed while its system-bus call is still pending.
#[derive(Default)]
struct LockState {
    closed: bool,
    fd: Option<OwnedFd>,
}

struct RequestState {
    owner: String,
    lock: Mutex<LockState>,
}

#[derive(Default)]
struct Requests {
    entries: Mutex<HashMap<OwnedObjectPath, Arc<RequestState>>>,
}

impl Requests {
    /// Remove only this particular request, not a newer request reusing its path.
    async fn remove(
        &self,
        server: &ObjectServer,
        path: &OwnedObjectPath,
        request: &Arc<RequestState>,
    ) -> zbus::Result<()> {
        let mut entries = self.entries.lock().await;
        {
            let mut lock = request.lock.lock().await;
            lock.closed = true;
            lock.fd.take();
        }
        if entries
            .get(path)
            .is_some_and(|entry| Arc::ptr_eq(entry, request))
        {
            // Reserve the path until it is unexported, so an old Close cannot
            // accidentally unexport a newly created request at the same path.
            let result = server.remove::<Request, _>(path.clone()).await;
            entries.remove(path);
            result?;
        }
        Ok(())
    }

    async fn remove_owner(&self, server: &ObjectServer, owner: &str) -> zbus::Result<()> {
        let matching: Vec<_> = self
            .entries
            .lock()
            .await
            .iter()
            .filter(|(_, request)| request.owner == owner)
            .map(|(path, request)| (path.clone(), request.clone()))
            .collect();
        for (path, request) in matching {
            self.remove(server, &path, &request).await?;
        }
        Ok(())
    }

    async fn clear(&self, server: &ObjectServer) {
        let matching: Vec<_> = self
            .entries
            .lock()
            .await
            .iter()
            .map(|(path, request)| (path.clone(), request.clone()))
            .collect();
        for (path, request) in matching {
            if let Err(error) = self.remove(server, &path, &request).await {
                eprintln!("Failed to unexport request {path}: {error}");
            }
        }
    }
}

struct Request {
    path: OwnedObjectPath,
    state: Arc<RequestState>,
    requests: Arc<Requests>,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Request")]
impl Request {
    async fn close(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(object_server)] server: &ObjectServer,
    ) -> fdo::Result<()> {
        if sender(&header)? != self.state.owner {
            return Err(fdo::Error::AccessDenied(
                "request belongs to another caller".into(),
            ));
        }
        self.requests
            .remove(server, &self.path, &self.state)
            .await
            .map_err(failed)
    }
}

struct Inhibit {
    system: Connection,
    requests: Arc<Requests>,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Inhibit")]
impl Inhibit {
    // This method intentionally has no output arguments. The frontend emits
    // the application-facing response only after this method finishes.
    #[allow(clippy::too_many_arguments)]
    async fn inhibit(
        &self,
        handle: OwnedObjectPath,
        app_id: &str,
        window: &str,
        flags: u32,
        options: Options,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
        #[zbus(object_server)] server: &ObjectServer,
    ) -> fdo::Result<()> {
        let owner = sender(&header)?;
        authorize(connection, &owner).await?;
        let what = inhibitor_types(flags)?;
        if !handle.as_str().starts_with(REQUEST_PREFIX) {
            return Err(fdo::Error::InvalidArgs("invalid request path".into()));
        }
        let reason = match options.get("reason") {
            Some(value) => <&str>::try_from(value)
                .map_err(|_| fdo::Error::InvalidArgs("reason must be a string".into()))?,
            None => "Application requested inhibition",
        };
        if reason.chars().count() > 256 {
            return Err(fdo::Error::InvalidArgs(
                "reason exceeds 256 characters".into(),
            ));
        }
        // Window identifiers are intentionally unused: no compositor surface
        // or parent window is needed for a logind inhibitor.
        let _ = window;
        let request = Arc::new(RequestState {
            owner: owner.clone(),
            lock: Mutex::new(LockState::default()),
        });
        {
            let mut entries = self.requests.entries.lock().await;
            if entries.contains_key(&handle) {
                return Err(fdo::Error::InvalidArgs(
                    "request handle already exists".into(),
                ));
            }
            let exported = server
                .at(
                    handle.clone(),
                    Request {
                        path: handle.clone(),
                        state: request.clone(),
                        requests: self.requests.clone(),
                    },
                )
                .await
                .map_err(failed)?;
            if !exported {
                return Err(fdo::Error::InvalidArgs(
                    "request object already exists".into(),
                ));
            }
            entries.insert(handle.clone(), request.clone());
        }

        let acquired = async {
            // Catch disconnection/name loss that occurred before registration.
            authorize(connection, &owner).await?;
            let manager = LoginManagerProxy::new(&self.system).await.map_err(failed)?;
            let who = if app_id.is_empty() {
                "Application"
            } else {
                app_id
            };
            let fd = manager
                .inhibit(what, who, reason, "block")
                .await
                .map_err(failed)?;
            // Do not retain a newly acquired lock after frontend name loss.
            authorize(connection, &owner).await?;
            Ok::<_, fdo::Error>(fd)
        }
        .await;

        match acquired {
            Ok(fd) => {
                let mut lock = request.lock.lock().await;
                if !lock.closed {
                    lock.fd = Some(fd);
                }
                // If Close arrived during acquisition, fd drops here instead.
                Ok(())
            }
            Err(error) => {
                if let Err(cleanup) = self.requests.remove(server, &handle, &request).await {
                    eprintln!("Failed to clean up request {handle}: {cleanup}");
                }
                Err(error)
            }
        }
    }

    // Session monitoring is a different capability from inhibition. Report
    // unsupported explicitly.
    async fn create_monitor(
        &self,
        handle: OwnedObjectPath,
        session_handle: OwnedObjectPath,
        app_id: &str,
        window: &str,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) -> fdo::Result<u32> {
        authorize(connection, &sender(&header)?).await?;
        let _ = (handle, session_handle, app_id, window);
        Err(fdo::Error::NotSupported(
            "session monitoring is not supported".into(),
        ))
    }

    async fn query_end_response(
        &self,
        session_handle: OwnedObjectPath,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) -> fdo::Result<()> {
        authorize(connection, &sender(&header)?).await?;
        let _ = session_handle;
        Err(fdo::Error::NotSupported(
            "session monitoring is not supported".into(),
        ))
    }
}

/// Register the service and run until a bus connection or logind is lost.
///
/// Connections are supplied separately to support isolated integration tests.
/// The real daemon uses Connection::session() and Connection::system().
pub async fn serve(session: Connection, system: Connection) -> zbus::Result<()> {
    let requests = Arc::new(Requests::default());
    let session_bus = fdo::DBusProxy::new(&session).await?;
    let system_bus = fdo::DBusProxy::new(&system).await?;
    // Subscribe before exporting anything, so frontend loss cannot be missed.
    let mut session_changes = session_bus.receive_name_owner_changed().await?;
    let mut system_changes = system_bus.receive_name_owner_changed().await?;
    session
        .object_server()
        .at(
            DESKTOP_PATH,
            Inhibit {
                system: system.clone(),
                requests: requests.clone(),
            },
        )
        .await?;
    session.request_name(BUS_NAME).await?;

    let frontend_monitor = async {
        while let Some(change) = session_changes.next().await {
            let args = change.args()?;
            if let Some(old) = args.old_owner().as_ref().filter(|_| {
                args.name().as_str() == FRONTEND_NAME
                    || (args.name().as_str().starts_with(':') && args.new_owner().is_none())
            }) {
                requests
                    .remove_owner(session.object_server(), old.as_str())
                    .await?;
            }
        }
        Err::<(), _>(zbus::Error::Failure("session bus disconnected".into()))
    };
    let logind_monitor = async {
        while let Some(change) = system_changes.next().await {
            let args = change.args()?;
            if args.name().as_str() == "org.freedesktop.login1" && args.old_owner().is_some() {
                // Restarting logind invalidates the old locks. Fail visibly;
                // never keep presenting requests as successfully inhibited.
                return Err(zbus::Error::Failure(
                    "logind disappeared or restarted".into(),
                ));
            }
        }
        Err::<(), _>(zbus::Error::Failure("system bus disconnected".into()))
    };
    let result = futures_lite::future::race(frontend_monitor, logind_monitor).await;
    requests.clear(session.object_server()).await;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_supported_flags() {
        assert_eq!(inhibitor_types(4).unwrap(), "sleep");
        assert_eq!(inhibitor_types(8).unwrap(), "idle");
        assert_eq!(inhibitor_types(12).unwrap(), "sleep:idle");
    }

    #[test]
    fn rejects_empty_unknown_and_unsupported_flags() {
        for flags in [0, 1, 2, 3, 5, 9, 15, 16, u32::MAX] {
            assert!(inhibitor_types(flags).is_err(), "accepted flags {flags}");
        }
    }
}
