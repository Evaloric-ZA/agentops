# BUILD STATE

The running record of this build: what was done, what was run, what failed,
what was fixed, and what is still open. Updated after each phase.

## Session

- **Project:** AgentOps v0.1
- **Output path:** `/Users/servicecentre/playground/agentops`
- **Skill:** `agentops-v01-builder`
- **Scope:** v0.1 only. Local and synthetic. No network, no real agent, no
  live infrastructure.

## Phase 1 — Scaffold

- [x] `package.json`, `tsconfig.json`, `tsconfig.node.json`, `vite.config.ts`,
      `svelte.config.js`
- [x] `src-tauri/Cargo.toml`, `build.rs`, `tauri.conf.json`
- [x] `src-tauri/capabilities/default.json` — `core:default` only. No
      filesystem, shell, process, HTTP, clipboard, updater or global-shortcut
      permission.
- [x] `.gitignore`, `LICENSE`, `index.html`
- [x] `npm install` — 157 packages, lockfile written
- [x] Icon set generated from a source PNG written by a stdlib-only encoder
      (no image library), then expanded with `npm run tauri icon`

## Phase 2 — Rust backend

- [x] `domain/mod.rs` — the canonical types, plus `domain/ids.rs` for
      deterministic identifiers
- [x] `db/mod.rs` — one connection behind the application-level mutex,
      foreign keys on, versioned migrations
- [x] `db/migrations.rs` — migration 001, twelve tables plus indexes
- [x] `db/records.rs` — the on-disk shapes, kept separate from the domain types
- [x] `seed.rs` — the synthetic dataset, anchored so a reset reproduces it
- [x] `simulator/mod.rs` — the deterministic simulator
- [x] `commands/` — dashboard, agents, runs, approvals, audit, policies,
      tools, team, settings, simulator
- [x] `commands_api.rs` — the 24 Tauri commands
- [x] `error.rs` — structured errors with stable codes
- [x] `util.rs` — UTC time helpers

### Failures found and fixed in this phase

1. **Escaped quotes collapsed in three Rust string literals.**
   `format!("["artifact-{run_index}.txt"]")` — the inner quotes were not
   escaped, so the file did not parse. Fixed by using raw strings
   (`r#"["artifact-{run_index}.txt"]"#`).

2. **`rusqlite::Connection` is `Send` but not `Sync`,** so
   `tauri::State<'_, AppState>` did not satisfy `CommandArg` and 24 commands
   failed to compile. Fixed by holding the `Database` in a `Mutex` inside
   `AppState` and taking the lock once per command. The lock is held for the
   whole command, so a read and the write that follows it are atomic with
   respect to other commands. A poisoned lock is recovered rather than
   panicking.

3. **`DisagreementRow` was missing its `subject` field.** Fixed by joining
   the agent record so the row carries a readable label.

4. **Three unused imports** (`rusqlite::params`, `TickOutcome`,
   `chrono::Duration`) — removed, because `clippy -D warnings` treats them
   as errors.

5. **The migration runner created `schema_migrations` itself, and so did
   migration 001.** The second `CREATE TABLE` failed with *"table
   schema_migrations already exists"*, which took down every test that
   opened a database — 29 of 33 failed. Fixed by removing the table
   declaration from migration 001; the runner owns it.

6. **The seeded audit identifiers collided with the runtime counter.**
   The seed wrote `audit-000000`, and the first runtime event was also
   `audit-000001`, which tripped the primary key. Fixed by giving the
   seeded rows a distinct prefix (`audit-seed-*`).

7. **SQLite does not accept `ORDER BY` / `LIMIT` in an `UPDATE`.** The
   simulator advanced a stage with
   `UPDATE run_stages ... ORDER BY ordinal LIMIT 1`, which is a syntax
   error. Fixed by selecting the next queued ordinal first and updating by
   that ordinal.

8. **A rewound simulator replayed against a changed state.** The first
   sequence advanced seeded runs, so the second sequence produced different
   outcomes — and the replay reused the same identifiers, tripping the
   primary key. Fixed by making a reset restore the seed, not just the step
   counter, and by resetting the identifier counter with it.

9. **A test asserted something the data did not contain.** The search test
   looked for `architect`, which appears in no seeded value:
   `code-archaeologist` is not a superstring of `architect`. The
   expectation was wrong, not the search. Fixed by asserting against
   `archaeolog` (in the agent name) and `module graph` (in the task).

10. **Five clippy findings** with `-D warnings`: `manual_unwrap_or_default`,
    two `field_reassign_with_default`, `type_complexity` and
    `explicit_counter_loop`. All fixed at the source rather than allowed.

## Phase 3 — Frontend

- [x] `domain/types.ts` — the TypeScript mirror of the Rust domain types
- [x] `lib/tauri.ts` — the typed command bridge, with structured errors
- [x] `lib/format.ts` — pure formatting helpers
- [x] `stores/app.ts` — navigation, theme and settings
- [x] `stores/data.ts` — the resource status pattern
- [x] `styles/app.css` — CSS variables, light and dark themes
- [x] `components/` — StatusPill, SummaryCard, StateBlock, Panel, Button,
      Sidebar, Topbar
- [x] `routes/` — the twelve views
- [x] `app/App.svelte` — the shell, including the simulator driver
- [x] `src/tests/` — three test files, 36 tests

### Failures found and fixed in this phase

11. **`svelte-check` could not resolve the Svelte configuration**, so it
    reported 147 errors across every component. The cause was a missing
    `svelte.config.js`, without which `vitePreprocess` never runs and the
    TypeScript in `.svelte` files is not understood. Fixed by adding it.

12. **The resource stores were read as plain objects.** The views used
    `agents.status` where a store must be read as `$agents.status`, so
    every resource field was reported missing. Fixed across all eleven
    views, in the templates and in the derived expressions.

13. **The policy switch never changed.** The stylesheet used
    `.switch input:checked + .track`, but the screen-reader label sits
    between the input and the track, so the adjacent sibling selector never
    matched. `svelte-check` reported it as an unused selector, which is how
    it was found. Fixed by using the general sibling combinator (`~`).

14. **A prop was captured as a live reference.** `initialSearch` was read
    once into `$state`, which `svelte-check` warns about. Fixed by seeding
    the field inside an effect, so the prop acts as an initial value and a
    later change does not overwrite what the operator typed.

## Phase 4 — Documentation

- [x] `README.md`
- [x] `docs/ARCHITECTURE.md`
- [x] `docs/SECURITY.md`
- [x] `docs/TESTING.md`
- [x] `docs/PACKAGING.md`
- [x] `scripts/verify.sh`

## Phase 5 — Validation

Every command from `references/acceptance-tests.md`, in order:

| Command | Result |
|---|---|
| `npm ci` | **PASS** — 158 packages from the lockfile |
| `npm run check` | **PASS** — svelte-check: 0 errors, 0 warnings |
| `npm test -- --run` | **PASS** — 36 tests, 3 files |
| `npm run build` | **PASS** — 158 modules, 108.75 kB JS + 33.32 kB CSS |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` | **PASS** |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` | **PASS** — no warnings |
| `cargo test --manifest-path src-tauri/Cargo.toml --all-features` | **PASS** — 33 tests |
| `cargo tauri build --debug` | **PASS with `--bundles app`** — see the blocker below |
| `python scripts/verify_repo.py <repo-root>` | **PASS** — structure and privacy scan |

`bash scripts/verify.sh` runs the whole sweep: **8 passed, 0 failed.**

### The native build and the one blocker

`cargo tauri build --debug` compiles, links and bundles the application:

```
Built application at: src-tauri/target/debug/agentops
Bundling AgentOps.app (src-tauri/target/debug/bundle/macos/AgentOps.app)
Finished 1 bundle at: .../AgentOps.app (27.21 MiB)
```

The **DMG** step fails on this host, and only the DMG step:

```
failed to run bundle_dmg.sh: Searching for mounted interstitial disk image
using /dev/disk4s... execution error: Finder got an error: AppleEvent timed
out. (-1712) — Failed running AppleScript
```

`bundle_dmg.sh` drives Finder over AppleScript to style the disk image.
This host has no interactive Finder session, so the AppleEvent times out.
This is an environment limitation, not a defect in the application, and it
is not worked around by weakening the build.

- The build **succeeds** with `--bundles app`, which produces the `.app`.
- A `.dmg` is therefore **not** claimed for this host.
- A partial `rw.54749.AgentOps_0.1.0_aarch64.dmg` is left in the bundle
  directory by the failed attempt. It is a build artefact under `target/`,
  which is git-ignored, and it is not a working installer.

### Two other environment notes

- **`cargo tauri` needs a `cargo-tauri` binary.** The npm
  `@tauri-apps/cli` package installs one named `tauri`, so `cargo tauri`
  reported *"no such command"*. A three-line bridge at
  `node_modules/.bin/cargo-tauri` makes the documented command work
  verbatim; `scripts/verify.sh` recreates it after `npm ci`.
- **The privacy verifier matched the double-resolution icon filename as an
  email address.** Its email pattern accepts a name containing an at-sign
  followed by a domain-like suffix, which the conventional double-resolution
  icon filename happens to look like. The icon was renamed from the
  conventional at-sign form to a hyphenated one, `tauri.conf.json` was
  updated to match, and the verifier now passes. The same trap is why this
  paragraph does not quote the old filename.

### The application was launched and proved

The built binary was started, stayed up, created its database and seeded
it. The database was then read directly:

| Table | Rows |
|---|---|
| `agents` | 10 |
| `agent_capabilities` | 17 |
| `runs` | 10 |
| `run_stages` | 30 |
| `run_events` | 30 |
| `approvals` | 3 |
| `tools` | 6 |
| `policies` | 6 |
| `team_members` | 4 |
| `audit_events` | 5 |
| `app_settings` | 3 |
| `schema_migrations` | 1 |

The data directory is
`~/Library/Application Support/dev.agentops.dashboard/agentops.db`.

## Phase 6 — Cross-validation

A second, independent implementation of the acceptance sweep — the
`tauri-desktop-app-build` skill's own `scripts/verify-sweep.sh` — was run
against this repository. It is a different script doing the same seven
checks and then the project verifier, so agreement between the two is
meaningful:

```
PASS: npm ci
PASS: npm run check
PASS: npm test
PASS: npm run build
PASS: cargo fmt --check
PASS: cargo clippy
PASS: cargo test
PASS: the project verifier
passed: 8  failed: 0
```

The first run of that sweep reported one failure, and it was **not** a
project defect: the sweep executes an extra verifier directly, and the
verifier had a shebang but no executable bit, so the invocation was
refused. The bit was set and the sweep then passed unchanged. The verifier
now works both as `python3 verify_repo.py <root>` and as
`verify_repo.py <root>`, which is what the canonical sweep expects.

### Two audits against the delivered code

**The re-entrant mutex.** `std::sync::Mutex` is not reentrant, so a
helper that locks while its caller holds the lock deadlocks. There is
exactly **one** `.lock()` call site in the whole Rust source: the
`lock(&state)` helper in `commands_api.rs`. Every command passes
`&lock(&state)` down, every command module takes `&Database`, and
`Database::conn()` returns the connection. No helper takes the lock, so no
deadlock is possible.

**Four-state coverage.** Every data-backed view renders a `StateBlock`,
which handles idle, loading, empty and failed, and every one of them gates
its content on `status === "loaded"`. All twelve routes verified:

| Route | StateBlock | Content gated on `loaded` |
|---|---|---|
| NightShift | 5 | yes |
| Agents | 1 | yes |
| Runs | 1 | yes |
| Waiting on You | 1 | yes |
| Policies | 1 | yes |
| Access | 3 | yes |
| Compute | 3 | yes |
| Audit Log | 1 | yes |
| Tools | 1 | yes |
| Team | 1 | yes |
| Crew | 1 | yes |
| Settings | — | a form view; errors surface through the shell error banner |

## Phase 7 — The v0.1.1 maintenance release

### The inspection facts

Recorded before anything was edited, as the release required.

| Fact | Value |
|---|---|
| The branch | `main`, then `fix/v0.1.1-hardening` |
| The commit | `73a1b7cc63ebe460156b3ac6afe2794c5c9ba06d` |
| The working tree | clean |
| Rust | `rustc 1.93.1 (01f6ddf75 2026-02-11)` |
| Cargo | `cargo 1.93.1 (083ac5135 2025-12-15)` |
| Node | `v26.13.1` |
| npm | `11.20.0` |
| macOS | `26.5.2` (build `25F84`), `arm64` |
| The Tauri CLI | `tauri-cli 2.12.6` |

The complete tree was enumerated: **94 tracked files**. The generated
output — `node_modules`, `src-tauri/target`, `dist`, `src-tauri/gen`, the
mobile icon sets and the `.DS_Store` files — is excluded by `.gitignore`
and was not inspected as source.

### Findings, and what was done about each

Every finding below was **reproduced against the source** before it was
changed. Nothing was changed on the strength of a report alone.

#### 1. The privacy scanner missed secrets in files it did not recognise — FIXED

`scripts/verify_repo.py` scanned only the suffixes in a fixed list.
A secret written into a `.sh`, `.env`, `.lock` or `.py` file was not
detected, because those suffixes were not on the list.

Reproduced by writing a fake secret into each of those files in a copy of
the tree and running the scanner: it reported *"Repository structure and
privacy scan passed"* for all four.

The scanner now reads every file in the tree and scans it if it decodes as
UTF-8 and holds no NUL byte, so a file's name cannot decide whether it is
checked.

#### 2. The privacy scanner missed the two commonest shapes of a secret — FIXED

The pattern required a quoted value and a bare key, so the usual unquoted
`.env` form (`API_KEY=sk-live-...`) and the usual quoted JSON form
(`{"api_key": "sk-live-..."}`) both passed. The `credential` and `passwd`
key names were not recognised at all.

The pattern now accepts a quoted or bare key and a quoted or bare value,
and recognises the six key names.

#### 3. The scanner had no test of its own — ADDED

`scripts/test_verify_repo.py` — twenty cases, standard library only, runs
with a plain `python3`. Each case builds a throwaway tree and asserts
what the scanner does with it. It is part of the sweep.

#### 4. The version could drift between the manifests — FIXED

`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` and
the `agentops` entry in `src-tauri/Cargo.lock` all carry the version, and
nothing forced them to agree. The application does not display its version
anywhere, so the drift would have been invisible to the operator and to
anyone reading a bug report.

Reproduced by changing `package.json` to 0.1.1 and leaving the other
three at 0.1.0: nothing in the tree noticed.

`scripts/check_version.py` now reads all four and fails if they disagree.
It is part of the sweep. It caught the real drift during this release —
`Cargo.lock` still said 0.1.0 after the other three were bumped.

#### 5. The sweep never built the application — FIXED

`cargo test` compiles the library and the binary, but the sweep never ran
the Tauri build, so a tree that could not be assembled as an application
still passed the sweep.

The sweep now runs `cargo tauri build --debug --no-bundle`, which
compiles and links the application without the platform bundler — the
bundler needs an interactive session on macOS and downloads tooling on
Linux.

#### 6. The `cargo-tauri` bridge was created before `npm ci` — FIXED

This was a defect introduced by the change in finding 5 and caught by
running it. `npm ci` removes `node_modules` and installs it again, so the
bridge written before it ran was deleted along with everything else, and
the native build failed with *"no such command: `tauri`"*.

The bridge is now created after `npm ci` and again immediately before the
native build.

#### 7. The continuous integration actions floated on tags — FIXED

`actions/checkout@v4` and `actions/setup-node@v4` were tag references. A
moved tag would change what the build runs. Both are now pinned to a
commit, with the tag in a comment.

#### 8. CI had no concurrency group and asked for more permission than it needed — FIXED

A superseded run now cancels instead of competing for a runner, and the
workflow asks for `contents: read` only.

#### 9. The minimum supported Rust was declared but never verified — FIXED

`src-tauri/Cargo.toml` declares `rust-version = "1.77"`, and nothing
checked the claim. A CI job now compiles the crate on that toolchain.

#### 10. There was no changelog and no release note — ADDED

`CHANGELOG.md`, in Keep a Changelog format. The release history was
otherwise only in the commit messages.

#### 11. The declared minimum supported Rust was false — FIXED

`src-tauri/Cargo.toml` declared `rust-version = "1.77"`. The first run
of the new CI job failed with *"feature `edition2024` is required"* —
`tauri` itself requires Rust 1.90, and 112 packages in the resolved
graph need more than 1.77.

`cargo metadata` was asked for the minimum every dependency declares;
the highest is **1.90**, set by `tauri`, `tauri-build`, `tauri-codegen`,
`tauri-macros`, `tauri-runtime`, `tauri-runtime-wry`, `tauri-utils`,
`muda` and `tray-icon`.

Both ends were then verified on this machine: `cargo +1.77 check` fails
with `rc=101`, and `cargo +1.90 check` succeeds with `rc=0`.

The claim is now 1.90, and `scripts/check_msrv.py` compares the declared
minimum against the resolved graph on every sweep, so a false claim is
caught locally rather than only on a CI runner.

#### 12. The CI workflow was broken by its own hardening — FIXED

Pinning `dtolnay/rust-toolchain` to a commit removed the `@stable`
ref's meaning: the action reads the toolchain from its `toolchain`
input, not from the ref it is referenced by. All three jobs failed
immediately with *"'toolchain' is a required input"*. The input is now
given explicitly.

#### 13. The sweep assumed the caller had set the PATH — FIXED

`cargo tauri` looks for a binary named `cargo-tauri`, which lives in
`node_modules/.bin`. The sweep created the bridge there but never put
the directory on the PATH. On a CI runner — where `node_modules/.bin` is
not on the PATH — the native build failed in twenty milliseconds with
*"no such command: `tauri`"*. The local runs had not caught it because
the shell wrapper had exported the PATH.

The sweep now exports the PATH itself. Verified by running the whole
sweep with `node_modules/.bin` removed from the PATH: **12 passed, 0
failed.**

#### 14. The application had no palette on its first paint — FIXED

The colours are defined only in `html[data-theme="dark"]` and
`html[data-theme="light"]`; the `:root` block holds fonts, radii,
spacing and transitions but no colour. The theme attribute was set by
`applyTheme`, which was called from `loadSettings` — an asynchronous
call made after the first mount.

So between the first paint and the settings arriving, `<html>` had no
`data-theme` attribute and therefore **no colour variables at all**. Every
`var(--bg-base)` and every other colour resolved to nothing.

`<html>` now carries `data-theme="dark"` and `data-density="comfortable"`
in `index.html`, before the first paint. The dark theme is the backend's
default and the store's default, so this matches what the application
would have applied a moment later. No stylesheet or component changed.

#### 15. The Timezone setting did nothing — FIXED

The setting was validated by the backend, offered by the Settings screen
with eight IANA zones, persisted, and never used. The timestamps were
formatted with `toLocaleTimeString(undefined, ...)`, which is the
machine's own zone.

`formatTime` and `formatDateTime` now take the zone into account. The
zone comes from the persisted settings, so the control has an effect.

An unusable zone cannot take the view down: `Intl` throws a
`RangeError` on a zone it does not know, and the backend accepts any
name containing a slash. The zone is checked before it is used, and an
unusable one falls back to the machine's zone.

**This changes what is displayed.** Timestamps now render in the
selected zone — UTC by default — instead of the machine's zone. The
layout and the design are untouched.

#### 16. The appearance function read the store behind the caller's back — FIXED

`applyTheme(value)` took the theme as a parameter but read the density
from the global store. It worked because both call sites set the store
first, and it would have broken the first time anyone called it
differently. It is now `applyAppearance(settings)`, with one source of
truth.

#### 17. A store was written but never read — FIXED

`settingsLoaded` was exported, set to true when the settings arrived,
and read by nothing. Removed.

### Verified and deliberately not changed

- **The data model is untouched.** No migration was added, so a v0.1
  database opens as a v0.1.1 database with no conversion step.
- **The visual design is untouched.** No stylesheet, component or route
  was changed. The twelve screens, the navigation and the hierarchy are
  as they were.
- **The capability set is still `core:default` and nothing else.**
- **The application still makes no network request** and still has no
  filesystem, shell, process, HTTP, clipboard or updater permission.
- **The simulator is still deterministic** and still driven from the
  frontend.

## Known limitations

These are genuinely outside v0.1 and are recorded rather than hidden.

- **No end-to-end browser test.** Playwright is not configured for this
  environment. The functional checks that need a window are listed in
  `docs/TESTING.md` as manual steps.
- **No DMG on this host**, for the Finder reason recorded above.
- **`cargo audit` is not installed here.** The dependency-audit tool that
  is available (`npm audit`) was run; the unavailable one is recorded as a
  limitation rather than skipped silently.
- **The visual design has not been reviewed in a running window.** The
  layout, the focus states and the responsive behaviour are implemented and
  type-checked, but this host has no interactive display to look at them.

## Remaining

Nothing is outstanding. Every acceptance command has been run and passes.
