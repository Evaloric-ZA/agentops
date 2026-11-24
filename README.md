# AgentOps

A local, synthetic agent operations console. AgentOps shows what a future
operator would see while distributed agents are working: a shift overview, an
agent registry, a run timeline, and the approvals that need a human.

**Version 0.1 is a local proof of architecture.** Every record in it is
synthetic. Nothing here connects to a real agent, a real model, or any
network service, and no installer for a production deployment is implied.
v0.1.2 is a maintenance release: no feature is added, no screen changes,
and the data model is unchanged.

The development and continuous integration baseline is **Node.js 24**. The
minimum supported Rust version is **1.90**. The versions actually tested
are recorded in [docs/TESTING.md](docs/TESTING.md).

## What it is

- A **Tauri 2** desktop application: a Rust core with a Svelte 5 and
  TypeScript frontend.
- **SQLite** persistence, owned entirely by the Rust side, with versioned
  migrations.
- A **deterministic simulator** that produces the activity you see. The same
  seed always produces the same sequence, so a demo is reproducible and a
  test is stable.
- **Synthetic data only** — invented agents, invented tasks, invented tools.

## Screens

| Screen | What it shows |
|---|---|
| **Night Shift** | Four summary cards, the shift timeline, the approvals waiting on you, and three lower panels: reviewer disagreement, compute allocation and tool health |
| **Agents** | The registry: searchable and filterable, with a detail drawer showing capabilities and event history |
| **Runs** | Every run, filterable by status and searchable by task or agent, with a detail view of stages and the event timeline |
| **Waiting on You** | The approval queue, with local approve, reject and comment |
| **Policies** | The local rule set, with each rule switchable |
| **Access** | Trust levels and which capabilities are granted or withheld |
| **Compute** | Sample compute units by model and runs by agent |
| **Audit Log** | Every recorded action, newest first |
| **Tools** | The tool registry and its health |
| **Team** | Who is on shift, and which agents they cover |
| **Settings** | Theme, density, simulator speed and timezone, all persisted |
| **Crew** | Compact status rows for every agent |

## Running it

Node.js 24 is required; `.node-version` carries `24`.

```bash
npm ci
npm run tauri dev
```

The application creates its database on first launch, in the platform's
application data directory, and seeds it with synthetic data.

## Building it

```bash
npm ci
npm run tauri build
```

See [docs/PACKAGING.md](docs/PACKAGING.md) for the native prerequisites on
each platform, and for what each build produces.

## Checking it

```bash
npm run check        # svelte-check: types and template diagnostics
npm test -- --run    # vitest: the frontend unit tests
npm run build        # the production frontend bundle

cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features

bash scripts/verify.sh
```

`scripts/verify.sh` runs the whole sweep and prints one line per stage:
the frontend, the Rust, the native build, the version agreement between
the manifests, the dependency audits, the privacy scan and the privacy
scanner's own regression test. The stages are listed in
[docs/TESTING.md](docs/TESTING.md). On Windows the same sweep is run by
`scripts/verify.ps1`; the checks themselves live in `scripts/verify.py`,
so the two platforms run exactly the same checks.

A stage that cannot run is reported as a skip, with its reason, and is
never counted as a pass.

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — the stack, the boundaries,
  the data model and the command surface
- [docs/SECURITY.md](docs/SECURITY.md) — the desktop security posture, the
  privacy boundary, and what v0.1 deliberately does not do
- [docs/TESTING.md](docs/TESTING.md) — what is tested and how
- [docs/PACKAGING.md](docs/PACKAGING.md) — prerequisites, build commands and
  outputs
- [CHANGELOG.md](CHANGELOG.md) — what changed in each release

## Scope

v0.1 deliberately excludes: real agent protocols, model APIs, remote network
connections, VM cloning, source-code processing, repository push, CI/CD,
identity providers and deployment actions. Where a future capability is
anticipated, an interface is defined but not activated, and the documentation
says so.

## Licence

MIT. See [LICENSE](LICENSE).
