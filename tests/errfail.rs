use std::process::Command;

fn errfail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_errfail"))
}

#[test]
fn clean_exit_no_stderr() {
    let out = errfail()
        .args(["sh", "-c", "echo hello"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello\n");
    assert!(out.stderr.is_empty());
}

#[test]
fn clean_exit_with_stderr_exits_2() {
    let out = errfail()
        .args(["sh", "-c", "echo out; echo warn >&2"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "out\n");
    assert_eq!(String::from_utf8_lossy(&out.stderr), "warn\n");
}

#[test]
fn nonzero_exit_preserved() {
    let out = errfail().args(["sh", "-c", "exit 42"]).output().unwrap();
    assert_eq!(out.status.code(), Some(42));
}

#[test]
fn stderr_only_exits_2() {
    let out = errfail()
        .args(["sh", "-c", "echo warn >&2"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&out.stderr), "warn\n");
}

#[test]
fn passes_stdout_through() {
    let out = errfail()
        .args(["sh", "-c", "echo line1; echo line2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "line1\nline2\n");
}

#[test]
fn missing_command_exits_127() {
    let out = errfail()
        .args(["nonexistent_command_xyz_123"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(127));
}

#[test]
fn no_args_shows_usage() {
    let out = errfail().output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage:"));
}

#[test]
fn double_dash_separator() {
    let out = errfail()
        .args(["--", "sh", "-c", "echo hello"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello\n");
}

#[test]
fn nonzero_exit_takes_priority_over_stderr() {
    let out = errfail()
        .args(["sh", "-c", "echo warn >&2; exit 7"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(7));
}
