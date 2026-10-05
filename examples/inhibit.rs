use std::{env, error::Error, os::fd::AsRawFd, process::ExitCode, thread, time::Duration};

use xdg_desktop_portal_logind::LoginManagerProxy;

const HELP: &str = "Logind/elogind inhibitor client example.

Usage: cargo run --example inhibit -- [idle|sleep|sleep:idle|idle:sleep] [SECONDS]

Defaults: idle, 20 seconds. Uses block mode; does not request suspend.
An idle lock suppresses configured swayidle timeouts; a sleep lock blocks
sleep operations that respect logind inhibitors. Exit releases the lock.

Example: cargo run --example inhibit -- idle 45
Inspect: elogind-inhibit --list (or systemd-inhibit --list)";

fn parse_args(args: &[String]) -> Result<Option<(String, Duration)>, String> {
    if args.len() == 1 && matches!(args[0].as_str(), "-h" | "--help") {
        return Ok(None);
    }
    if args.len() > 2 {
        return Err("expected at most an inhibitor type and duration".into());
    }
    let what = args.first().map(String::as_str).unwrap_or("idle");
    if !matches!(what, "idle" | "sleep" | "sleep:idle" | "idle:sleep") {
        return Err(format!("unsupported inhibitor type: {what}"));
    }
    let seconds = match args.get(1) {
        Some(value) => value
            .parse::<u64>()
            .map_err(|_| "SECONDS must be a nonnegative integer".to_owned())?,
        None => 20,
    };
    Ok(Some((what.to_owned(), Duration::from_secs(seconds))))
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some((what, duration)) = parse_args(&args)? else {
        println!("{HELP}");
        return Ok(());
    };

    println!("Acquiring {what} block inhibitor on the system bus...");
    let lock = futures_lite::future::block_on(async {
        let connection = zbus::Connection::system().await?;
        let manager = LoginManagerProxy::new(&connection).await?;
        manager
            .inhibit(
                &what,
                "xdg-desktop-portal-logind-demo",
                "Testing the Rust logind inhibitor client",
                "block",
            )
            .await
    })?;

    // The connection may now go away: the returned owned FD holds the lock.
    println!(
        "Lock active: FD {}, PID {}. Holding for {} seconds.",
        lock.as_raw_fd(),
        std::process::id(),
        duration.as_secs()
    );
    println!("Inspect with elogind-inhibit --list or systemd-inhibit --list.");
    println!("Ctrl+C also releases the lock through kernel FD cleanup.");
    // This standalone demo has no other work while holding the FD. A portal
    // server will instead handle requests concurrently and own an FD per request.
    thread::sleep(duration);
    drop(lock);
    println!("Lock released cleanly.");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn defaults_to_short_idle_test() {
        assert_eq!(
            parse_args(&[]).unwrap(),
            Some(("idle".into(), Duration::from_secs(20)))
        );
    }

    #[test]
    fn accepts_supported_locks() {
        for what in ["idle", "sleep", "sleep:idle", "idle:sleep"] {
            assert_eq!(
                parse_args(&args(&[what, "1"])).unwrap(),
                Some((what.into(), Duration::from_secs(1)))
            );
        }
    }

    #[test]
    fn help_needs_no_bus_connection() {
        assert_eq!(parse_args(&args(&["--help"])).unwrap(), None);
    }

    #[test]
    fn rejects_invalid_arguments() {
        for values in [
            vec!["shutdown"],
            vec!["idle", "-1"],
            vec!["idle", "oops"],
            vec!["idle", "1", "extra"],
        ] {
            assert!(parse_args(&args(&values)).is_err());
        }
    }
}
