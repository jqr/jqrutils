# jqrutils

Small Unix command-line tools.

### quiet - suppress output unless it fails

```sh
quiet rsync -a src/ dest/        # silent on success, shows output on failure
quiet -t 10 docker build .       # starts streaming after 10s if still running
```

### errfail - treat stderr as failure

```sh
errfail terraform apply           # exit 2 if it printed warnings but "succeeded"
quiet errfail make build          # silent unless it fails or warns
```

### prefix - label output lines

```sh
prefix "[api]" ./start-api &
prefix "[web]" ./start-web &      # interleaved output, easy to tell apart
prefix --stderr "[warn]" make     # only prefix stderr, leave stdout alone
```

## Install

```sh
cargo install jqrutils
```

Or via Homebrew on macOS:

```sh
brew tap jqr/jqrutils
brew install jqrutils
```

## Release

```sh
bin/release 0.0.1
```

This runs tests, tags, pushes, and attempts to update the Homebrew tap.
