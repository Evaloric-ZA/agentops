# BUILD STATE

The running record of this build: what was done, what was run, what failed,
what was fixed, and what is still open. Updated after each phase.

## Session

- **Project:** AgentOps
- **Repository root:** the root of this repository. Every path in this
  document is relative to it.
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
- [x] `commands_api.rs` — the 23 Tauri commands
- [x] `error.rs` — structured errors with stable codes
- [x] `util.rs` — UTC time helpers

### Failures found and fixed in this phase

1. **Escaped quotes collapsed in three Rust string literals.**
   `format!("["artifact-{run_index}.txt"]")` — the inner quotes were not
   escaped, so the file did not parse. Fixed by using raw strings
   (`r#"["artifact-{run_index}.txt"]"#`).

2. **`rusqlite::Connection` is `Send` but not `Sync`,** so
   `tauri::State<'_, AppState>` did not satisfy `CommandArg` and 23 commands
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
| `npm run tauri -- build --debug` | **PASS with `--bundles app`** — see the blocker below |
| `python scripts/verify_repo.py <repo-root>` | **PASS** — structure and privacy scan |

`bash scripts/verify.sh` runs the whole sweep: **8 passed, 0 failed.**

### The native build and the one blocker

`npm run tauri -- build --debug` compiles, links and bundles the
application:

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

The sweep now runs `npm run tauri -- build --debug --no-bundle`, which
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

#### 13. After a restart, the first approval decision failed — FIXED

This is the most serious defect found in the release.

The identifier counter (`src-tauri/src/domain/ids.rs`) is a process-local
`AtomicU64` that starts at one in every new process. The database, however,
still holds the identifiers the previous process minted. The counter was
never advanced past what was stored — `observe_counter` existed for exactly
that purpose and was **never called** — so the new process was handed an
identifier that was already in use.

The identifiers the counter mints are `audit-{n:06}` and `decision-{n:06}`,
and both are primary keys. The simulator's own event identifiers are
derived from the step counter (`sim-event-{step}-{n}`) and are unaffected.

Reproduced live, before the fix:

```
the first decision after a restart failed: Some(Database(SqliteFailure(
  Error { code: ConstraintViolation, extended_code: 1555 },
  Some("UNIQUE constraint failed: approval_decisions.id"))))
```

and the same for `audit_events.id`.

The application is restarted far more often than it is reset, so this was
reachable in ordinary use: an operator who approved something, quit the
application and opened it again could not approve or reject anything
afterwards.

`Database::open` and `Database::open_memory` now advance the counter past
the highest identifier already stored. The regression test is an
**integration** test — `src-tauri/tests/restart_ids.rs` — because it opens
a database on disk and resets the process-local counter, which must not
race the unit tests that share theirs.

The test was verified to catch the defect: removing the two
`observe_stored_ids()` call sites makes both integration tests fail with
the same constraint violations, and the restore was checked byte for byte
with SHA-256.

#### 14. The sweep assumed the caller had set the PATH — FIXED

`cargo tauri` looks for a binary named `cargo-tauri`, which lives in
`node_modules/.bin`. The sweep created the bridge there but never put
the directory on the PATH. On a CI runner — where `node_modules/.bin` is
not on the PATH — the native build failed in twenty milliseconds with
*"no such command: `tauri`"*. The local runs had not caught it because
the shell wrapper had exported the PATH.

The sweep now exports the PATH itself. Verified by running the whole
sweep with `node_modules/.bin` removed from the PATH: **12 passed, 0
failed.**

#### 15. The application had no palette on its first paint — FIXED

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

#### 16. The Timezone setting did nothing — FIXED

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

#### 17. The appearance function read the store behind the caller's back — FIXED

`applyTheme(value)` took the theme as a parameter but read the density
from the global store. It worked because both call sites set the store
first, and it would have broken the first time anyone called it
differently. It is now `applyAppearance(settings)`, with one source of
truth.

#### 18. A store was written but never read — FIXED

`settingsLoaded` was exported, set to true when the settings arrived,
and read by nothing. Removed.

#### 19. The privacy scanner missed key names with an underscore — FIXED

The key alternation ended in a word boundary, and `_` is a word character,
so `SECRET_KEY`, `DB_PASSWORD`, `PASSWORD_FILE` and `secret_key` were all
passed. Verified live against the scanner before the fix: all four missed.

The key may now carry a prefix or a suffix. The separators are explicit,
so a word that merely starts with one of the names — `tokenizer` — is not
treated as a key. Six cases were added to the scanner's own regression
test, which now has twenty-eight.

#### 20. The simulator was changing the seeded data — FIXED

The simulator advanced, failed and completed the **seeded** runs, and
attached its approvals to seeded runs that already carried one.

Reproduced live with a probe that ticked the simulator and printed the
runs and the approvals after every step. The first probe showed the
simulator failing seeded `run-000007` — a run with a pending approval on
it — and then completing another seeded run. The seeded approvals were
left pending on a run that had already failed.

`active_run` now considers only the simulator's own runs, whose identifiers
begin `sim-run-`.

#### 21. The simulator's six phases belonged to six different agents — FIXED

The phase and the agent were both derived from the step. The phase cycle
is documented as create, advance, advance, approval, fail, complete, and
those six phases belong to one run — but the run was created for one agent
and the next phase looked for a run belonging to a different one. The
probe showed steps two, three, seven and eight all doing nothing.

The agent is now chosen by the cycle rather than by the step, and the
phases are counted from the first tick, so the cycle begins with the
create. The probe now shows created, advanced, advanced, approval.

#### 22. The simulator failed runs that were waiting on a decision — FIXED

The fail phase failed the run a tick after the approval phase raised an
approval on it, so the approval was left pending on a run that had already
failed. A run cannot be failed behind the operator's back, so it now waits
until the decision is made, and the complete phase leaves it alone too.

#### 23. A rejected run was left looking as though it were still running — FIXED

The reject path set the run's status but not its end time or its
duration, unlike the simulator's own failure path. The run detail view
reads "still running" when the end time is missing, so a rejected run
said so.

#### 24. A decided approval left the agent waiting — FIXED

The decision path updated the approval, the decision record, the audit
record and the run, but never the agent — so an agent stayed in the
`approval` state for good. An approved run now continues and its agent
goes back to work; a rejected run stops and its agent stops with it.

The decision also no longer resurrects a run that has already finished:
the decision is recorded whatever the run has become, because it is a fact
about what the operator did, but a run that has already failed is not moved
back to running.

#### 25. A tick that failed part way through left the simulator wedged — FIXED

An independent audit of the Rust backend found this, and proved it by
injecting a conflicting `run_events` primary key. The tick returned an
error, yet one run and four stages were already committed while the step
counter stayed at five. Every retry hit the same primary key, so the
simulator was permanently stuck.

`tick` called `apply_step` and then wrote the step counter as two
separate auto-commit units, and `apply_step` performs many independent
statements. The step and everything it writes are now one transaction:
if any part of the step fails, nothing is committed and the step is not
consumed, so the next tick tries the same step from the same state.

#### 26. Enabling a policy that does not exist reported success — FIXED

The update matched no rows and the command returned `Ok(())`, so the
frontend showed a successful toggle for a policy that is not there. The
number of affected rows is now checked, and a policy that does not exist
is reported as not found.

#### 27. The wildcards in a search were matched as wildcards — FIXED

The search term was interpolated into a `LIKE` pattern unescaped, so
searching for `%` or `_` matched every run. The value was bound as a
parameter, so this was never an injection — the search simply did not
match what was asked for. The metacharacters are now escaped and the
pattern declares its escape character.

#### 28. A reset was not reproducible — FIXED

The seed wrote the wall clock into `seeded_at`, so every re-seed stored a
different value, while the seed's own documentation claimed the data was
anchored to a fixed instant so that "reset demo data" reproduces
byte-for-byte the same state. The seed now records the anchor it was
given.

#### 29. A database fault was reported as a missing approval — FIXED

The approval state query discarded its error, so any database problem
became `None`, which became "record not found". The operator was sent
looking for a missing approval when the real cause was a database fault.
Only a missing row is now reported as missing; anything else stays a
database error.

#### 30. An identifier was measured in bytes while its message said characters — FIXED

The length check counted bytes and the message said characters. The
validator only accepts ASCII, so the difference never changed whether an
input was accepted — it changed only *which* refusal explained why. A
hundred accented letters is a hundred characters, not eight hundred
bytes, so the length rule must not be the one that speaks. The check now
counts characters.

#### 31. The documentation claimed twenty-four commands — FIXED

There are twenty-three `#[tauri::command]` attributes, twenty-three
entries in the handler, and twenty-three calls in the frontend — an exact
three-way match. Four places in the documentation said twenty-four.

#### 32. The icon set could not be regenerated from its documented source — FIXED

The `.gitignore` says every icon is regenerated from `icons/source.png`,
which is the committed source of truth. But the tree carried a renamed
double-resolution icon, so running the icon generator again produced a
different tree and reintroduced a file the scanner flagged as an email
address.

The conventional name is back, the configuration points at it, and the
privacy scanner now excludes the image extensions instead of treating any
file name containing an `@` and a three-letter extension as an address.
Verified: the icon file name is clean, a real address is still caught,
and the scanner's own twenty-eight regression cases still pass.

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

## Phase 8 — The v0.1.2 maintenance release

A maintenance, correctness, portability, verification and documentation
release. No screen changed, no command was added or removed, and the
database schema did not change: a v0.1 database opens as a v0.1.2
database.

### The nine items an independent review raised

Each was checked against the code before anything was changed. Two were
already correct in the repository and are recorded as such.

1. **The build and CI configuration still referenced Node.js 20.**
   CONFIRMED and FIXED. `.github/workflows/ci.yml` asked for
   `node-version: "20"` in two places. The baseline is now Node.js 24,
   and `.node-version` carries `24`.

2. **The packaging documentation still permitted Node.js 20.**
   CONFIRMED and FIXED. `docs/PACKAGING.md` said "Node 20 or newer"; it
   now says Node.js 24.

3. **The test counts in the engineering documentation were stale.**
   CONFIRMED and FIXED. `BUILD_STATE.md` said thirty-six frontend tests
   and thirty-three Rust tests. The measured numbers are in the table
   below.

4. **The scanner test counts in the changelog and the documentation were
   stale.** CONFIRMED and FIXED. The scanner's own regression test now
   runs fifty cases, up from twenty-eight.

5. **The empty-source-file rule did not cover the whole source tree.**
   CONFIRMED and FIXED. The rule looked at `src` and `src-tauri/src`
   and missed `src-tauri/tests` and `scripts`, so an empty integration
   test and an empty sweep script were both invisible. An empty sweep
   script is the worse of the two: it exits with a success and reports
   nothing, which is indistinguishable from a sweep that found no
   problems. The rule now covers all four directories, uses the
   scanner's own walk so the generated directories, the symbolic links
   and the unreadable files are handled the way the scan handles them,
   and reports a file that holds nothing but whitespace. Nine mutations
   of the rule are caught by the regression test, which now holds
   sixty-two cases.

5. **The privacy scanner had a four megabyte limit whose behaviour and
   documentation needed improving.** CONFIRMED and FIXED. An oversized
   file used to disappear from verification without a word. It is now
   reported, and the verification fails until a person has looked at it.

6. **Windows was not in the continuous integration matrix.** CONFIRMED
   and FIXED. The matrix now covers Linux, macOS and Windows.

7. **The approval error handling had no regression test proving that a
   real SQLite failure is not turned into a not-found error.** CONFIRMED
   and FIXED. `src-tauri/tests/approval_errors.rs` now proves it.

8. **There was no automated packaged desktop or end-to-end coverage.**
   CONFIRMED, and addressed as far as it honestly can be. The frontend
   command boundary now has integration tests that drive the real
   components. Automated packaged desktop testing is still not
   implemented, and the limitation is kept.

9. **Some release and build documentation described results that were
   not derived from the verification scripts.** CONFIRMED and FIXED. The
   counts in the documentation are now measured by the sweep, and the
   sweep prints what it measured.

### Two items that were already correct

- The capability set was already `core:default` alone, with no HTTP,
  shell, filesystem, process, clipboard or updater permission.
- The native build check was already in the sweep. It was kept, and its
  meaning is now stated precisely: it proves the application compiles
  and links. It does not prove that a bundle, an installer, a signature
  or a notarisation exists.

### What changed in the code

- **The privacy scanner now decides what happened to every file.**
  Six verdicts: scanned, binary skipped, generated excluded, oversize
  review required, unreadable, and a violation. An oversized file that
  could hold source, a script or a configuration is reported and the
  verification fails. A known dependency lockfile is not exempt: it is
  searched a line at a time, with the same patterns as every other file.
  The value that matched is never printed.
- **The scanner reports six counters** at the end of every run.
- **The approval lookup is now proved.** A missing approval is reported
  as not found; a broken lookup is reported as a database error. The
  failure is manufactured with a column removed from the test's own
  database, so the production code carries no test hook.
- **A policy update that fails now says so.** The command's error was
  swallowed, so a policy that was not there left the operator looking at
  a switch that had not moved, with no explanation.
- **The application no longer panics when the window cannot open.** It
  prints the reason and leaves with a failure code.
- **The frontend integration tests exist.** Eighteen tests drive the
  real components against a deterministic mock of the command boundary:
  the shell, the navigation, the overview, the agent and run searches,
  the approval decision and its failure, the policy toggle and its
  not-found failure, the simulator, and the settings contract.

### What the manual smoke test found

The application was launched on this host and driven through its
accessibility tree. The screen itself cannot be captured — there is a
window server but no attached display — so the checks were made against
what the accessibility tree exposes, which is what the frontend renders.
The driver used for it is not part of this repository's toolchain, so
the smoke test itself is not committed here; the checklist it followed is
in `docs/TESTING.md`, and a person can walk it by hand.

Thirty-six checks were made against the running window, and one of them
failed.

- **The confirmation of an approval was never shown.** The confirmation
  was rendered inside the detail drawer, and a decided approval leaves
  the pending list, so the drawer closed and the message went with it.
  The operator approved an approval and was told nothing. The
  confirmation is now rendered outside the drawer, and a regression test
  covers it.

The other thirty-five checks passed: the shell renders with its palette
and its summary, all twelve views open with their own content, the agent
search narrows the registry, the approval queue shows the pending
approvals, the policy toggle is confirmed, the simulator starts and
pauses, the settings form and the demo data panel are there, and the
approval decision reaches the audit log.

### Making the Windows build a real gate

The sweep proved that the application compiles and links on Windows. It
did not prove that the Windows *application* — the `.msi` and the NSIS
installer — can be produced, because the sweep deliberately does not
bundle. The continuous integration had no job that did the bundling
either, so nothing in the build ever touched the WebView2 loader, the
`.ico`, WiX or NSIS.

- **A `windows-app` job** now runs the Tauri bundler on
  `windows-latest` and fails if no installer comes out. It is its own
  job rather than another step in the matrix, so a bundling failure is
  visible on its own.
- **The bridge that makes `cargo tauri ...` work is now its own script**,
  `scripts/ensure_cargo_tauri.py`. The sweep and the new job call it, so
  there is one implementation of it rather than two.
- **The line endings are pinned.** `.gitattributes` says every text file
  is checked out with line feeds on every platform, which is what keeps
  `scripts/verify.sh`'s shebang working on a Windows checkout.

### What was checked here, and what was not

The application was checked against a Windows target on this host, which
is not Windows:

| Check | Result |
|---|---|
| `cargo check --target x86_64-pc-windows-gnu --all-targets` | passes — the whole tree, including the bundled SQLite C code, `webview2-com`, `muda` and `tao` |
| `cargo check --target x86_64-pc-windows-msvc --all-targets` | passes — the target the continuous integration uses, including the Windows resource embedding that carries the icon and the version into the binary |

The MSVC check needed `llvm-rc` for the resource compiler, which is in
the keg-only `llvm@22` Homebrew formula and is therefore not on the path
by default. The bundled SQLite is the one part of the MSVC target that
cannot be checked here, because it needs Microsoft's own C compiler; the
check was made with that feature off, and the manifest and the lockfile
were restored exactly afterwards.

What this proves: the code compiles for Windows. What it does not prove:
that the installer is produced, and that the application runs. Those are
the continuous integration's to report.

**They have now been reported.** The continuous integration on the Windows
runner produced both installers — `AgentOps_0.1.2_x64_en-US.msi`
(4.77 MiB) and `AgentOps_0.1.2_x64-setup.exe` (2.97 MiB) — with the
application built by MSVC, the icon embedded in the executable, and WiX
and NSIS downloaded and verified by the bundler. The sweep on the same
runner reports fourteen passed, zero failed and one skipped; the skip is
the Rust dependency audit, because `cargo-audit` is not installed on the
runner.

So Windows now has: the code compiles for it, the application compiles
and links there, and the installers are produced. It does not yet have:
the application installed and run, which is a different thing and would
need a runner with a display.

### What the first run on a Windows runner found

The first run of the Windows jobs found three real defects, all of them
in the build rather than in the application. None of them could have been
found without a Windows runner.

| What | How it showed up | Cause | Fix |
|---|---|---|---|
| every npm check was skipped | `SKIP: npm ci (npm is not installed)` — on a runner where npm was installed | Python starts a program without a shell, so it does not apply the path extension rules. On Windows npm is `npm.cmd`, and a bare `npm` is not found | the programs are resolved with `shutil.which`, which applies those rules |
| the minimum Rust check failed | `UnicodeDecodeError: 'charmap' codec can't decode byte 0x81` | the `cargo metadata` output was decoded with the platform's encoding, which is a code page that is not UTF-8 on Windows | the output is captured as bytes and decoded as UTF-8 |
| the installer job failed | `error: no such command: 'tauri'` | Cargo looks for an external subcommand as a program and did not find the `.cmd` bridge | the sweep and the job use `npm run tauri -- ...`, which resolves the command line interface itself on every platform |

The first of them is the dangerous kind: a check that reports itself as
skipped is invisible, and four checks and the native build were quietly
not running at all. The sweep was right to report them as skips rather
than as passes — that is what made the difference between a green job and
a job that had done nothing visible.

### The verification system

The checks now live in `scripts/verify.py`, so `scripts/verify.sh` for
macOS and Linux and `scripts/verify.ps1` for Windows run the same
checks in the same order. The two entry points cannot drift apart,
because there is only one list of checks.

A check that cannot run is reported as a skip, with the reason, and is
never counted as a pass.

### The measured results

Measured during the v0.1.2 acceptance run, on Node.js 24.22.3 with npm
11.19.1 and Rust 1.93.1.

| | v0.1.0 | v0.1.1 | v0.1.2 |
|---|---|---|---|
| Rust tests | 33 | 46 | 49 |
| Frontend tests | 36 | 42 | 60 |
| Sweep stages | 8 | 13 | 15 |
| Scanner regression cases | 20 | 28 | 50 |
| Screens | 12 | 12 | 12 |
| Typed commands | 23 | 23 | 23 |

The sweep reports fifteen passed, zero failed and zero skipped.

### The Rust dependency audit, and what it found

`cargo audit` was installed on this host, so the stage that used to be a
skip is now a gate. It denies warnings, not only vulnerabilities: an
unmaintained, an unsound or a yanked crate fails the sweep too.

It found one vulnerability. `RUSTSEC-2026-0459` — the bundled SQLite C
library, through `rusqlite` — carried `CVE-2026-119375`, medium
severity, 6.8. The application compiles SQLite into its binary, so the
defect would have shipped with it. `rusqlite` moved from 0.32 to 0.43
and `libsqlite3-sys` from 0.30.1 to 0.40.0, which is past the 0.39.0
that carries the fix. The application's own use of the library did not
change: the same calls compile, all forty-nine Rust tests pass, the
clippy gate is clean, and the upgrade needs Rust 1.89, which is below
the declared minimum of 1.90.

Three warnings remain, and none of them can be closed by a compatible
upgrade:

| Advisory | Package | Path | Kind | Exercised here |
|---|---|---|---|---|
| RUSTSEC-2026-0543 | chrono | agentops → chrono | unmaintained | Only on the application's own timestamps |
| RUSTSEC-2024-0429 | glib | agentops → tauri → gtk → glib (Linux only) | unsound | No: not in the macOS or Windows build |
| RUSTSEC-2024-0370 | proc-macro-error | agentops → tauri → gtk → glib → glib-macros → proc-macro-error (Linux only) | unmaintained | No: a build-time dependency of the Linux GTK bindings |

Each is listed in `scripts/rust_audit_allowlist.json` with its reason
and the action that would close it. The audit still fails on anything
that is not listed there, and the gate was proved by putting the
vulnerable binding back: it reported the vulnerability and failed.

`cargo-audit` 0.23 does not read an `audit.toml` configuration file, so
the list is a file the sweep reads and passes to the tool as
exclusions. The `audit.toml` that was tried first was removed rather
than left in place doing nothing.

## Known limitations

These are genuinely outside v0.1 and are recorded rather than hidden.

- **No automated packaged desktop test.** The frontend command boundary
  is covered by integration tests that drive the real components, but
  there is no automation that launches the built application and drives
  its window. The functional checks that need a window are listed in
  `docs/TESTING.md` as a manual smoke checklist.
- **No DMG on this host**, for the Finder reason recorded above.
- **Three Rust advisories are accepted rather than closed.** They are
  warnings, not vulnerabilities, and none can be closed by an upgrade
  that is compatible with this release. They are listed in
  `scripts/rust_audit_allowlist.json` with their reason and their
  action, and the audit still denies them if they are not listed there.
- **The visual design has not been reviewed in a running window.** The
  layout, the focus states and the responsive behaviour are implemented and
  type-checked, but this host has no interactive display to look at them.
- **The Windows job has not run here.** The continuous integration
  configuration covers Windows and the PowerShell entry point is in
  place, but a Windows runner is not available on this host, so the
  Windows result is the continuous integration's to report, not this
  document's.
- **The development toolchain has advisories.** Six, all in the test and
  build tools, none of them in the shipped application. They are listed
  in `docs/SECURITY.md`.

## Remaining

Nothing is outstanding. Every acceptance command has been run and passes.

## The v0.1.2 release

The release is cut the same way the v0.1.1 release was: a maintenance
branch is merged into `main` with `--no-ff`, the sweep runs again on the
merge commit, and the tag is made on the merge commit, so the tag and
`main` always name the same tree.

The branch is verified before the merge in three ways:

- the sweep passes locally on Node.js 24;
- the continuous integration passes on the branch head, on Linux, macOS
  and Windows;
- the sweep passes again on the merge commit, before the tag is made.

The continuous integration conclusions are read from the job list and the
test summaries in the logs, never from the badge alone.

### What the release contains

| | v0.1.1 | v0.1.2 |
|---|---|---|
| Rust tests | 46 | 49 |
| Frontend tests | 42 | 60 |
| Sweep stages | 13 | 15 |
| Scanner regression cases | 28 | 50 |
| Screens | 12 | 12 |
| Typed commands | 23 | 23 |

No screen changed, no command was added or removed, and the database
schema did not change: a v0.1 database opens as a v0.1.2 database.

## The v0.1.1 release

The release is cut from the `fix/v0.1.1-hardening` branch, which is
merged into `main` with `--no-ff` so the release boundary is visible in
the history, then tagged `v0.1.1` and published as a GitHub release whose
notes are the `0.1.1` section of `CHANGELOG.md`.

The tag is made on the merge commit, not on the branch head, so the tag
and `main` always name the same tree.

The branch was verified before the merge in three ways:

- the sweep passes thirteen of thirteen, locally;
- the continuous integration passes all three jobs — the sweep on
  `ubuntu-latest`, the sweep on `macos-latest`, and the minimum supported
  Rust job — on the branch head;
- the sweep passes again on the merge commit on `main`, before the tag is
  made.

The continuous integration conclusions are read from the job list and the
test summaries in the logs, never from the badge alone.

### What the release contains

| | v0.1.0 | v0.1.1 |
|---|---|---|
| Rust tests | 33 | 41 |
| Frontend tests | 36 | 42 |
| Sweep checks | 8 | 13 |
| Scanner regression cases | 20 | 28 |
| Screens | 12 | 12 |
| Typed commands | 23 | 23 |

No screen changed, no command was added or removed, and the database
schema did not change: a v0.1 database opens as a v0.1.1 database.

## Phase 8 — The v0.1.3 supply-chain security release

A maintenance release. No feature is added, no screen changes, no
command is added or removed, and the database schema does not change: a
v0.1 database opens as a v0.1.3 database, and no persisted timestamp
format changes.

The purpose was to turn the dependency audits from advisory tooling into
a deterministic, enforceable release-security gate.

### The baseline, recorded before anything was changed

| | |
|---|---|
| The commit | `502ec4d02f9669d45ffba54b075288a0d2703fd0` |
| The branch | `main` |
| The application version | 0.1.2 |
| Node | v26.13.1 installed; v24.22.3 at the declared baseline |
| npm | 11.20.0 installed; 11.19.1 at the declared baseline |
| Rust | 1.93.1 |
| Cargo | 1.93.1 |
| Python | 3.9.6 |
| The operating system family | Darwin |
| The CPU architecture | arm64 |
| The working tree | clean |

The sweep was run before the first edit and reported **fifteen passed,
zero failed and zero skipped**. The six npm advisories were reported as
a note, and the Rust audit was a gate that a missing tool could turn
into a skip. Those two facts are what this phase changed.

### The JavaScript dependency audit

The actual state, measured rather than assumed: `npm audit` reported
**six vulnerabilities — two critical, one high, three moderate**. The
prompt's assumed versions were partly wrong: the Vitest chain was
2.1.9 with tinypool 1.1.1, and the esbuild that was flagged was the
nested copy under `vite-node`, not the direct one, which was already
0.25.12.

| Package | Severity | Advisories | Path |
|---|---|---|---|
| tinypool 1.1.1 | critical | GHSA-5gmw-xhrv-c9v3, GHSA-85c8-ppgw-ccpr | vitest → tinypool |
| vitest 2.1.9 | critical | GHSA-5xrq-8626-4rwp, GHSA-82fw-gwwq-j7x9 | vitest |
| vite | high | eight advisories | vitest → vite |
| @vitest/mocker | moderate | GHSA-82fw-gwwq-j7x9, GHSA-qxg6-89mg-rqv8 | vitest → @vitest/mocker |
| esbuild | moderate | GHSA-67mh-4wv8-2f99 | vitest → vite-node → esbuild |
| vite-node | moderate | depends on vite | vitest → vite-node |

Every one of them is in the development and test toolchain. None of them
ships, and the running application exercises none of them — but
developers and the continuous integration execute all of them, so they
were gated rather than reported.

The fix was made deliberately, with
`npm install --save-dev vitest@^5.1.1`, and not with
`npm audit fix --force`. Vitest 5.1.1 accepts `vite ^6.4.0 || ^7.0.0 ||
^8.0.0`, so the project's Vite did not have to move from 6.4.6, and its
`engines` field accepts Node 24, so the Node baseline did not change.
One direct dependency changed. `tinypool` and `vite-node` left the tree
entirely, and the lockfile lost about eleven hundred lines.

After the change `npm audit` reports **no advisories at all** — not none
that were accepted, none. The whole frontend suite was run after the
change: sixty tests pass, `svelte-check` reports zero errors and zero
warnings, and the production bundle assembles.

### The two gates

`scripts/npm_audit_gate.py` and `scripts/rust_audit_gate.py` apply the
policy themselves rather than leaving it to a tool's exit code, so that
the rules can be read in one place and tested without a registry or an
advisory database. `scripts/audit_policy.py` holds the schema and the
expiry rule that both of them share.

The npm policy fails on Critical and High, warns on Moderate, and
reports Low and informational. The Rust policy does not distinguish a
vulnerability from an unmaintained, an unsound or a yanked crate when
deciding what to do: all of them fail unless the advisory is listed.

The gates exit with 0 when they pass, 1 when they fail and **2 when they
could not run**, so that the sweep can tell a check that passed from a
check that did not happen.

Every rule was proved to bite, against the real tools rather than only
against the policy functions:

| What was done | What happened |
|---|---|
| a fake `npm` returning a Critical advisory | the npm gate exited 1 |
| a fake `npm` that could not reach the registry | the npm gate exited 1 in release mode and 0 in developer mode |
| `cargo-audit` removed from the path | the Rust gate exited 1 in release mode and 0 in developer mode |
| one advisory removed from the allowlist | the Rust gate exited 1 |
| an exception's review date set in the past | the Rust gate exited 1 in release mode and warned in developer mode |

### The release mode

`scripts/verify.py --release` runs the same sweep with one rule changed:
a check that cannot run is a failure rather than a skip. The developer
mode is unchanged and stays convenient offline. `scripts/verify.sh` and
`scripts/verify.ps1` pass the flag through.

### The Rust advisories that are accepted

All three were re-verified: the advisory still exists, the crate is
still in the lockfile, the dependency paths were re-derived with
`cargo tree --target all -i`, the reason still applies and the
remediation still stands.

| Advisory | Crate | Scope | Review by |
|---|---|---|---|
| RUSTSEC-2026-0543 | chrono 0.4.45 | runtime | 2027-02-23 |
| RUSTSEC-2024-0429 | glib 0.18.5 | platform-specific (Linux only) | 2027-01-24 |
| RUSTSEC-2024-0370 | proc-macro-error 1.0.4 | build-time (Linux only) | 2027-02-23 |

The glib advisory has a patched version — 0.20.0 — but the Tauri 2 line
does not move to it; a dry-run dependency upgrade confirmed that the GTK
versions do not change inside that line.

The chrono migration was assessed and deferred. The use was inventoried:
`src-tauri/src/util.rs` (four helpers that format, parse and subtract
RFC 3339 strings), `src-tauri/src/simulator/mod.rs` (the current instant)
and `src-tauri/src/seed.rs` (whole-minute shifts on the seed's own
timestamps). The stored representation is an RFC 3339 string with whole
seconds and a `Z` suffix in a `TEXT` column, and the frontend parses
those strings with the platform's date parser. The migration to `time`
or `jiff` is small — three files — but it changes the timestamp library
in the same release as the test-toolchain upgrade, which widens the
blast radius of the release for an advisory that carries no exploit. It
is planned for the next maintenance release, with a comparison test
that asserts the new formatter's output is byte-identical to the old
one.

### The continuous integration

Two jobs were added.

- **The security gate.** The two gates in release mode, the two
  allowlist validators, the privacy scanner's regression test and the
  repository privacy scan. It is independent of the compile and
  unit-test jobs, and it fails if the security tooling cannot initialise.
  It needs only `contents: read`.
- **The release verification.** The whole sweep in release mode, so that
  the release evidence is machine-verifiable rather than a claim in a
  document.

`cargo-audit` is installed in both at the pinned version **0.23.0**, with
`cargo install cargo-audit --version 0.23.0 --locked`, and the installed
version is printed with `cargo audit --version`. A runner's own copy is
never trusted, and `cargo install cargo-audit` without a version is not
used. The repository pins no cache action, so each job that needs the
tool installs it rather than sharing a cache.

### The runtime is still offline

Re-checked for this release: no frontend networking API, no Rust
networking crate, no HTTP capability, no updater configuration, no
telemetry, no analytics, no crash upload, no remote asset. The Tauri
capability set is still `core:default`, and the Tauri dependency still
carries no plugin. The privacy scanner scanned ninety-nine files and
found no violation.

The continuous integration does download advisory databases. That is a
build-time action by the build, and it is not the application reaching
the network.

### The continuous integration, and what it actually showed

The workflow now has five jobs, and the run on the head of `main` after
the v0.1.3 merge was executed, not merely configured:

| Job | Result | What it showed |
|---|---|---|
| the release verification | PASS | `RELEASE VERIFICATION: every check has to run, and every check has to pass` · **passed: 16  failed: 0  skipped: 0** · 49 Rust tests |
| the security gate | PASS | the npm gate passed · the Rust audit found 0 vulnerabilities and 3 informational advisories, all accepted · 44 policy cases · 50 scanner cases · 0 privacy violations |
| the sweep (ubuntu-latest) | PASS | 15 passed, 0 failed, 1 skipped · 49 Rust tests |
| the sweep (macos-latest) | PASS | 15 passed, 0 failed, 1 skipped · 49 Rust tests |
| the sweep (windows-latest) | PASS | 15 passed, 0 failed, 1 skipped · 49 Rust tests |
| the Windows application | PASS | `AgentOps_0.1.3_x64_en-US.msi` (4.77 MiB) and `AgentOps_0.1.3_x64-setup.exe` (2.97 MiB) |
| the minimum supported Rust | PASS | it compiles on Rust 1.90 |

The one skip in each of the three matrix legs is the Rust dependency
audit, and it says so: `cargo-audit is not installed, so the Rust
advisories were not scanned`. That is the developer mode behaving as
designed. The audit is covered by the two jobs that install the pinned
tool, and the release verification job has nothing skipped at all.

The tag pushes each produced their own run, and both were green, so the
two tagged commits are verified too.

### The packaged desktop smoke test

The gap is documented precisely in `docs/TESTING.md`: against the ten
things a packaged smoke test has to demonstrate, three are proven by the
automated suite, four are proven against a mock of the command boundary
rather than against the real bridge, and three are covered only by a
person looking at the application. The options were evaluated and the
harness is recorded as a v0.1.x follow-up; the reason is that a native
webview needs a different driver on each platform and macOS has no
supported one, so a harness that only works on one platform is not a
release gate. The manual checklist stays.

### What the release contains

| | v0.1.1 | v0.1.2 | v0.1.3 |
|---|---|---|---|
| Rust tests | 41 | 49 | 49 |
| Frontend tests | 42 | 60 | 60 |
| Sweep stages | 13 | 15 | 16 |
| Scanner regression cases | 28 | 50 | 62 |
| Audit policy regression cases | — | — | 44 |
| Screens | 12 | 12 | 12 |
| Typed commands | 23 | 23 | 23 |
| Accepted Rust advisories | 0 | 3 | 3, all with review dates |
| Unresolved npm advisories | — | 6 | 0 |
