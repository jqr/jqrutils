# jqrutils

Small Unix command-line tools.

### quiet


```sh
# quiet: Suppress command output unless it fails (or takes too long).
quiet make build          # exit 0 means it will hide all output
quiet -t 5 make build     # after 5 seconds it will start streaming

# errfail: Any stderr output forces non-zero exit code if would've exited 0.
errfail make build        # wrote to stderr? exit 2

# prefix: Prefix each line of stdout and stderr with a label.
prefix "[build]" make build           # both stdout and stderr
prefix --stdout "[build]" make build  # only stdout
prefix --stderr "[build]" make build  # only stderr

quiet errfail make build  # composable
```

## Install

```sh
cargo install jqrutils
```

Or via Homebrew on macOS:

```sh
brew tap jqr/jqrutils
brew install quiet errfail
```

## Release

```sh
bin/release 0.0.1
```

This runs tests, tags, and pushes. Then update the Homebrew tap:

```sh
cd ../homebrew-jqrutils
bin/update 0.0.1
```
