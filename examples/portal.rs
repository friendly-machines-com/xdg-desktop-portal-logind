//! Application-side test of the public portal API, including Response and Close.

use std::{collections::HashMap, env, error::Error, process::ExitCode, time::Duration};

use futures_lite::StreamExt;
use zbus::{
    Connection, Proxy, fdo,
    zvariant::{OwnedObjectPath, OwnedValue, Str},
};

const FRONTEND: &str = "org.freedesktop.portal.Desktop";
const DESKTOP: &str = "/org/freedesktop/portal/desktop";
const HELP: &str = "Public portal Inhibit client example.

Usage: cargo run --example portal -- [idle|sleep|sleep:idle] [SECONDS]

Defaults: idle for 45 seconds. Waits for a successful Request.Response,
then holds the request and closes it. Does not request suspend.
Run only after selecting the logind backend, or via scripts/test-portal.sh.";

type Results = HashMap<String, OwnedValue>;

fn parse_args(args: &[String]) -> Result<Option<(u32, Duration)>, String> {
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        return Ok(None);
    }
    if args.len() > 2 {
        return Err("expected at most an inhibitor type and duration".into());
    }
    let flags = match args.first().map(String::as_str).unwrap_or("idle") {
        "idle" => 8,
        "sleep" => 4,
        "sleep:idle" | "idle:sleep" => 12,
        other => return Err(format!("unsupported inhibitor type: {other}")),
    };
    let seconds = match args.get(1) {
        Some(value) => value
            .parse::<u64>()
            .map_err(|_| "invalid SECONDS".to_owned())?,
        None => 45,
    };
    Ok(Some((flags, Duration::from_secs(seconds))))
}

async fn test_portal(flags: u32, duration: Duration) -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;
    let bus = fdo::DBusProxy::new(&connection).await?;
    // The runner starts the frontend manually; do not accidentally activate a
    // different frontend while it is starting up.
    let mut ready = false;
    for _ in 0..200 {
        if bus.name_has_owner(FRONTEND.try_into()?).await? {
            ready = true;
            break;
        }
        async_io::Timer::after(Duration::from_millis(50)).await;
    }
    if !ready {
        return Err("portal frontend did not acquire its bus name within 10 seconds".into());
    }

    // Predict the request path and subscribe BEFORE invoking Inhibit, since
    // the frontend may emit Response before its method reply is received.
    let unique = connection.unique_name().ok_or("missing unique bus name")?;
    let sender = unique.as_str().trim_start_matches(':').replace('.', "_");
    let token = format!("logind_test_{}", std::process::id());
    let expected = OwnedObjectPath::try_from(format!("{DESKTOP}/request/{sender}/{token}"))?;
    let request = Proxy::new(
        &connection,
        FRONTEND,
        expected.clone(),
        "org.freedesktop.portal.Request",
    )
    .await?;
    let mut responses = request.receive_signal("Response").await?;
    let options = HashMap::from([
        ("handle_token", OwnedValue::from(Str::from(token.as_str()))),
        (
            "reason",
            OwnedValue::from(Str::from(
                "Testing xdg-desktop-portal-logind through the portal frontend",
            )),
        ),
    ]);
    let reply = connection
        .call_method(
            Some(FRONTEND),
            DESKTOP,
            Some("org.freedesktop.portal.Inhibit"),
            "Inhibit",
            &("", flags, options),
        )
        .await?;
    let handle: OwnedObjectPath = reply.body().deserialize()?;
    if handle != expected {
        let actual = Proxy::new(
            &connection,
            FRONTEND,
            handle,
            "org.freedesktop.portal.Request",
        )
        .await?;
        let _: () = actual.call("Close", &()).await?;
        return Err("frontend did not honor handle_token; cannot reliably observe Response".into());
    }

    let response = futures_lite::future::race(
        async {
            let signal = responses.next().await.ok_or("Response stream ended")?;
            let (code, _results): (u32, Results) = signal.body().deserialize()?;
            Ok::<_, Box<dyn Error>>(code)
        },
        async {
            async_io::Timer::after(Duration::from_secs(30)).await;
            Err("timed out waiting for portal Response".into())
        },
    )
    .await;
    let code = match response {
        Ok(code) => code,
        Err(error) => {
            let _ = request.call::<_, _, ()>("Close", &()).await;
            return Err(error);
        }
    };
    if code != 0 {
        let _ = request.call::<_, _, ()>("Close", &()).await;
        return Err(
            format!("portal rejected inhibition: Response {code} (0 means success)").into(),
        );
    }
    println!("Portal Response=0. Request: {expected}");
    println!(
        "Holding flags={flags} for {} seconds. Inspect with elogind-inhibit --list.",
        duration.as_secs()
    );
    async_io::Timer::after(duration).await;
    let _: () = request.call("Close", &()).await?;
    println!("Request.Close acknowledged; inhibitor released.");
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    match parse_args(&args)? {
        Some((flags, duration)) => futures_lite::future::block_on(test_portal(flags, duration)),
        None => {
            println!("{HELP}");
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("portal test: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_supported_flags() {
        assert_eq!(parse_args(&[]).unwrap(), Some((8, Duration::from_secs(45))));
        for (what, flags) in [("idle", 8), ("sleep", 4), ("sleep:idle", 12)] {
            assert_eq!(
                parse_args(&[what.into(), "1".into()]).unwrap(),
                Some((flags, Duration::from_secs(1)))
            );
        }
    }

    #[test]
    fn help_and_invalid_arguments() {
        assert_eq!(parse_args(&["--help".into()]).unwrap(), None);
        assert!(parse_args(&["shutdown".into()]).is_err());
        assert!(parse_args(&["idle".into(), "-1".into()]).is_err());
    }
}
