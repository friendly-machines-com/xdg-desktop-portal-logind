//! Portal Inhibit backend and typed logind/elogind client.

pub mod backend;

use zbus::zvariant::OwnedFd;

/// An inhibitor's (what, who, why, mode, uid, pid) fields, in wire order.
pub type InhibitorEntry = (String, String, String, String, u32, u32);

/// Client for the shared systemd-logind/elogind D-Bus API.
#[zbus::proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
pub trait LoginManager {
    /// Acquire an inhibitor. Keep the returned FD alive until inhibition ends.
    ///
    /// Closing every copy of the FD releases the lock, independently of the
    /// D-Bus connection's lifetime. Do not convert it to an unowned raw FD.
    fn inhibit(&self, what: &str, who: &str, why: &str, mode: &str) -> zbus::Result<OwnedFd>;

    /// Entries contain (what, who, why, mode, uid, pid).
    fn list_inhibitors(&self) -> zbus::Result<Vec<InhibitorEntry>>;
}
