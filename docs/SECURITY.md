# Security and Privacy

## The privacy boundary

Everything in this repository is synthetic. The seed data uses invented
agent names, invented tasks and invented tool names. There are no real hosts,
no real people, no real model names and no network addresses.

The build is checked for accidental leakage: `scripts/verify.sh` runs the
skill's verifier, which scans every text file for email addresses, private
IPv4 ranges, bearer tokens and secret assignments. RFC 5737 documentation
ranges and `127.0.0.1` are the only addresses permitted.

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
for the frontend and `Cargo.lock` for the Rust side. `npm audit` and
`cargo audit` are the tools that would report dependency advisories;
`cargo audit` is not installed in every environment, and when a tool is
unavailable that is recorded as a limitation rather than skipped silently.

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
