use std::env;
use std::io::{self, Read, Write};
use std::process::{Command, ExitCode, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let args = if args.first().map(|s| s.as_str()) == Some("--") {
        &args[1..]
    } else {
        &args
    };

    if args.is_empty() {
        eprintln!("Usage: errfail [--] <command> [args...]");
        return ExitCode::from(1);
    }

    let mut child = match Command::new(&args[0])
        .args(&args[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("errfail: {}: {}", args[0], e);
            return ExitCode::from(127);
        }
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let had_stderr = Arc::new(AtomicBool::new(false));

    let t1 = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut pipe = stdout;
        loop {
            match pipe.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let _ = io::stdout().write_all(&buf[..n]);
                }
            }
        }
    });

    let had_stderr_clone = Arc::clone(&had_stderr);
    let t2 = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut pipe = stderr;
        loop {
            match pipe.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    had_stderr_clone.store(true, Ordering::Relaxed);
                    let _ = io::stderr().write_all(&buf[..n]);
                }
            }
        }
    });

    let status = child.wait().expect("failed to wait on child");
    t1.join().unwrap();
    t2.join().unwrap();

    #[cfg(unix)]
    let code = {
        use std::os::unix::process::ExitStatusExt;
        status
            .code()
            .unwrap_or_else(|| status.signal().map(|s| 128 + s).unwrap_or(1))
    };
    #[cfg(not(unix))]
    let code = status.code().unwrap_or(1);

    if code != 0 {
        return ExitCode::from(code as u8);
    }

    if had_stderr.load(Ordering::Relaxed) {
        return ExitCode::from(2);
    }

    ExitCode::SUCCESS
}
