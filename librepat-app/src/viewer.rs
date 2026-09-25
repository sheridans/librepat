use std::{
    io,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

pub(crate) fn open(path: &Path) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    return launch("cmd", &["/C", "start", ""], path);
    #[cfg(target_os = "macos")]
    return launch("open", &[], path);
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return open_unix(path);
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn open_unix(path: &Path) -> io::Result<()> {
    let launchers: [(&str, &[&str]); 6] = [
        ("gio", &["open"]),
        ("okular", &[]),
        ("evince", &[]),
        ("zathura", &[]),
        ("mupdf", &[]),
        ("xdg-open", &[]),
    ];
    let mut failures = Vec::new();
    for (program, arguments) in launchers {
        match launch(program, arguments, path) {
            Ok(()) => return Ok(()),
            Err(error) => failures.push(format!("{program}: {error}")),
        }
    }
    Err(io::Error::other(format!(
        "no PDF viewer could be started ({})",
        failures.join("; ")
    )))
}

fn launch(program: &str, arguments: &[&str], path: &Path) -> io::Result<()> {
    let mut child = Command::new(program)
        .args(arguments)
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    for _attempt in 0..5 {
        thread::sleep(Duration::from_millis(20));
        if let Some(status) = child.try_wait()? {
            return if status.success() {
                Ok(())
            } else {
                Err(io::Error::other(format!("exited with {status}")))
            };
        }
    }
    Ok(())
}
