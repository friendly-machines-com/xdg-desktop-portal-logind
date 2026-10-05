use std::{
    env,
    ffi::OsString,
    fs::{self, File, OpenOptions, Permissions},
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::{Component, Path, PathBuf},
    process::ExitCode,
    sync::atomic::{AtomicU64, Ordering},
};

const BINARY_NAME: &str = "xdg-desktop-portal-logind";
const SERVICE_NAME: &str = "org.freedesktop.impl.portal.desktop.logind.service";
const PORTAL: &[u8] = include_bytes!("../../data/logind.portal");
const SERVICE: &str =
    include_str!("../../data/org.freedesktop.impl.portal.desktop.logind.service.in");
const HELP: &str = "Upstream installer for xdg-desktop-portal-logind.

Usage: cargo xtask install --binary PATH [OPTIONS]

  --binary PATH       Already-built daemon to install (required)
  --prefix DIR        Final installation prefix (default: /usr/local)
  --libexecdir DIR    Final binary directory (default: PREFIX/libexec)
  --datadir DIR       Final data directory (default: PREFIX/share)
  -h, --help          Show this help

Environment:
  DESTDIR             Staging root; unset or empty means no staging. Never
                      embedded in activation paths.

Directory options must be absolute and may not contain '..'. Both --key VALUE
and --key=VALUE forms are accepted. Relative binary paths use the current
working directory. Existing files are replaced, not followed as symlinks.

Examples:
  cargo xtask install --binary target/release/xdg-desktop-portal-logind --prefix /usr/local
  DESTDIR=/tmp/staging cargo xtask install --binary target/release/xdg-desktop-portal-logind --prefix /usr";

#[derive(Debug)]
struct Options {
    binary: PathBuf,
    libexecdir: PathBuf,
    datadir: PathBuf,
    destdir: Option<PathBuf>,
}

fn absolute_directory(path: PathBuf, option: &str) -> Result<PathBuf, String> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(format!(
            "{option} must be absolute and contain no '..': {}",
            path.display()
        ));
    }
    Ok(path.components().collect())
}

fn set_once(slot: &mut Option<PathBuf>, value: OsString, key: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{key} must not be empty"));
    }
    if slot.replace(PathBuf::from(value)).is_some() {
        return Err(format!("{key} was specified more than once"));
    }
    Ok(())
}

fn parse(
    args: &[OsString],
    environment_destdir: Option<OsString>,
) -> Result<Option<Options>, String> {
    if args == [OsString::from("--help")]
        || args == [OsString::from("-h")]
        || args == [OsString::from("install"), OsString::from("--help")]
        || args == [OsString::from("install"), OsString::from("-h")]
    {
        return Ok(None);
    }
    if args.first().is_none_or(|arg| arg != "install") {
        return Err("expected 'install'; use --help for usage".into());
    }
    let mut binary = None;
    let mut prefix = None;
    let mut libexecdir = None;
    let mut datadir = None;
    let mut args = args[1..].iter();
    while let Some(arg) = args.next() {
        let text = arg.to_str().ok_or("option names must be valid UTF-8")?;
        let (key, inline) = text
            .split_once('=')
            .map_or((text, None), |(key, value)| (key, Some(value)));
        let slot = match key {
            "--binary" => &mut binary,
            "--prefix" => &mut prefix,
            "--libexecdir" => &mut libexecdir,
            "--datadir" => &mut datadir,
            _ => return Err(format!("unknown argument: {text}")),
        };
        let value = match inline {
            Some(value) => OsString::from(value),
            None => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("missing value for {key}"))?;
                if value.to_str().is_some_and(|value| value.starts_with("--")) {
                    return Err(format!("missing value for {key}"));
                }
                value.clone()
            }
        };
        set_once(slot, value, key)?;
    }
    let binary = binary.ok_or("--binary is required; installation never builds the daemon")?;
    let prefix = absolute_directory(prefix.unwrap_or_else(|| "/usr/local".into()), "--prefix")?;
    let libexecdir = absolute_directory(
        libexecdir.unwrap_or_else(|| prefix.join("libexec")),
        "--libexecdir",
    )?;
    let datadir = absolute_directory(datadir.unwrap_or_else(|| prefix.join("share")), "--datadir")?;
    let destdir = environment_destdir
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|path| absolute_directory(path, "DESTDIR"))
        .transpose()?;
    Ok(Some(Options {
        binary,
        libexecdir,
        datadir,
        destdir,
    }))
}

fn staged(destdir: Option<&Path>, final_path: &Path) -> PathBuf {
    match destdir {
        Some(root) => root.join(
            final_path
                .strip_prefix("/")
                .expect("absolute installation path"),
        ),
        None => final_path.to_owned(),
    }
}

/// D-Bus activation parses the key-file value first, then shell-style argv.
/// Quote one literal executable argument; no shell or variable expansion occurs.
fn activation_executable(path: &Path) -> Result<String, String> {
    let text = path
        .to_str()
        .ok_or("activation executable path must be UTF-8")?;
    if text.chars().any(char::is_control) {
        return Err("activation executable path must not contain control characters".into());
    }
    let mut quoted = String::from("\"");
    for character in text.chars() {
        if matches!(character, '\\' | '"' | '$' | '`') {
            quoted.push('\\');
        }
        quoted.push(character);
    }
    quoted.push('"');
    // The service key-file parser needs doubled backslashes to preserve the
    // escapes that the later command-line parser consumes.
    Ok(quoted.replace('\\', "\\\\"))
}

struct Plan {
    executable: PathBuf,
    descriptor: PathBuf,
    service: PathBuf,
    service_contents: String,
}

fn plan(options: &Options) -> Result<Plan, String> {
    let final_executable = options.libexecdir.join(BINARY_NAME);
    let executable_arg = activation_executable(&final_executable)?;
    if SERVICE.matches("@executable@").count() != 1 {
        return Err("activation template must contain exactly one @executable@ placeholder".into());
    }
    Ok(Plan {
        executable: staged(options.destdir.as_deref(), &final_executable),
        descriptor: staged(
            options.destdir.as_deref(),
            &options
                .datadir
                .join("xdg-desktop-portal/portals/logind.portal"),
        ),
        service: staged(
            options.destdir.as_deref(),
            &options.datadir.join("dbus-1/services").join(SERVICE_NAME),
        ),
        service_contents: SERVICE.replace("@executable@", &executable_arg),
    })
}

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TemporaryFile(PathBuf);

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// Replace each file atomically, avoiding symlink following and permitting
/// installation over an executable that is currently running.
fn replace_file(
    path: &Path,
    mode: u32,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("missing destination parent"))?;
    let (temporary, mut file) = loop {
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let name = parent.join(format!(".logind-install-{}-{serial}", std::process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&name) {
            Ok(file) => break (TemporaryFile(name), file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    write(&mut file)?;
    file.set_permissions(Permissions::from_mode(mode))?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary.0, path)?;
    Ok(())
}

fn install(options: &Options) -> Result<(), Box<dyn std::error::Error>> {
    // Validate inputs and render the template before touching destinations.
    let mut binary = File::open(&options.binary)?;
    if !binary.metadata()?.is_file() {
        return Err("--binary must name a regular file".into());
    }
    let plan = plan(options)?;
    let paths = [&plan.executable, &plan.descriptor, &plan.service];
    for path in paths {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_dir() => {
                return Err(format!("destination is a directory: {}", path.display()).into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    for path in paths {
        fs::create_dir_all(path.parent().ok_or("missing destination parent")?)?;
    }
    replace_file(&plan.executable, 0o755, |file| {
        io::copy(&mut binary, file).map(|_| ())
    })?;
    replace_file(&plan.descriptor, 0o644, |file| file.write_all(PORTAL))?;
    replace_file(&plan.service, 0o644, |file| {
        file.write_all(plan.service_contents.as_bytes())
    })?;
    for path in paths {
        println!("Installed {}", path.display());
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match parse(
        &env::args_os().skip(1).collect::<Vec<_>>(),
        env::var_os("DESTDIR"),
    )? {
        Some(options) => install(&options),
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
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    // Standard-library-only temporary directory with automatic cleanup.
    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            loop {
                let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
                let path = env::temp_dir()
                    .join(format!("logind-xtask-test-{}-{serial}", std::process::id()));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("creating temporary directory: {error}"),
                }
            }
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }
    fn fixture(root: &Path, destdir: Option<PathBuf>) -> Options {
        let binary = root.join("input-daemon");
        fs::write(&binary, b"example binary bytes").unwrap();
        Options {
            binary,
            libexecdir: root.join("prefix/libexec"),
            datadir: root.join("prefix/share"),
            destdir,
        }
    }
    fn mode(path: &Path) -> u32 {
        fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn parses_defaults_equals_and_overrides() {
        let defaults = parse(&args(&["install", "--binary", "target/daemon"]), None)
            .unwrap()
            .unwrap();
        assert_eq!(defaults.libexecdir, Path::new("/usr/local/libexec"));
        assert_eq!(defaults.datadir, Path::new("/usr/local/share"));
        let options = parse(
            &args(&[
                "install",
                "--binary=daemon",
                "--prefix=/opt/test",
                "--libexecdir",
                "/other/libexec",
                "--datadir=/other/data",
            ]),
            Some("/stage".into()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(options.libexecdir, Path::new("/other/libexec"));
        assert_eq!(options.datadir, Path::new("/other/data"));
        assert_eq!(options.destdir.as_deref(), Some(Path::new("/stage")));
    }

    #[test]
    fn rejects_bad_arguments_and_relative_directories() {
        for values in [
            vec![],
            vec!["install"],
            vec!["build"],
            vec!["install", "--binary"],
            vec!["install", "--binary=daemon", "--prefix=relative"],
            vec!["install", "--binary=daemon", "--prefix=/opt/../usr"],
            vec!["install", "--binary=daemon", "--destdir=relative"],
            vec!["install", "--binary=daemon", "--prefix="],
            vec!["install", "--binary=daemon", "--unknown"],
            vec!["install", "--binary=daemon", "--binary=other"],
        ] {
            assert!(parse(&args(&values), None).is_err(), "accepted {values:?}");
        }
        assert!(parse(&args(&["--help"]), None).unwrap().is_none());
    }

    #[test]
    fn destdir_environment_unset_empty_and_set() {
        let arguments = args(&["install", "--binary=daemon"]);
        for value in [None, Some(OsString::new())] {
            assert!(parse(&arguments, value).unwrap().unwrap().destdir.is_none());
        }
        assert_eq!(
            parse(&arguments, Some("/staging".into()))
                .unwrap()
                .unwrap()
                .destdir,
            Some(PathBuf::from("/staging"))
        );
        for value in ["relative", "/staging/../other"] {
            assert!(
                parse(&arguments, Some(value.into()))
                    .unwrap_err()
                    .contains("DESTDIR")
            );
        }
    }

    #[test]
    fn rejects_removed_destdir_option() {
        for arguments in [
            args(&["install", "--binary=daemon", "--destdir=/explicit"]),
            args(&["install", "--binary=daemon", "--destdir", "/explicit"]),
        ] {
            let error = parse(&arguments, Some("/environment".into())).unwrap_err();
            assert!(error.contains("unknown argument: --destdir"));
        }
    }

    #[test]
    fn installs_exactly_three_files_with_correct_modes_and_paths() {
        let temp = TestDirectory::new();
        let options = fixture(&temp.0, None);
        install(&options).unwrap();
        let plan = plan(&options).unwrap();
        assert_eq!(fs::read(&plan.executable).unwrap(), b"example binary bytes");
        assert_eq!(fs::read(&plan.descriptor).unwrap(), PORTAL);
        assert_eq!(
            fs::read_to_string(&plan.service).unwrap(),
            plan.service_contents
        );
        assert_eq!(mode(&plan.executable), 0o755);
        assert_eq!(mode(&plan.descriptor), 0o644);
        assert_eq!(mode(&plan.service), 0o644);
        assert_eq!(fs::read_dir(options.libexecdir).unwrap().count(), 1);
        assert_eq!(
            fs::read_dir(options.datadir.join("xdg-desktop-portal/portals"))
                .unwrap()
                .count(),
            1
        );
        assert_eq!(
            fs::read_dir(options.datadir.join("dbus-1/services"))
                .unwrap()
                .count(),
            1
        );
        assert!(!plan.service_contents.contains('@'));
    }

    #[test]
    fn staging_never_appears_in_activation_path() {
        let temp = TestDirectory::new();
        let mut options = fixture(&temp.0, Some(temp.0.join("staging")));
        options.libexecdir = "/usr/libexec".into();
        options.datadir = "/usr/share".into();
        install(&options).unwrap();
        let plan = plan(&options).unwrap();
        assert_eq!(
            plan.executable,
            temp.0.join("staging/usr/libexec").join(BINARY_NAME)
        );
        assert!(
            plan.service_contents
                .contains("Exec=\"/usr/libexec/xdg-desktop-portal-logind\"")
        );
        assert!(!plan.service_contents.contains(temp.0.to_str().unwrap()));
    }

    #[test]
    fn supports_space_and_escaped_activation_paths() {
        assert_eq!(
            activation_executable(Path::new("/opt/a b/daemon")).unwrap(),
            "\"/opt/a b/daemon\""
        );
        assert_eq!(
            activation_executable(Path::new("/opt/a\"b\\c$d`e/daemon")).unwrap(),
            "\"/opt/a\\\\\"b\\\\\\\\c\\\\$d\\\\`e/daemon\""
        );
        assert!(activation_executable(Path::new("/opt/a\nb/daemon")).is_err());
    }

    #[test]
    fn replacement_does_not_follow_destination_symlink() {
        let temp = TestDirectory::new();
        let options = fixture(&temp.0, None);
        install(&options).unwrap();
        let plan = plan(&options).unwrap();
        let victim = temp.0.join("must-not-change");
        fs::write(&victim, b"unchanged").unwrap();
        fs::remove_file(&plan.executable).unwrap();
        symlink(&victim, &plan.executable).unwrap();
        install(&options).unwrap();
        assert_eq!(fs::read(&victim).unwrap(), b"unchanged");
        assert!(
            !fs::symlink_metadata(&plan.executable)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn failed_preflight_does_not_create_destinations() {
        let temp = TestDirectory::new();
        let mut options = fixture(&temp.0, None);
        options.binary = temp.0.join("missing");
        assert!(install(&options).is_err());
        assert!(!temp.0.join("prefix").exists());
        options.binary = temp.0.join("input-daemon");
        options.libexecdir = temp.0.join("bad\npath");
        assert!(install(&options).is_err());
        assert!(!temp.0.join("prefix").exists());
    }
}
