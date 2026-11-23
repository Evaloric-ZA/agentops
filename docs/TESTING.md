# Testing

## What is tested

**Rust — 33 unit tests.** The database layer (migrations apply once and are
recorded; foreign keys are enforced), the domain helpers (a valid timestamp
round-trips; a malformed one is `None` rather than a panic; a negative
duration clamps to zero), every command module (the registry lists every
seeded agent in a stable order; a malformed identifier is rejected before
it reaches SQL; an unknown status is rejected; runs are listed newest first;
a status filter narrows the result; a search matches both the task and the
agent name), the approval queue (a decision records both the decision and
the state change; deciding twice is refused; an unknown outcome is rejected),
the settings (they round-trip; an unknown theme and a malformed timezone are
rejected), the audit log (a recorded event appears; a malformed JSON array
reads as empty rather than as an error), and the simulator (a paused
simulator does nothing; the same step sequence produces the same result; the
speed is clamped; a non-finite speed is rejected; a reset restores the
original seeded state).

**Frontend — 36 unit tests.** The formatting helpers (a valid timestamp
formats; an invalid one shows a placeholder rather than `NaN`; relative
times at every scale; durations; percentages; status labels; initials), the
resource pattern (idle, loaded, empty and failed transitions, including a
non-`Error` rejection), the navigation model (every view is unique and
labelled), and the command bridge (the command name and arguments are passed
through; an omitted filter is sent as `null`; a structured error becomes a
`CommandError` carrying its code; a transport failure and an unknown
rejection both become a `CommandError`).

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
python3 scripts/test_verify_repo.py
cargo tauri build --debug --no-bundle

bash scripts/verify.sh
```

`npm run check` must report **0 errors and 0 warnings**: `svelte-check`
covers both the TypeScript and the Svelte template diagnostics.

`cargo clippy` runs with `-D warnings`, so a single warning fails the build.

`scripts/verify.sh` runs all of the above and prints one line per check.
There are eleven:

| Check | Proves |
|---|---|
| `npm ci` | The lockfile is sufficient on its own |
| `npm run check` | The type and template diagnostics are clean |
| `npm test` | The frontend unit tests pass |
| `npm run build` | The production bundle assembles |
| `cargo fmt --check` | The Rust is formatted |
| `cargo clippy` | The Rust is idiomatic; one warning fails the build |
| `cargo test` | The Rust unit tests pass |
| the version check | The four manifests agree on the version |
| the scanner's regression test | The privacy scanner still catches what it must |
| the native build | The application compiles and links as an application |
| the privacy verifier | The required paths exist and nothing is embedded |

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

`scripts/test_verify_repo.py` is the scanner's own regression test: twenty
cases, each building a throwaway tree and asserting what the scanner does
with it. It runs with a plain `python3` and needs no third-party package.

## Manual checks

These need a running application, so they are not part of the automated
sweep. They are listed so they are not forgotten:

- the application launches with the network disconnected
- the seed data appears on first launch
- every navigation item opens a real view
- search and filters work in the agent and run registries
- selecting an agent updates the detail drawer
- a simulated run advances through its stages
- pause and resume work
- an approval can be approved or rejected, and the decision survives a
  restart
- the audit log records both the simulator and the approval actions
- the theme and the settings survive a restart
- reset restores the deterministic demo state
- the empty, offline, failed and error states are visibly reachable
- the layout works at 320, 768, 1024 and 1440 CSS pixels with no
  page-level horizontal scrolling
- keyboard navigation reaches every control and the focus is always visible

## Not tested

Playwright is not used: no end-to-end browser run is configured for this
environment, and a test that cannot run reliably is worse than no test.
Those checks are listed above as manual steps instead.
