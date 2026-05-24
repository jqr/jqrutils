use std::env;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Command, ExitCode, Stdio};
use std::thread;

fn prefix_lines(pipe: impl Read, prefix: &str, mut out: impl Write) {
    let reader = BufReader::new(pipe);
    for line in reader.split(b'\n') {
        match line {
            Ok(data) => {
                let _ = out.write_all(prefix.as_bytes());
                let _ = out.write_all(&data);
                let _ = out.write_all(b"\n");
                let _ = out.flush();
            }
            Err(_) => break,
        }
    }
}

fn passthrough(pipe: impl Read, mut out: impl Write) {
    let mut buf = [0u8; 8192];
    let mut pipe = pipe;
    loop {
        match pipe.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                let _ = out.write_all(&buf[..n]);
            }
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut do_stdout = false;
    let mut do_stderr = false;
    let mut rest = &args[..];

    while let Some(arg) = rest.first() {
        match arg.as_str() {
            "--stdout" => { do_stdout = true; rest = &rest[1..]; }
            "--stderr" => { do_stderr = true; rest = &rest[1..]; }
            _ => break,
        }
    }

    if !do_stdout && !do_stderr {
        do_stdout = true;
        do_stderr = true;
    }

    if rest.len() < 2 {
        eprintln!("Usage: prefix [--stdout] [--stderr] <prefix> [--] <command> [args...]");
        return ExitCode::from(1);
    }

    let prefix = format!("{} ", &rest[0]);
    let cmd_args = if rest.get(1).map(|s| s.as_str()) == Some("--") {
        &rest[2..]
    } else {
        &rest[1..]
    };

    if cmd_args.is_empty() {
        eprintln!("Usage: prefix [--stdout] [--stderr] <prefix> [--] <command> [args...]");
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
            eprintln!("prefix: {}: {}", cmd_args[0], e);
            return ExitCode::from(127);
        }
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let prefix_clone = prefix.clone();
    let t1 = thread::spawn(move || {
        if do_stdout {
            prefix_lines(stdout, &prefix_clone, io::stdout().lock());
        } else {
            passthrough(stdout, io::stdout().lock());
        }
    });

    let t2 = thread::spawn(move || {
        if do_stderr {
            prefix_lines(stderr, &prefix, io::stderr().lock());
        } else {
            passthrough(stderr, io::stderr().lock());
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

    ExitCode::from(code as u8)
}
