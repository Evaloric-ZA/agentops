# Packaging

## Prerequisites

| Platform | What is needed |
|---|---|
| Linux | `libwebkit2gtk-4.1-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev` |
| macOS | Xcode Command Line Tools (`xcode-select --install`) and Rust |
| Windows | Microsoft C++ Build Tools, the WebView2 runtime, and Rust |

Rust is installed with `rustup`. The frontend toolchain is Node 20 or newer.

### The `cargo tauri` command

`cargo tauri build` invokes a binary named `cargo-tauri`, but the npm
`@tauri-apps/cli` package installs one named `tauri`. `scripts/verify.sh`
recreates a three-line bridge at `node_modules/.bin/cargo-tauri` after
`npm ci`, so the documented command works verbatim:

```bash
export PATH="$PWD/node_modules/.bin:$PATH"
cargo tauri build --debug
```

Installing the Rust CLI instead (`cargo install tauri-cli --version "^2"`)
makes the bridge unnecessary.

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
