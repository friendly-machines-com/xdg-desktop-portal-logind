use std::{env, error::Error, process::ExitCode};

use xdg_desktop_portal_logind::backend;

const HELP: &str = "xdg-desktop-portal-logind: logind/elogind Inhibit portal backend

Usage: xdg-desktop-portal-logind [--help]

Runs on the session bus as org.freedesktop.impl.portal.desktop.logind.
Requires a system bus with systemd-logind or elogind.
Normally started through D-Bus activation by xdg-desktop-portal.

Client example: cargo run --example inhibit -- idle 45";

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!("{HELP}");
        return Ok(());
    }
    if !args.is_empty() {
        return Err("unexpected arguments; use --help for usage".into());
    }
    futures_lite::future::block_on(async {
        let session = zbus::Connection::session().await?;
        let system = zbus::Connection::system().await?;
        backend::serve(session, system).await
    })?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xdg-desktop-portal-logind: {error}");
            ExitCode::FAILURE
        }
    }
}
