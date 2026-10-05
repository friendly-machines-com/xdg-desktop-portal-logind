//! Explicitly opt-in: this test briefly inhibits real system idleness.

use std::{
    thread,
    time::{Duration, Instant},
};
use xdg_desktop_portal_logind::LoginManagerProxy;

#[test]
#[ignore = "requires a live system bus, logind/elogind, and inhibitor authorization"]
fn acquires_and_releases_real_idle_inhibitor() {
    futures_lite::future::block_on(async {
        let connection = zbus::Connection::system().await.unwrap();
        let manager = LoginManagerProxy::new(&connection).await.unwrap();
        let who = format!("xdg-desktop-portal-logind-test-{}", std::process::id());
        let lock = manager
            .inhibit("idle", &who, "Testing owned FD lifecycle", "block")
            .await
            .unwrap();

        let entries = manager.list_inhibitors().await.unwrap();
        assert!(entries.iter().any(|entry| {
            entry.0 == "idle"
                && entry.1 == who
                && entry.3 == "block"
                && entry.5 == std::process::id()
        }));

        drop(lock);
        // logind processes the pipe close asynchronously; allow a short grace
        // period.
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let entries = manager.list_inhibitors().await.unwrap();
            if !entries.iter().any(|entry| entry.1 == who) {
                break;
            }
            assert!(Instant::now() < deadline, "inhibitor survived FD closure");
            thread::sleep(Duration::from_millis(10));
        }
    });
}
