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

The sweep runs two dependency audits.

- **The production dependencies are the gate.** `npm audit --omit=dev`
  fails the sweep if the shipped dependency tree has an advisory.
- **The development toolchain is reported, not gated.** Its advisories are
  printed on every run and listed here.

`cargo audit` is the tool that would report the Rust advisories. It is
not part of the Rust toolchain and is not installed without the
operator's agreement, so the sweep reports it as a skip with that reason.
It is never reported as a pass.

### The advisories in the development toolchain

All of these are in the test and build tools. None of them ships with the
application, and the running application does not exercise any of them.

| Advisory | Package | Path | Severity | Kind |
|---|---|---|---|---|
| GHSA-5gmw-xhrv-c9v3, GHSA-85c8-ppgw-ccpr | tinypool | vitest → tinypool | critical | development |
| GHSA-67mh-4wv8-2f99 | esbuild | vitest → vite-node → esbuild | moderate | development |
| GHSA-82fw-gwwq-j7x9, GHSA-qxg6-89mg-rqv8 | @vitest/mocker | vitest → @vitest/mocker | moderate | development |

The tinypool advisories are a prototype-pollution gadget in the worker
options that leads to remote code execution. Reaching it needs control of
the options a Vitest worker is started with; nothing in this repository
takes them from untrusted input, and the application itself contains no
JavaScript at run time beyond the bundle. The esbuild advisory concerns a
development server that answers requests from any site; the application
ships no development server. The mocker advisories concern the test
runner's own mock registration.

The fixes require Vitest 5, which is a breaking change. A broad
dependency upgrade is not performed for a release, so these are recorded
here and in `BUILD_STATE.md` instead.

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
