use std::process::Command;
use std::time::Instant;

fn quiet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_quiet"))
}

#[test]
fn success_suppresses_output() {
    let out = quiet()
        .args(["sh", "-c", "echo hello; echo world >&2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());
}

#[test]
fn failure_shows_stdout() {
    let out = quiet()
        .args(["sh", "-c", "echo visible; exit 1"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "visible\n");
}

#[test]
fn failure_shows_stderr() {
    let out = quiet()
        .args(["sh", "-c", "echo oops >&2; exit 3"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    assert_eq!(String::from_utf8_lossy(&out.stderr), "oops\n");
}

#[test]
fn preserves_exit_code() {
    let out = quiet().args(["sh", "-c", "exit 42"]).output().unwrap();
    assert_eq!(out.status.code(), Some(42));
}

#[test]
fn missing_command_exits_127() {
    let out = quiet()
        .args(["nonexistent_command_xyz_123"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(127));
}

#[test]
fn no_args_shows_usage() {
    let out = quiet().output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage:"));
}

#[test]
fn double_dash_separator() {
    let out = quiet()
        .args(["--", "sh", "-c", "echo hidden"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(out.stdout.is_empty());
}

#[test]
fn timeout_shows_output_for_slow_command() {
    use std::io::Read;
    use std::process::Stdio;

    let start = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_quiet"))
        .args(["-t", "0.3", "sh", "-c", "echo early; sleep 2; echo late"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let mut stdout = child.stdout.take().unwrap();
    let mut buf = [0u8; 1024];
    let n = stdout.read(&mut buf).unwrap();
    let first_output_at = start.elapsed().as_secs_f64();
    assert!(n > 0);
    assert!(
        first_output_at >= 0.25 && first_output_at < 1.0,
        "first output arrived at {first_output_at:.2}s, expected ~0.3s"
    );

    let mut rest = String::from_utf8_lossy(&buf[..n]).to_string();
    let n = stdout.read_to_string(&mut rest).unwrap_or(0);
    let _ = n;
    let status = child.wait().unwrap();
    assert!(status.success());
    assert!(rest.contains("late"));
}

#[test]
fn timeout_suppresses_fast_success() {
    let out = quiet()
        .args(["-t", "5", "sh", "-c", "echo fast"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(out.stdout.is_empty());
}

#[test]
fn timeout_still_shows_on_failure() {
    let out = quiet()
        .args(["-t", "5", "sh", "-c", "echo fail; exit 1"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "fail\n");
}

#[test]
fn timeout_missing_value() {
    let out = quiet().args(["-t"]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("-t requires a numeric argument"), "{stderr}");
}

#[test]
fn timeout_invalid_value() {
    let out = quiet().args(["-t", "notanumber"]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("invalid timeout 'notanumber'"), "{stderr}");
}
