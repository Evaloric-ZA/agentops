# Packaging

## Prerequisites

| Platform | What is needed |
|---|---|
| Linux | `libwebkit2gtk-4.1-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev` |
| macOS | Xcode Command Line Tools (`xcode-select --install`) and Rust |
| Windows | Microsoft C++ Build Tools with the **Desktop development with C++** workload, the WebView2 runtime, and Rust |

Rust is installed with `rustup`. The frontend toolchain is **Node.js 24**;
`.node-version` carries `24` and the package manifest declares the engine.
Node.js 20 is no longer supported.

On Windows the C++ workload is not optional: the application compiles the
SQLite C library into its binary, so a C compiler is part of the build.
The WebView2 runtime is what the window is drawn in; it is preinstalled
on current Windows 10 and 11, and the Tauri bundler installs it for the
user if it is missing.

The line endings are the same on every platform. `.gitattributes` says so
explicitly, so a file checked out on Windows is the same file that was
checked out on Linux — which matters for `scripts/verify.sh`, whose
shebang a stray carriage return would break.

### The Tauri command

**Use `npm run tauri -- ...`.** It works the same on every platform,
because npm resolves the command line interface that the project already
depends on:

```bash
npm run tauri -- build --debug
```

```powershell
npm run tauri -- build --debug
```

The `--` matters: without it npm takes the arguments for itself.

**`cargo tauri ...` is a convenience on macOS and Linux.** Cargo invokes
a binary named `cargo-tauri`, but the npm `@tauri-apps/cli` package
installs one named `tauri`. A small bridge at
`node_modules/.bin/cargo-tauri` connects the two, so this works
verbatim:

```bash
export PATH="$PWD/node_modules/.bin:$PATH"
cargo tauri build --debug
```

**The bridge does not work on Windows.** Cargo looks for an external
subcommand as a program, and on the Windows runner it did not find the
`.cmd` bridge: the first run there failed with `no such command:
tauri`. That is why the sweep and the continuous integration use the npm
form, and why the bridge is only a convenience.

The bridge is created by `scripts/ensure_cargo_tauri.py`. The sweep
creates it after `npm ci` and again immediately before the native build,
and the continuous integration job that builds the Windows application
calls the same script, so there is one implementation of it.

Installing the Rust CLI instead (`cargo install tauri-cli --version "^2"`)
makes the bridge unnecessary.

The bridge has to be created **after** `npm ci`. `npm ci` removes
`node_modules` and installs it again, so a bridge written before it runs is
deleted along with everything else.

Cargo passes the name of the subcommand to an external subcommand, so the
bridge receives `tauri build`, not `build`. It drops that first name
before calling the Tauri command line interface.

### The sweep builds the application, but does not bundle it

`scripts/verify.sh` runs `cargo tauri build --debug --no-bundle`. That
compiles and links the application without the platform bundler, which
keeps the check usable on a machine with no interactive session and
without downloading bundler tooling.

It is deliberately **not** the same as producing an installer. The
`.app`, the `.dmg` and the Linux packages come from a separate
`cargo tauri build` run, and on macOS the `.dmg` step needs an
interactive Finder session.

### Six different things, and what proves each

It is easy to read "the build passed" as more than it is. These are
separate, and each needs its own evidence.

| | What it is | What proves it |
|---|---|---|
| A | the frontend production bundle | `npm run build` — `dist/` assembles |
| B | the Rust code compiles and its tests pass | `cargo build` and `cargo test` |
| C | the Tauri application compiles and links | `cargo tauri build --debug --no-bundle` — the sweep's native build |
| D | a native bundle or installer is produced | `cargo tauri build` — the `.app`, the `.dmg`, the `.deb`, the `.msi` |
| E | the bundle is signed | a signing identity and a signing step; v0.1 has none |
| F | the bundle is notarised | a notarisation step with an online submission; v0.1 has none |

**The sweep proves C.** It does not prove D, E or F, and a green sweep
must never be reported as a signed, notarised installer.

### What the continuous integration proves about Windows

Two jobs, and they prove different things.

- **The sweep on `windows-latest`** proves C on Windows: the application
  compiles and links there, with MSVC and the bundled SQLite.
- **The `windows-app` job** proves D on Windows: it runs the Tauri
  bundler, which produces the `.msi` and the NSIS installer, and it
  fails if no installer comes out. That step needs the WebView2 loader,
  the `.ico`, WiX and NSIS, none of which the sweep touches.

Neither job proves E or F. v0.1 is unsigned.

### What was checked here, and what was not

On this host, the application was checked against a **Windows target**
without a Windows machine:

| Target | Result | What it proves |
|---|---|---|
| `x86_64-pc-windows-gnu` | passes | the whole Rust tree compiles for Windows, including the bundled SQLite C code, `webview2-com`, `muda` and `tao` |
| `x86_64-pc-windows-msvc` | passes | the same, for the target the continuous integration uses, including the Windows resource embedding that puts the icon and the version into the binary |

The MSVC check needed two things this host does not have by default: a C
compiler for the bundled SQLite (it is the one part of the MSVC target
that cannot be checked without `cl.exe`) and `llvm-rc` for the resource
compiler. The resource compiler is in the `llvm@22` Homebrew formula,
which is keg-only, so it is not on the path by default.

**Neither check is a substitute for the job.** A cross-compiled check
proves the code compiles for Windows; it does not prove the installer is
produced, and it does not prove the application runs. Those are the
continuous integration's to report.

**And the job has now run.** On the Windows runner the continuous
integration produced both installers:

| Artefact | Size |
|---|---|
| `AgentOps_0.1.2_x64_en-US.msi` | 4.77 MiB |
| `AgentOps_0.1.2_x64-setup.exe` | 2.97 MiB |

The WiX toolset and NSIS are downloaded and verified by the bundler, the
icon is embedded in the executable, and the application itself is built
with MSVC. The sweep on the same runner reports fourteen passed, zero
failed and one skipped — the skip is the Rust dependency audit, because
`cargo-audit` is not installed on the runner.

What that establishes: the Windows application builds and can be
installed. What it does not establish: that it was installed and run. A
runner is a machine with no person and no display, and no test on it
starts the application.

## Development build

```bash
npm ci
npm run tauri dev
```

The dev server listens on `http://localhost:1420` and the Tauri window
loads it.

## Release build

```bash
npm ci
npm run tauri build
```

This produces:

| Output | Path |
|---|---|
| The application bundle | `src-tauri/target/release/bundle/` |
| The binary | `src-tauri/target/release/agentops` |
| The frontend bundle | `dist/` |

The bundle directory contains the platform installer: a `.deb` and
`.rpm` and an AppImage on Linux, a `.app` and a `.dmg` on macOS, and an
`.msi` and an NSIS installer on Windows.

## Debug build

```bash
npm run tauri build -- --debug
```

The debug build is what the acceptance sweep uses, because it is much
faster to produce and is enough to prove the application assembles and
links.

## Before a release is cut

A release is verified by the release verification, not by the developer
one. The difference is that a check which cannot run is a failure rather
than a skip:

```bash
bash scripts/verify.sh --release
```

The command needs `cargo-audit`, because the release verification
refuses to pass without it. It is not part of the Rust toolchain:

```bash
cargo install cargo-audit --version 0.23.0 --locked
cargo audit --version
```

The version is pinned for the same reason it is pinned in the continuous
integration: a tool that floats is a gate that moves. The continuous
integration installs the same version, so the local and the continuous
runs audit with the same rules.

What the release verification does that the developer one does not:

- it fails when `cargo-audit` is not installed;
- it fails when the advisory database cannot be fetched;
- it fails when the package registry cannot be reached;
- it fails when an advisory exception has passed its review date;
- it fails when an advisory exception names an advisory that no longer
  exists;
- it fails on any check that cannot run at all.

The two dependency gates can also be run on their own, which is what the
continuous integration security job does:

```bash
python3 scripts/npm_audit_gate.py --release .
python3 scripts/rust_audit_gate.py --release .
```

They exit with 0 when they pass, 1 when they fail and 2 when they could
not run.

The continuous integration has a job for each of the two: the security
gate, which runs the gates and the scanners, and the release
verification, which runs the whole sweep in release mode. Both install
the pinned audit tool, and both fail if it cannot be installed.

## How releases are made

There is **one branch: `main`**. It always carries the current code,
and it is the branch a release is cut from.

**A patch release inside v0.1 goes straight onto `main`.** There is no
branch for it: the change is made, the sweep is run, the commit is
pushed, and the version is bumped. v0.1.0, v0.1.1, v0.1.2 and v0.1.3
were all made this way.

**A branch is created for a version bump** — a new minor or major
version, such as v0.2 — because that is the point at which the work is
large enough to deserve its own review and its own line of history. The
branch is merged back into `main` with `--no-ff`, the merge commit is
what the tag names, and the branch is deleted afterwards. Nothing is
left on the remote except `main` and the tags.

So the rule is: **v0.1.x changes go straight to `main`; a branch is for
a version bump.**

Before a release is cut, the release verification has to pass, and the
version history has to describe what changed:

```bash
bash scripts/verify.sh --release
```

The version history is `CHANGELOG.md`. It records every release, what
changed in it and what it fixed, and each version is a tag.

## The application data directory

On first launch the application creates its database and seeds it:

| Platform | Path |
|---|---|
| Linux | `~/.local/share/dev.agentops.dashboard/agentops.db` |
| macOS | `~/Library/Application Support/dev.agentops.dashboard/agentops.db` |
| Windows | `%APPDATA%\dev.agentops.dashboard\agentops.db` |

Deleting that file resets the application to a clean first launch.

## Signing and notarisation

v0.1 is unsigned. There is no signing identity, no notarisation and no
auto-update configuration, and none is implied. A distributed build needs
its own signing setup.
