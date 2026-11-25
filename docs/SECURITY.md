# Security and Privacy

## The privacy boundary

Everything in this repository is synthetic. The seed data uses invented
agent names, invented tasks and invented tool names. There are no real hosts,
no real people, no real model names and no network addresses.

The build is checked for accidental leakage: the sweep runs
`scripts/verify_repo.py`, which scans every file in the tree for email
addresses, private IPv4 ranges, bearer tokens and secret assignments. The
scan is extension-blind — it reads every file and scans it if it decodes as
UTF-8 and holds no NUL byte — so a file's name cannot decide whether it is
checked. RFC 5737 documentation ranges and `127.0.0.1` are the only
addresses permitted.

The scanner has its own regression test, `scripts/test_verify_repo.py`, so
the false negatives it once had cannot come back unnoticed.

It also fails on a **source file that holds nothing**. An empty source
file is usually an edit that went to the wrong path: the file is
created, and the content lands somewhere else. Nothing declares the
stray as a module, so the compiler never sees it and nothing else
notices it either. One of them sat in the tree through three releases
before somebody looked at the file list.

The rule covers every source directory — `src`, `src-tauri/src`,
`src-tauri/tests` and `scripts` — and a file holding nothing but
whitespace counts as empty, because it declares an empty module. An
empty sweep script matters most here: it exits with a success and
reports nothing, which is indistinguishable from a sweep that found no
problems.

### What the scanner decides about each file

Every file gets one of six verdicts, and the six counters are printed at
the end of every run.

| Verdict | What it means |
|---|---|
| scanned | the file was searched for patterns |
| binary skipped | the file is not text, so there is nothing in it to read |
| generated excluded | the file is generated output, not source |
| oversize review required | the file is larger than the limit and is not a known dependency lockfile, so it could not be searched; a person has to look at it |
| unreadable | the file or its directory could not be read |
| violation | a pattern matched |

An oversized file used to disappear from verification without a word, which
meant a large file could be added to the tree and never be looked at. It
is now reported, and the verification fails until someone has reviewed it.
A known dependency lockfile is not exempt from the search: it is read a
line at a time, with the same patterns as every other file. The one
limitation of that reader is that a match spanning more than two line
breaks would not be found in such a file.

**The scanner never prints the value it found.** A finding names the file,
the line, the column and the kind of pattern, and nothing else.

### What the scanner is not

The scanner is a local repository source scanner. It is pattern-based. It
is **not** a guarantee that no secret exists: it looks for shapes it
knows, and a secret written in a shape it does not recognise will pass.
It does **not** scan the Git history — it looks at the working tree only,
so a secret that was committed and later removed is not found. It is
**not** a replacement for a dedicated secret-scanning service, which can
search the history, apply an organisation's own rules, and be configured
separately from the build.

There is **no telemetry, no analytics, no crash upload and no remote font**.
The frontend loads nothing from the network: the CSP restricts it to
`'self'`, and the bundle contains every asset.

## Desktop security

- The Tauri capability set is `core:default` and nothing else. No
  filesystem, shell, process, HTTP, clipboard, updater or global-shortcut
  permission is granted, so the frontend cannot reach any of them even by
  accident.
- The CSP is restrictive: `default-src 'self'`, with no remote scripts and
  no unsafe inline execution.
- Every command input is validated in Rust. Identifiers are checked for
  length and character set; free text is length-bounded; enumerations are
  checked against the allowed set; the simulator speed is rejected if it is
  not finite and is clamped to a renderable range.
- Every SQL statement is parameterised. No SQL is built by concatenation.
- The frontend never receives a raw database error or a stack trace.
- The application binds no network listener of its own and makes no outbound
  request.
- Mock and simulator labels are visible in the interface: the simulator bar
  says "All activity is simulated and local", and the compute view says the
  figures are synthetic.
- `unwrap()` is not used in application paths. A poisoned lock is recovered;
  a malformed record reads as an empty value rather than panicking.

## Dependency handling

Direct dependencies are pinned through the lockfiles: `package-lock.json`
for the frontend and `Cargo.lock` for the Rust side.

**Both dependency trees are audited by a gate, and the development
toolchain is audited as strictly as the shipped code.** A developer runs
the test toolchain and the continuous integration runs it, so an advisory
in it is an advisory in something that executes. "It does not ship" is
not a reason to accept a Critical advisory.

The two gates live in their own files, so the continuous integration
security job can run them on their own and so their rules can be tested
without a registry or an advisory database:

| Gate | What it reads | What fails it |
|---|---|---|
| `scripts/npm_audit_gate.py` | `npm audit --json` | Critical, High, an expired exception, an exception for an advisory that no longer exists, an audit that could not run |
| `scripts/rust_audit_gate.py` | `cargo audit --json` | a vulnerability, an unmaintained, unsound or yanked crate, an expired exception, an exception for an advisory that no longer exists, an audit that could not run |

The npm policy is by severity:

| Severity | What the gate does |
|---|---|
| Critical | fails |
| High | fails |
| Moderate | warns |
| Low | reports |
| Informational | reports |

The Rust policy does not distinguish a vulnerability from an
informational advisory when deciding what to do about it: both fail
unless the advisory is listed in `scripts/rust_audit_allowlist.json`. An
unsound crate is a defect whether or not anyone has exploited it, and an
unmaintained crate is a risk that nobody is fixing.

### The exceptions, and the dates on them

Each exception is a dated decision, not a permanent exemption. The two
allowlists carry the same rules:

- an exception says which advisory it covers, which package it covers,
  why it is accepted, what its scope is, what would close it, the date
  it was added and **the date it has to be looked at again**;
- **an exception past its review date fails the release gate**. Nothing
  renews it automatically: an operator has to look at it and decide
  again;
- an exception for an advisory that the audit no longer finds fails the
  release gate, because the file has to describe the world as it is;
- an advisory that is not listed is never accepted, however new it is;
- a malformed allowlist — a missing field, an empty field, a malformed
  date, a duplicate entry, an unknown scope — fails the gate rather than
  being quietly ignored. A malformed allowlist must not turn into an
  empty allowlist.

The review periods are finite: between thirty and ninety days from the
date an exception is added, depending on the severity and on whether a
fix exists.

The npm allowlist, `scripts/npm_audit_allowlist.json`, is **empty**. The
Vitest upgrade in the v0.1.3 release closed every advisory the audit
found, so there was nothing to accept. The file is kept, with its schema
and its policy, so that a future exception has one place to live and one
place to be reviewed.

### Developer verification and release verification

The sweep has two modes, and the difference between them is the point of
this release.

| | Developer verification | Release verification |
|---|---|---|
| the command | `bash scripts/verify.sh` | `bash scripts/verify.sh --release` |
| a check that cannot run | reported as a skip | **a failure** |
| `cargo-audit` not installed | the Rust audit is skipped | **the Rust audit fails** |
| the registry unreachable | the npm audit is skipped | **the npm audit fails** |
| an expired exception | reported as a warning | **a failure** |
| an unaccepted advisory | a failure | a failure |

A release is not verified by a control that did not run. The developer
mode exists so that somebody working offline can still run the rest of
the sweep; it is not a weaker release gate, it is a different command.

The two gates can also be run on their own, which is what the continuous
integration security job does:

```bash
python3 scripts/npm_audit_gate.py --release .
python3 scripts/rust_audit_gate.py --release .
```

They exit with 0 when they pass, 1 when they fail and **2 when they
could not run**, so that the sweep can tell a check that passed from a
check that did not happen.

`cargo audit` is not part of the Rust toolchain. When it is not
installed, the developer sweep reports the stage as a skip with that
reason. It is never reported as a pass. The release sweep fails it.

### The vulnerability that was found and fixed

`RUSTSEC-2026-0459` — the bundled SQLite C library, through
`rusqlite` — carried `CVE-2026-119375`, medium severity, 6.8. The
application compiles SQLite into its binary, so the defect shipped with
it. `rusqlite` moved from 0.32 to 0.43 and `libsqlite3-sys` from 0.30.1
to 0.40.0, which is past the 0.39.0 that carries the fix. The
application's own use of the library did not change: the same calls
compile, all forty-nine Rust tests pass, and the upgrade needs Rust 1.89,
which is below the declared minimum of 1.90.

### The Rust advisories that are accepted

These three are warnings, not vulnerabilities. Each was re-checked for
the v0.1.3 release: the advisory still exists, the crate is still in the
lockfile, the reason still applies and the remediation still stands. The
dependency paths were re-derived with `cargo tree --target all -i`, so
they are the current paths and not remembered ones.

| Advisory | Package | Scope | Kind | Exercised here | Review by |
|---|---|---|---|---|---|
| RUSTSEC-2026-0543 | chrono 0.4.45 | runtime | unmaintained | Only on the application's own timestamps | 2027-02-23 |
| RUSTSEC-2024-0429 | glib 0.18.5 | platform-specific (Linux only) | unsound | No: not in the macOS or Windows build, and the affected functions are never called | 2027-01-24 |
| RUSTSEC-2024-0370 | proc-macro-error 1.0.4 | build-time (Linux only) | unmaintained | No: a build-time dependency of the Linux GTK bindings | 2027-02-23 |

The paths are `agentops → chrono`, and
`agentops → tauri 2.12.7 → gtk 0.18.2 → atk / tao / webkit2gtk → glib 0.18.5`,
with `proc-macro-error` reached through `glib → glib-macros`.

**chrono.** The advisory reports no patched release, because the crate
is no longer maintained. Its use was inventoried for the v0.1.3
release: three files, `src-tauri/src/util.rs` (four helpers that format,
parse and subtract RFC 3339 strings), `src-tauri/src/simulator/mod.rs`
(the current instant) and `src-tauri/src/seed.rs` (whole-minute shifts on
the seed's own timestamps). The stored representation is an RFC 3339
string with whole seconds and a `Z` suffix in a `TEXT` column, and the
frontend parses those strings with the platform's date parser.

The migration to `time` or `jiff` was assessed and deliberately
deferred. It is small — three files — but it changes the timestamp
library in the same release as the test-toolchain upgrade, which widens
the blast radius of the release for an advisory that carries no exploit.
It is planned for the next maintenance release, with a comparison test
that asserts the new formatter's output is byte-identical to the old
one, so the persisted format and the frontend contract cannot drift
silently. No persisted timestamp format changes in this release.

**glib.** A patched version exists: glib 0.20.0 carries the fix. It is
not reachable. The GTK stack arrives through Tauri, and a dry-run
upgrade of the Tauri dependency does not move it; the GTK versions only
change in the next major version of Tauri, which is a prerelease.
Upgrading Tauri's major line is an application change rather than a
dependency security fix, and a deeply transitive GTK crate must not be
forced with a version override.

**proc-macro-error.** It is a build-time dependency of the GTK bindings
and never reaches the shipped binary's runtime. It disappears with the
same GTK upgrade as glib.

### The advisories in the development toolchain, and what was done about them

The v0.1.2 release shipped with six advisories in the test and build
tools, and reported them rather than gating them. The v0.1.3 release
gated them and then closed them.

| Advisory | Package | Path | Severity | Kind |
|---|---|---|---|---|
| GHSA-5gmw-xhrv-c9v3, GHSA-85c8-ppgw-ccpr | tinypool 1.1.1 | vitest → tinypool | critical | development |
| GHSA-5xrq-8626-4rwp, GHSA-82fw-gwwq-j7x9 | vitest 2.1.9 | vitest | critical | development |
| GHSA-4w7w-66w2-5vf9, GHSA-v6wh-96g9-6wx3, GHSA-698v-4r3x-3vcv, GHSA-r825-w8mm-5pxw, GHSA-587h-7wmj-h5pm, GHSA-9jrq-w75r-8gcw, GHSA-rq7h-c2jc-7f22, GHSA-fx2h-pf6j-xcff | vite | vitest → vite | high | development |
| GHSA-82fw-gwwq-j7x9, GHSA-qxg6-89mg-rqv8 | @vitest/mocker | vitest → @vitest/mocker | moderate | development |
| GHSA-67mh-4wv8-2f99 | esbuild | vitest → vite-node → esbuild | moderate | development |

The tinypool advisories are a prototype-pollution gadget in the worker
options that leads to remote code execution. The Vitest advisories
include one that reads and executes an arbitrary file while the Vitest UI
server is listening. The vite advisories include path traversal, a
Windows `server.fs.deny` bypass and several `launch-editor` defects that
can turn into command execution.

**None of them is now accepted.** Vitest was upgraded from 2.1.9 to
5.1.1, which is the smallest maintained line that carries the fixes:

| Package | Before | After | Why |
|---|---|---|---|
| vitest | 2.1.9 | 5.1.1 | the fixes are in this line and later |
| vite | 6.4.6 | 6.4.6 | unchanged: the direct dependency was already past the vulnerable range, and Vitest 5 accepts `vite ^6.4.0` |

The upgrade was made deliberately rather than with `npm audit fix
--force`, and it was not a broad upgrade of the frontend stack: one
direct dependency changed. Vitest 5 accepts `vite ^6.4.0 || ^7.0.0 ||
^8.0.0`, so the project's Vite did not have to move, and its `engines`
field accepts Node 22.12, Node 24 and Node 26, so the Node 24 baseline
is unchanged.

Two packages left the tree entirely: `tinypool`, which was the source of
the Critical advisories, and `vite-node`, which pulled in its own nested
copy of esbuild. The lockfile lost about eleven hundred lines and the
audit now reports **no advisories at all** — not none that were
accepted, none.

The whole frontend suite was run after the change: sixty tests pass,
`svelte-check` reports zero errors and zero warnings, and the production
bundle assembles. The upgrade changed no application code and no test
code.

## What v0.1 deliberately does not do

These are out of scope, and their absence is not a gap to be filled by
turning on a setting:

- no real agent protocol, model API or remote network connection
- no repository push, CI/CD or deployment action
- no identity provider, no account, no authentication service
- no VM cloning or source-code processing
- no updater, no clipboard access, no global shortcuts

## Defined but not activated

The following are interfaces only. They are not security claims for v0.1,
and nothing in this build implements them:

- device identities
- one-time pairing
- encrypted transport
- connector capability declarations
- scoped task authorisation
- human approval gates beyond the local, synthetic approval queue

The local approval queue is a demonstration of the interaction, not an
enforced authorisation boundary for a real system.
