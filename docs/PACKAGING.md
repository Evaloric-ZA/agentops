# Packaging

## Prerequisites

| Platform | What is needed |
|---|---|
| Linux | `libwebkit2gtk-4.1-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev` |
| macOS | Xcode Command Line Tools (`xcode-select --install`) and Rust |
| Windows | Microsoft C++ Build Tools, the WebView2 runtime, and Rust |

Rust is installed with `rustup`. The frontend toolchain is **Node.js 24**;
`.node-version` carries `24` and the package manifest declares the engine.
Node.js 20 is no longer supported.

### The `cargo tauri` command

`cargo tauri build` invokes a binary named `cargo-tauri`, but the npm
`@tauri-apps/cli` package installs one named `tauri`. `scripts/verify.sh`
recreates a three-line bridge at `node_modules/.bin/cargo-tauri`, so the
documented command works verbatim:

```bash
export PATH="$PWD/node_modules/.bin:$PATH"
cargo tauri build --debug
```

Installing the Rust CLI instead (`cargo install tauri-cli --version "^2"`)
makes the bridge unnecessary.

The bridge has to be created **after** `npm ci`. `npm ci` removes
`node_modules` and installs it again, so a bridge written before it runs is
deleted along with everything else. The sweep creates it after `npm ci`
and again immediately before the native build.

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
