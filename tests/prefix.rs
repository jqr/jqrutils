use std::process::Command;

fn prefix() -> Command {
    Command::new(env!("CARGO_BIN_EXE_prefix"))
}

#[test]
fn prefixes_stdout() {
    let out = prefix()
        .args(["[build]", "sh", "-c", "echo hello"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "[build] hello\n");
}

#[test]
fn prefixes_stderr() {
    let out = prefix()
        .args(["[build]", "sh", "-c", "echo warn >&2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stderr), "[build] warn\n");
}

#[test]
fn prefixes_multiple_lines() {
    let out = prefix()
        .args(["[x]", "sh", "-c", "echo a; echo b; echo c"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "[x] a\n[x] b\n[x] c\n"
    );
}

#[test]
fn preserves_exit_code() {
    let out = prefix()
        .args(["[x]", "sh", "-c", "exit 42"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(42));
}

#[test]
fn missing_command_exits_127() {
    let out = prefix()
        .args(["[x]", "nonexistent_command_xyz_123"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(127));
}

#[test]
fn no_args_shows_usage() {
    let out = prefix().output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage:"));
}

#[test]
fn prefix_only_no_command_shows_usage() {
    let out = prefix().args(["[x]"]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage:"));
}

#[test]
fn double_dash_separator() {
    let out = prefix()
        .args(["[x]", "--", "sh", "-c", "echo hello"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "[x] hello\n");
}

#[test]
fn both_stdout_and_stderr() {
    let out = prefix()
        .args(["[mix]", "sh", "-c", "echo out; echo err >&2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "[mix] out\n");
    assert_eq!(String::from_utf8_lossy(&out.stderr), "[mix] err\n");
}

#[test]
fn stdout_only_flag() {
    let out = prefix()
        .args(["--stdout", "[x]", "sh", "-c", "echo out; echo err >&2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "[x] out\n");
    assert_eq!(String::from_utf8_lossy(&out.stderr), "err\n");
}

#[test]
fn stderr_only_flag() {
    let out = prefix()
        .args(["--stderr", "[x]", "sh", "-c", "echo out; echo err >&2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "out\n");
    assert_eq!(String::from_utf8_lossy(&out.stderr), "[x] err\n");
}

#[test]
fn both_flags_same_as_default() {
    let out = prefix()
        .args(["--stdout", "--stderr", "[x]", "sh", "-c", "echo out; echo err >&2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "[x] out\n");
    assert_eq!(String::from_utf8_lossy(&out.stderr), "[x] err\n");
}
