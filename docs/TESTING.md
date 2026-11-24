# Testing

## The versions that were tested

| | |
|---|---|
| Node.js | 24.22.3 |
| npm | 11.19.1 |
| Rust | 1.93.1 |
| The minimum supported Rust | 1.90 |

Node.js 24 is the baseline. The Rust minimum is the version declared in
`src-tauri/Cargo.toml`, and the sweep checks that the claim is true
against the whole resolved dependency graph.

## What is tested

The counts below are what the sweep measured during the v0.1.2 acceptance
run. They are not a design claim: run the sweep and read its own output.

**Rust — 49 tests.** Thirty-three unit tests inside the library, and
sixteen integration tests in four files. The unit tests cover the database
layer (migrations apply once and are recorded; foreign keys are enforced),
the domain helpers (a valid timestamp round-trips; a malformed one is
`None` rather than a panic; a negative duration clamps to zero), every
command module (the registry lists every seeded agent in a stable order; a
malformed identifier is rejected before it reaches SQL; an unknown status
is rejected; runs are listed newest first; a status filter narrows the
result; a search matches both the task and the agent name), the approval
queue (a decision records both the decision and the state change; deciding
twice is refused; an unknown outcome is rejected), the settings (they
round-trip; an unknown theme and a malformed timezone are rejected), the
audit log (a recorded event appears; a malformed JSON array reads as empty
rather than as an error), and the simulator (a paused simulator does
nothing; the same step sequence produces the same result; the speed is
clamped; a non-finite speed is rejected; a reset restores the original
seeded state).

The integration tests are in `src-tauri/tests/`:

- `approval_errors.rs` — a missing approval is reported as a missing
  approval, and a broken lookup is reported as a database error. The
  failure is manufactured by removing a column from the test's own
  database, so the production code carries no test hook.
- `audit_fixes.rs` — the defects an independent audit found.
- `restart_ids.rs` — the identifier counter after a restart.
- `simulator_cycle.rs` — the simulator's cycle.

**Frontend — 60 tests.** Forty-two unit tests and eighteen integration
tests.

The unit tests cover the formatting helpers (a valid timestamp formats; an
invalid one shows a placeholder rather than `NaN`; relative times at every
scale; durations; percentages; status labels; initials), the resource
pattern (idle, loaded, empty and failed transitions, including a
non-`Error` rejection), the navigation model (every view is unique and
labelled), and the command bridge (the command name and arguments are
passed through; an omitted filter is sent as `null`; a structured error
becomes a `CommandError` carrying its code; a transport failure and an
unknown rejection both become a `CommandError`).

The integration tests are in `src/tests/integration.test.ts`. They drive
the **real components** against a deterministic mock of the Tauri command
boundary: every command is answered by a fixture, so a test states exactly
what the backend returned and what the operator would see. They cover the
application shell rendering, the navigation between views, the Night Shift
summary, the agent search and status filter, the run search, the approval
decision and its failure, the policy toggle and its not-found failure, the
simulator starting, pausing, advancing on its own interval and resetting,
the rendering of a structured backend error, and the settings persistence
contract.

## The commands

```bash
npm ci
npm run check
npm test -- --run
npm run build

cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features

python3 scripts/check_version.py .
python3 scripts/check_msrv.py .
python3 scripts/test_verify_repo.py
python3 scripts/check_types.py .
python3 scripts/verify_repo.py .
npm run tauri -- build --debug --no-bundle
npm audit --omit=dev

bash scripts/verify.sh          # macOS and Linux
pwsh ./scripts/verify.ps1       # Windows
```

`npm run check` must report **0 errors and 0 warnings**: `svelte-check`
covers both the TypeScript and the Svelte template diagnostics.

`cargo clippy` runs with `-D warnings`, so a single warning fails the build.

### The sweep

The checks live in `scripts/verify.py`. `scripts/verify.sh` runs them on
macOS and Linux and `scripts/verify.ps1` runs them on Windows, so the two
platforms cannot drift apart: there is only one list of checks. Every
stage prints one `PASS:`, `FAIL:` or `SKIP:` line.

A stage that cannot run is reported as a **skip**, with the reason, and is
never counted as a pass.

| Stage | Proves |
|---|---|
| `npm ci` | The lockfile is sufficient on its own |
| `npm run check` | The type and template diagnostics are clean |
| `npm test` | The frontend tests pass |
| `npm run build` | The production bundle assembles |
| `cargo fmt --check` | The Rust is formatted |
| `cargo clippy` | The Rust is idiomatic; one warning fails the build |
| `cargo test` | The Rust tests pass |
| the version check | Every manifest agrees on the version |
| the minimum supported Rust check | The declared minimum compiler is true |
| the scanner's regression test | The privacy scanner still catches what it must |
| the type vocabulary check | The Rust enums and the TypeScript unions agree |
| the native build | The application compiles and links as an application |
| the production dependency audit | The shipped dependencies have no advisory |
| the development dependency audit | Reported, with the advisories listed in `docs/SECURITY.md` |
| the Rust dependency audit | The Rust tree has no vulnerability, and no unmaintained, unsound or yanked crate that is not accepted in `scripts/rust_audit_allowlist.json` |
| the privacy verifier | The required paths exist and nothing is embedded |

During the v0.1.2 acceptance run the sweep reported **fifteen passed,
zero failed and zero skipped**.

The Rust dependency audit denies warnings, so it fails on an unmaintained
or an unsound crate as well as on a vulnerability. Three advisories are
accepted and listed in `scripts/rust_audit_allowlist.json`, each with
its reason and the action that would close it; they are described in
`docs/SECURITY.md`. When `cargo-audit` is not installed the stage is
reported as a skip with that reason, never as a pass.

## The privacy check

`scripts/verify_repo.py` asserts the required paths exist and scans the
tree for email addresses, private IPv4 ranges, bearer tokens and secret
assignments.

The scan is **extension-blind**: it reads every file and scans it if it
decodes as UTF-8 and holds no NUL byte. An earlier version scanned only a
list of known text suffixes, and that was a real false negative — a secret
in a `.sh`, `.env`, `.lock` or `.py` file was not detected. It also
required a quoted value and a bare key, so the usual unquoted `.env` form
and the usual quoted JSON form both passed.

`scripts/test_verify_repo.py` is the scanner's own regression test: fifty
cases, each building a throwaway tree and asserting what the scanner does
with it. It runs with a plain `python3` and needs no third-party package.

The scanner decides one of six things about every file — scanned, binary
skipped, generated excluded, oversize review required, unreadable, or a
violation — and prints the six counters at the end. An oversized file is
reported and the verification fails until someone has looked at it; a
known dependency lockfile is not exempt, but is read a line at a time
instead. The value that matched is never printed. `docs/SECURITY.md`
describes what the scanner is and is not.

## The type vocabulary check

`scripts/check_types.py` compares the Rust domain enums with the
TypeScript string unions that mirror them — `AgentStatus`, `RunStatus`,
`ApprovalState` and `Severity`. A value added on one side and forgotten
on the other is not a compile error: the backend would send a value the
frontend's union does not contain, and the frontend would treat it as an
unknown member. The check reads both sides and compares them, and the
sweep runs it.

## The manual desktop smoke checklist

These need a running application, so they are not part of the automated
sweep. The checklist is precise so it can be handed to somebody else and
followed without asking questions.

**The shell**

- [ ] The application launches, and the Night Shift view renders with its
      palette: dark background, readable text, no unstyled frame.
- [ ] The sidebar shows every navigation item, and the Crew list shows the
      agents.
- [ ] The simulator bar reads "paused" and "step 0" before anything is
      started.

**Every view**

- [ ] Each of the twelve navigation items opens a real view with content,
      not a blank area: Night Shift, Agents, Runs, Waiting on You,
      Policies, Access, Compute, Audit Log, Tools, Team, Settings, Crew.

**Searching and filtering**

- [ ] Typing in the agent search narrows the registry to the matching
      agents, and clearing it restores all of them.
- [ ] The agent status filter narrows the registry, and says so when
      nothing matches.
- [ ] Typing in the run search narrows the run registry.

**Decisions**

- [ ] An approval can be approved, and the view says the decision was
      recorded and that it is in the audit log.
- [ ] An approval can be rejected, and the run it belongs to stops.
- [ ] When a decision fails, the failure is shown with the backend's
      message, and nothing appears to have changed.

**Policies**

- [ ] A policy switch toggles, and the change survives a reload.
- [ ] When a policy does not exist, the failure is shown; the switch does
      not silently do nothing.

**The simulator**

- [ ] Start begins the simulation and the bar reads "running".
- [ ] Pause stops it and the step counter holds.
- [ ] The simulation advances through its phases on its own.
- [ ] Reset restores the original seeded state exactly.

**Everything else**

- [ ] The audit log records both the simulator's actions and the approval
      decisions.
- [ ] The theme, the density, the simulator speed and the timezone are
      saved and are still there after a restart.
- [ ] No Rust panic is shown, and no unexpected error appears in the
      developer console.
- [ ] The running application makes no outbound connection. Watch the
      network with the machine offline and with the interface up.
- [ ] The layout works at 320, 768, 1024 and 1440 CSS pixels with no
      page-level horizontal scrolling.
- [ ] Keyboard navigation reaches every control and the focus is always
      visible.

## Not tested

**Automated packaged desktop end-to-end testing is not yet implemented.**
No automation launches the built application and drives its window.

What exists is the integration layer described above: the real components
are driven against a deterministic mock of the command boundary, which
covers what the operator sees and what the frontend sends. What it does
not cover is the native window, the webview, the real IPC bridge and the
application's own start-up.

Native webview automation would need a test harness that is reliable
across three platforms and a webview version. Rather than add one that
fails intermittently and is then trusted, the limitation is kept and the
manual checklist above is the substitute.
