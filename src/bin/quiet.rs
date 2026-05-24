use std::env;
use std::io::{self, Read, Write};
use std::process::{Command, ExitCode, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Copy, Clone)]
enum Output {
    Stdout,
    Stderr,
}

struct State {
    streaming: bool,
    buffer: Vec<(Output, Vec<u8>)>,
}

impl State {
    fn flush(&mut self) {
        for (output, data) in self.buffer.drain(..) {
            let _ = match output {
                Output::Stdout => io::stdout().write_all(&data),
                Output::Stderr => io::stderr().write_all(&data),
            };
        }
        self.streaming = true;
    }
}

fn read_pipe(mut pipe: impl Read, output: Output, state: &Mutex<State>) {
    let mut buf = [0u8; 8192];
    loop {
        match pipe.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                let data = buf[..n].to_vec();
                let mut s = state.lock().unwrap();
                if s.streaming {
                    drop(s);
                    let _ = match output {
                        Output::Stdout => io::stdout().write_all(&data),
                        Output::Stderr => io::stderr().write_all(&data),
                    };
                } else {
                    s.buffer.push((output, data));
                }
            }
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let (timeout, cmd_start) = match args.first().map(|s| s.as_str()) {
        Some("-t") => {
            let secs: f64 = match args.get(1) {
                None => {
                    eprintln!("quiet: -t requires a numeric argument");
                    return ExitCode::from(1);
                }
                Some(s) => match s.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("quiet: invalid timeout '{}', -t requires a numeric argument", s);
                        return ExitCode::from(1);
                    }
                },
            };
            (Some(secs), 2)
        }
        _ => (None, 0),
    };

    let cmd_args = &args[cmd_start..];
    let cmd_args = if cmd_args.first().map(|s| s.as_str()) == Some("--") {
        &cmd_args[1..]
    } else {
        cmd_args
    };

    if cmd_args.is_empty() {
        eprintln!("Usage: quiet [-t <seconds>] [--] <command> [args...]");
        return ExitCode::from(1);
    }

    let mut child = match Command::new(&cmd_args[0])
        .args(&cmd_args[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("quiet: {}: {}", cmd_args[0], e);
            return ExitCode::from(127);
        }
    };

    let state = Arc::new(Mutex::new(State {
        streaming: false,
        buffer: Vec::new(),
    }));

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let s1 = Arc::clone(&state);
    let t1 = thread::spawn(move || read_pipe(stdout, Output::Stdout, &s1));

    let s2 = Arc::clone(&state);
    let t2 = thread::spawn(move || read_pipe(stderr, Output::Stderr, &s2));

    let timeout_cancel = if let Some(secs) = timeout {
        let s3 = Arc::clone(&state);
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        thread::spawn(move || {
            use std::sync::mpsc::RecvTimeoutError;
            if rx.recv_timeout(Duration::from_secs_f64(secs)) == Err(RecvTimeoutError::Timeout) {
                let mut s = s3.lock().unwrap();
                if !s.streaming {
                    s.flush();
                }
            }
        });
        Some(tx)
    } else {
        None
    };

    let status = child.wait().expect("failed to wait on child");
    t1.join().unwrap();
    t2.join().unwrap();
    drop(timeout_cancel);

    let mut s = state.lock().unwrap();
    if !s.streaming && !status.success() {
        s.flush();
    }

    #[cfg(unix)]
    let code = {
        use std::os::unix::process::ExitStatusExt;
        status
            .code()
            .unwrap_or_else(|| status.signal().map(|s| 128 + s).unwrap_or(1))
    };
    #[cfg(not(unix))]
    let code = status.code().unwrap_or(1);

    ExitCode::from(code as u8)
}
