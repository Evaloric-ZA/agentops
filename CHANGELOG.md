# Changelog

All notable changes to AgentOps are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
uses [semantic versioning](https://semver.org/).

AgentOps v0.1 is a local proof of architecture. Everything in it is
synthetic. There is no real agent, no model API and no network service in
any release.

## [Unreleased]

Nothing yet.

## [0.1.2] - 2026-11-24

A maintenance, correctness, portability, verification and documentation
release. No feature is added, no screen changes, and the data model is
unchanged: a v0.1 database opens as a v0.1.2 database.

The development and continuous integration baseline is Node.js 24. Node.js
20 is no longer supported.

### Added

- **The privacy scanner decides what happened to every file.** It now
  reports six verdicts — scanned, binary skipped, generated excluded,
  oversize review required, unreadable, and a violation — and prints the
  six counters at the end of every run. An oversized file used to
  disappear from verification without a word; it is now reported, and the
  verification fails until a person has looked at it. A known dependency
  lockfile is not exempt from the search: it is read a line at a time,
  with the same patterns as every other file.
- **The scanner never prints the value it found.** A finding names the
  file, the line and the column, and the kind of pattern, and nothing
  else.
- **A regression test for the approval error handling.** A missing
  approval is reported as a missing approval, and a broken lookup is
  reported as a database error. The failure is manufactured by removing a
  column from the test's own database, so the production code carries no
  test hook.
- **Frontend integration tests at the command boundary.** Eighteen
  tests drive the real components against a deterministic mock of the
  command boundary: the shell, the navigation, the overview, the agent
  and run searches, the approval decision and its failure, the policy
  toggle and its not-found failure, the simulator, and the settings
  contract.
- **Windows in the continuous integration matrix.** The sweep now runs
  on Linux, macOS and Windows.
- **A continuous integration job that builds the Windows application.**
  The sweep compiles and links the application on Windows but does not
  bundle it, so nothing in the build touched the WebView2 loader, the
  Windows icon, WiX or NSIS. The `windows-app` job runs the Tauri
  bundler and fails if no `.msi` or NSIS installer comes out.
- **One implementation of the `cargo tauri` bridge.** It is now
  `scripts/ensure_cargo_tauri.py`, called by both the sweep and the
  Windows job.
- **The line endings are pinned.** `.gitattributes` checks every text
  file out with line feeds on every platform, which is what keeps the
  shell entry point's shebang working on a Windows checkout.
- **One list of checks for every platform.** The checks live in
  `scripts/verify.py`, so `scripts/verify.sh` for macOS and Linux and
  `scripts/verify.ps1` for Windows run the same checks in the same order
  and cannot drift apart.
- **A sweep that says what it skipped, and why.** A check that cannot
  run is reported as a skip with its reason, and is never counted as a
  pass.
- **A dependency audit as part of the sweep.** The shipped dependencies
  are the gate; the advisories in the development toolchain are reported
  and listed in `docs/SECURITY.md`.
- **The Rust dependency audit is a gate.** It denies warnings as well as
  vulnerabilities, so an unmaintained, an unsound or a yanked crate fails
  the sweep too. The advisories that have been accepted are listed in
  `scripts/rust_audit_allowlist.json`, each with its reason and the
  action that would close it; anything not listed fails.

### Fixed

- **The bundled SQLite carried a known vulnerability.** The Rust
  dependency audit found `RUSTSEC-2026-0459`, `CVE-2026-119375`, medium
  severity, in the SQLite C library that `rusqlite` compiles into the
  binary. `rusqlite` moved from 0.32 to 0.43 and `libsqlite3-sys` from
  0.30.1 to 0.40.0, which is past the 0.39.0 that carries the fix. The
  application's own use of the library did not change.
- **A policy update that failed said nothing.** The error was swallowed,
  so a policy that was not there left the operator looking at a switch
  that had not moved, with no explanation. The failure is now shown.
- **The confirmation of an approval was never shown.** It was rendered
  inside the detail drawer, and a decided approval leaves the pending
  list, so the drawer closed and the message went with it: the operator
  approved an approval and was told nothing. The confirmation is now
  rendered outside the drawer.
- **The application panicked when the window could not open.** It now
  prints the reason and leaves with a failure code.

### Changed

- **Node.js 24 is the baseline.** The package manifest declares the
  engine, `.node-version` carries `24`, and the continuous integration
  asks for it.
- **The documentation's counts are measured, not remembered.** The
  numbers in the documentation come from the sweep's own output.
- **The native build check is stated precisely.** It proves the
  application compiles and links. It does not prove that a bundle, an
  installer, a signature or a notarisation exists.
- **`rusqlite` moved from 0.32 to 0.43**, to pick up the fix for the
  SQLite vulnerability above. It needs Rust 1.89, which is below the
  declared minimum of 1.90.

## [0.1.1] - 2026-11-24

A maintenance release. No feature is added, no screen changes, and the
data model is unchanged: a v0.1 database opens as a v0.1.1 database.

### Added

- **A check that the Rust enums and the TypeScript unions agree.** The
  frontend mirrors four Rust enums as string unions. A value added on one
  side and forgotten on the other is not a compile error — the backend
  would send a value the frontend's union does not contain — so the sweep
  now compares them.

### Fixed

- **A tick that failed part way through left the simulator wedged.** The
  step and everything it writes were two separate commits, so a failure
  committed what had already been written while the step counter stayed
  where it was. Every retry then hit the same fault and the simulator was
  stuck for good. The step and its writes are now one transaction.
- **Enabling a policy that does not exist reported success.** The update
  matched no rows and the command said it had worked.
- **The wildcards in a search were matched as wildcards.** Searching for
  `%` or `_` returned every run. They are now matched literally.
- **A reset was not reproducible.** The seed recorded the wall clock, so
  every re-seed stored a different value and "reset demo data" did not
  restore the same state. The seed now records the anchor it was given.
- **A database fault was reported as a missing approval.** The state
  query discarded its error, so a genuine database problem was reported
  as "record not found" and sent the operator looking in the wrong place.
- **An identifier was measured in bytes while its message said
  characters.** The validator only accepts ASCII, so the difference shows
  only in which refusal explains the problem.
- **The documentation claimed twenty-four commands; there are
  twenty-three.**
- **The icon set could not be regenerated from its documented source.**
  The tree carried a renamed double-resolution icon, so running the icon
  generator again produced a different tree. The conventional name is
  back, and the privacy scanner no longer mistakes an image file name for
  an email address.

- **The simulator was changing the seeded data.** It advanced, failed and
  completed the seeded runs, and attached its approvals to seeded runs
  that already carried one — so a run could show two approvals, and the
  reviewer disagreement panel counted them wrongly. The seeded runs are
  the fixed baseline; the simulator now only ever acts on its own runs.
- **The simulator's six phases belonged to six different agents.** The
  phase and the agent were both derived from the step, so the cycle
  described as create, advance, advance, approval, fail, complete was
  spread across unrelated agents, and most ticks found no run of their own
  to act on. One agent now owns a whole cycle, and the phases are counted
  from the first tick so the cycle begins with the create.
- **The simulator failed runs that were waiting on a human decision.** A
  run cannot be failed behind the operator's back, so it now waits until
  the decision is made. The operator's decision is what moves it on.
- **A rejected run was left looking as though it were still running.** The
  reject path set the status but not the end time or the duration, so the
  run detail view read "still running" for a run that had been rejected.
- **A decided approval left the agent waiting.** The decision path never
  touched the agent, so an agent stayed in the `approval` state for good.
  An approved run continues and its agent goes back to work; a rejected
  run stops and its agent stops with it.

- **After the application restarted, the first approval decision failed.**
  The identifier counter is a process-local atomic that starts at one in
  every new process, and the database still held the identifiers the
  previous process had minted. The new process was therefore handed an
  identifier that was already in use, and the write failed on the primary
  key with *"UNIQUE constraint failed: approval_decisions.id"*. The same
  applied to every audit event.
  The counter is now advanced past every identifier already stored when
  the database is opened. The application is restarted far more often
  than it is reset, so this was reachable in normal use.

- **The application had no palette on its first paint.** The colours live
  only in `html[data-theme="dark"]` and `html[data-theme="light"]`, and
  the theme attribute was set after the settings had loaded
  asynchronously — so every launch rendered a frame with no colours at
  all before the settings arrived. The theme is now on the element in
  `index.html`, before the first paint.
- **The Timezone setting did nothing.** It was validated by the backend,
  offered by the Settings screen and persisted, but never reached a
  formatter: the timestamps were rendered in the machine's own zone. The
  formatters now use the selected zone, and an unusable zone falls back
  to the machine's zone rather than throwing. Timestamps now display in
  the selected zone — UTC by default — instead of the machine's zone.

- **The privacy scanner missed secrets in files it did not recognise.**
  It scanned only a list of known text suffixes, so a secret written into
  a `.sh`, `.env`, `.lock` or `.py` file was not detected. It now reads
  every file in the tree and scans it if it decodes as UTF-8 and holds no
  NUL byte, so a file's name cannot decide whether it is checked.
- **The privacy scanner missed the two commonest shapes of a secret.** A
  value without quotes — the usual `.env` form — and a quoted key — the
  usual JSON form — both passed the scan. Both are caught now, and so are
  the `credential` and `passwd` key names.
- **The privacy scanner missed key names with an underscore.** The key
  alternation ended in a word boundary, and an underscore is a word
  character, so `SECRET_KEY`, `DB_PASSWORD` and `PASSWORD_FILE` were all
  passed. A key may now carry a prefix or a suffix.
- **The version could drift between the manifests.** Nothing in the
  toolchain forces `package.json`, `src-tauri/Cargo.toml`,
  `src-tauri/tauri.conf.json` and `src-tauri/Cargo.lock` to agree, and
  the application does not display its version anywhere, so a bump that
  missed one of them was invisible. There is now a check, and it is part
  of the sweep.
- **The declared minimum supported Rust was false.** The crate claimed
  `rust-version = "1.77"`, but `tauri` itself requires 1.90 — 112
  dependencies in the resolved graph need more than 1.77. Cargo only
  warns about this, and a build on 1.77 fails with the obscure message
  *"feature `edition2024` is required"*. The claim is now 1.90, which is
  the true minimum, and it is verified on both toolchains.

### Added

- **The privacy scanner has its own regression test** —
  `scripts/test_verify_repo.py`, twenty cases at the time of that
  release, standard library only. It builds a throwaway tree for each
  case and asserts what the scanner does with it, so the false negatives
  above cannot come back.
- **The sweep builds the native application.** `cargo test` compiles the
  library and the binary, but the sweep never ran the Tauri build, so a
  tree that could not be assembled as an application still passed. The
  sweep now runs `cargo tauri build --debug --no-bundle`, which compiles
  and links the application without the platform bundler.
- **A changelog.** This file.
- **CI checks the minimum supported Rust.** `src-tauri/Cargo.toml`
  declares `rust-version`, and nothing verified the claim. A job now
  compiles the crate on that toolchain, and `scripts/check_msrv.py`
  compares the declared minimum against what every dependency in the
  resolved graph itself requires.
- **The sweep checks the minimum supported Rust too**, so a false claim
  is caught locally rather than only on a CI runner.

### Changed

- **The sweep now puts `node_modules/.bin` on the PATH itself.**
  `cargo tauri` looks for a binary named `cargo-tauri`, which lives in
  `node_modules/.bin`. The sweep created the bridge there but left the
  PATH to the caller, so on a CI runner the native build failed with
  *"no such command: `tauri`"*. The sweep is now self-contained.

- **The continuous integration actions are pinned to a commit** rather
  than to a tag, so a moved tag cannot change what the build runs.
- **CI cancels a superseded run** instead of competing with it for a
  runner, and it asks for only `contents: read`.

## [0.1.0] - 2026-11-23

The first release.

### Added

- A Tauri 2 desktop application: a Rust core, a Svelte 5 and TypeScript
  frontend, and SQLite persistence through `rusqlite`.
- Twelve screens: Night Shift, Agents, Runs, Waiting on You, Policies,
  Access, Compute, Audit Log, Tools, Team, Settings and Crew.
- Twenty-three typed Tauri commands. The frontend never receives a raw
  database error or a stack trace; every failure arrives as a stable code
  and a message that is safe to display.
- A deterministic simulator driven from the frontend, so the operator
  controls the pace and a replay reproduces the same events and the same
  identifiers.
- A synthetic dataset: ten agents, ten runs, thirty stages, thirty
  events, three approvals, six tools, six policies, four team members and
  five audit records, all invented and anchored to a fixed instant.
- A validation sweep — `scripts/verify.sh` — and continuous integration
  on Ubuntu and macOS.
- A privacy scan, `scripts/verify_repo.py`, that fails the build if a
  secret, an email address or a private network address is embedded in
  the tree.

[Unreleased]: https://github.com/Evaloric-ZA/agentops/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/Evaloric-ZA/agentops/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Evaloric-ZA/agentops/releases/tag/v0.1.0
