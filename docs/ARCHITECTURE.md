# Architecture

## Stack

| Layer | Choice | Why |
|---|---|---|
| Shell | Tauri 2 | A small native surface with a webview frontend; the Rust side owns every privileged operation |
| Core | Rust (stable, edition 2021) | Validation, persistence and simulation all happen in one place, behind a typed boundary |
| Frontend | Svelte 5 + TypeScript + Vite | Small runtime, component-scoped CSS, real type checking |
| Persistence | SQLite via `rusqlite` (bundled) | A single local file, no service to run, foreign keys and transactions |
| Tests | Vitest (frontend) and `cargo test` (Rust) | Both run without a display server |

## Repository shape

```text
agentops/
├── BUILD_STATE.md
├── README.md
├── LICENSE
├── package.json
├── package-lock.json
├── .node-version                the Node.js baseline: 24
├── .gitattributes               the line endings are the same on every platform
├── svelte.config.js
├── tsconfig.json
├── tsconfig.node.json
├── vite.config.ts
├── index.html
├── icons/
├── src/
│   ├── app/App.svelte          the shell: navigation, settings, simulator driver
│   ├── components/             StatusPill, SummaryCard, StateBlock, Panel, Button,
│   │                           Sidebar, Topbar
│   ├── domain/types.ts         the TypeScript mirror of the Rust domain types
│   ├── lib/tauri.ts            the typed command bridge
│   ├── lib/format.ts           pure formatting helpers
│   ├── routes/                 the twelve views
│   ├── stores/app.ts           navigation, theme, settings
│   ├── stores/data.ts          the resource status pattern
│   ├── styles/app.css          CSS variables, light and dark themes
│   └── tests/                  the frontend tests: the unit tests and the
│                               integration tests at the command boundary
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── migrations/             (the schema lives in db/migrations.rs)
│   ├── icons/
│   ├── tests/                  the Rust integration tests
│   └── src/
│       ├── main.rs             a thin binary
│       ├── lib.rs              the application builder
│       ├── commands_api.rs     the 23 Tauri commands
│       ├── commands/           the command implementations
│       ├── db/                 the connection, migrations and records
│       ├── domain/             the canonical types
│       ├── error.rs            structured errors with stable codes
│       ├── seed.rs             the synthetic dataset
│       ├── simulator/          the deterministic simulator
│       └── util.rs             UTC time helpers
├── docs/
├── CHANGELOG.md
└── scripts/
    ├── verify.py                the sweep: the checks themselves, one line each
    ├── verify.sh                the sweep entry point for macOS and Linux
    ├── verify.ps1               the sweep entry point for Windows
    ├── ensure_cargo_tauri.py    the bridge that makes `cargo tauri ...` work
    ├── verify_repo.py           the structure and privacy scan
    ├── test_verify_repo.py      the privacy scan's own regression test
    ├── check_version.py         the manifests must agree on the version
    ├── check_msrv.py            the declared minimum Rust must be true
    ├── check_types.py           the Rust enums and the TypeScript unions must agree
    ├── audit_policy.py          the allowlist schema and the expiry rule
    ├── npm_audit_gate.py        the JavaScript dependency gate
    ├── rust_audit_gate.py       the Rust dependency gate
    ├── npm_audit_allowlist.json the accepted npm advisories, with their dates
    ├── rust_audit_allowlist.json the accepted Rust advisories, with their dates
    └── test_audit_policy.py     the two gates' own regression test
```

The checks live in one file, `scripts/verify.py`, and the two entry points
call it. That is deliberate: a platform that ran a different list of
checks would be a silent difference nobody noticed.

The two dependency gates live in their own files for the same reason. The
continuous integration security job runs them on their own, without the
compile and the unit tests, and their rules have to be testable without a
registry or an advisory database — so the policy is applied by the gate
rather than by the audit tool's exit code. `scripts/audit_policy.py`
holds the schema both allowlists are validated against and the rule that
decides when an exception has expired.

## The boundary

The frontend may call a narrow set of typed Tauri commands. The Rust side
owns persistence, validation, simulator state, approvals and audit records.
The frontend never receives a raw SQLite error or a stack trace: every
failure arrives as `{ code, message }` where `code` is stable and `message`
is safe to display.

The command classes are:

- dashboard summary and timeline
- list and get agents
- list and get runs, and run events
- list approvals and record a decision
- list audit events
- list policies and toggle one
- list tools, list the team
- get and update settings
- simulator state, start, pause, tick, speed and reset

A transport trait and a connector trait are the intended seams for future
remote agents. The v0.1 implementation uses a local simulator adapter only.

## Concurrency

`rusqlite::Connection` is `Send` but not `Sync`, so the shared state holds
the database behind a mutex. The lock is taken **once per command** and held
for the whole command, which is what keeps a read and the write that follows
it atomic with respect to other commands. A poisoned lock is recovered rather
than allowed to panic: a command that panicked must not take the whole
application down.

## The data model

Twelve tables, created by migration 001:

`agents`, `agent_capabilities`, `runs`, `run_stages`, `run_events`,
`approvals`, `approval_decisions`, `tools`, `policies`, `team_members`,
`audit_events`, `app_settings`

plus `schema_migrations`, which the migration runner owns.

Foreign keys are enforced. Common filter columns are indexed. Timestamps are
RFC 3339 in UTC and are localised only for display. Multi-table writes run
in a transaction — recording an approval decision and the state change it
causes are one atomic act, so a decision can never be recorded without the
state changing, or the other way round.

The on-disk shapes (`db/records.rs`) are deliberately separate from the
domain types (`domain/mod.rs`). A schema change therefore does not silently
change the UI contract.

## State and errors

Commands return `Result<T, AppError>`. `AppError` carries a stable code
(`db_error`, `not_found`, `invalid_input`, `conflict`, `internal_error`) and
a message that is safe to show a person. The UI branches on the code, never
on the message text.

## The simulator

The simulator is the only source of "live" activity, and it is driven by an
explicit tick from the frontend rather than by a background thread. The
frontend sets an interval whose period is derived from the configured speed
and clamped to a floor of 400 ms, so the backend only advances when the UI is
ready for it.

Determinism comes from the persisted step counter: the step selects what
happens, so the same step always produces the same event. The phase cycle is
create, advance, advance, approval, fail, complete. Resetting restores the
seed and rewinds the counter, so a replay reproduces the same sequence —
including the same identifiers.

Two rules keep the simulation honest. The six phases of a cycle belong to
**one agent and one run**: the agent is chosen by the cycle, not by the
step, so the run is created, advanced and finally resolved as a single
piece of work. And the simulator only ever acts on its **own** runs, whose
identifiers begin `sim-run-`; the seeded runs are the fixed baseline the
operator sees, and the simulation never advances, fails or completes them.

A run that is waiting on a human decision is neither failed nor completed
behind the operator's back. It waits until the decision is made.
