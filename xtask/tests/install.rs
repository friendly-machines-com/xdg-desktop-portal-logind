//! Exercise the installer's actual process environment.

use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        loop {
            let path = env::temp_dir().join(format!(
                "logind-destdir-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("creating test directory: {error}"),
            }
        }
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn installer(binary: &Path, prefix: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
    command
        .arg("install")
        .arg("--binary")
        .arg(binary)
        .arg("--prefix")
        .arg(prefix);
    command
}

#[test]
fn destdir_environment_stages_all_files_without_touching_final_prefix() {
    let directory = Directory::new();
    let binary = directory.0.join("input-daemon");
    fs::write(&binary, b"test daemon bytes").unwrap();
    let prefix = directory.0.join("final-prefix");
    let staging = directory.0.join("staging root");
    let output = installer(&binary, &prefix)
        .env("DESTDIR", &staging)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !prefix.exists(),
        "installer ignored DESTDIR and wrote into the final prefix"
    );

    let installed = staging.join(prefix.strip_prefix("/").unwrap());
    let daemon = installed.join("libexec/xdg-desktop-portal-logind");
    assert_eq!(fs::read(&daemon).unwrap(), b"test daemon bytes");
    assert_eq!(
        fs::metadata(daemon).unwrap().permissions().mode() & 0o777,
        0o755
    );
    assert!(
        installed
            .join("share/xdg-desktop-portal/portals/logind.portal")
            .is_file()
    );
    let service = fs::read_to_string(
        installed.join("share/dbus-1/services/org.freedesktop.impl.portal.desktop.logind.service"),
    )
    .unwrap();
    assert!(service.contains(&format!(
        "Exec=\"{}/libexec/xdg-desktop-portal-logind\"",
        prefix.display()
    )));
    assert!(!service.contains(staging.to_str().unwrap()));
}

#[test]
fn unset_and_empty_destdir_install_directly() {
    let directory = Directory::new();
    let binary = directory.0.join("input-daemon");
    fs::write(&binary, b"test daemon bytes").unwrap();
    for (name, value) in [("unset", None), ("empty", Some(""))] {
        let prefix = directory.0.join(name);
        let mut command = installer(&binary, &prefix);
        command.env_remove("DESTDIR");
        if let Some(value) = value {
            command.env("DESTDIR", value);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(prefix.join("libexec/xdg-desktop-portal-logind").is_file());
    }
}

#[test]
fn invalid_destdir_fails_before_installation() {
    let directory = Directory::new();
    let binary = directory.0.join("input-daemon");
    fs::write(&binary, b"test daemon bytes").unwrap();
    let prefix = directory.0.join("final-prefix");
    let output = installer(&binary, &prefix)
        .env("DESTDIR", "relative")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("DESTDIR"));
    assert!(!prefix.exists());
}
