# M0-08 — Validation & Proof-of-Concept Gates

**Status:** M0 completion contract.  
**Purpose:** Define objective evidence required to declare architecture ready for implementation.

## 1. M0 is complete only when

- system context, trust boundaries and deployment modes are documented;
- Rust crate ownership and forbidden dependencies are agreed;
- OpenAPI, AsyncAPI and JSON Schemas are present and pass validators;
- domain lifecycle and event invariants are documented;
- P0 threat controls and fail behavior are assigned;
- Windows and browser support tiers are explicit;
- Moonframe dependency strategy and clean-consumer test are specified;
- POC experiments, named owners, environments and pass/fail criteria are recorded (the POCs themselves are the first M1 coding work, not a precondition that can be completed without implementation);
- all index/README links are valid;
- no contract claims that source code or security controls already exist.

## 2. M0 artifact inventory

| Artifact | Required location | Status at initial M0 document publication |
|---|---|---|
| System context/deployment | docs/specs/m0/01-system-context-and-deployment.md | Written |
| Crate ownership/dependency | docs/specs/m0/02-rust-crate-boundaries.md | Written |
| API/session/event contract | docs/specs/m0/03-api-session-event-protocol.md | Written |
| Domain/event invariants | docs/specs/m0/04-domain-model-and-event-invariants.md | Written |
| Threat model/capabilities | docs/specs/m0/05-threat-model-and-capability-security.md | Written |
| Platform matrix | docs/specs/m0/06-platform-support-matrix.md | Written |
| Moonframe integration | docs/specs/m0/07-moonframe-consumption-contract.md | Written |
| Validation gates | docs/specs/m0/08-validation-and-poc-gates.md | Written |
| Machine schemas | contracts/** | Must be validated before M0 closure |
| M0 link/consistency audit | Repository check | Must pass before M0 closure |

## 3. Proof-of-concept experiments

### POC-01 — Minimal Rust kernel boundary
Build a minimal Rust workspace with domain, kernel, runtime and server boundaries only.

Pass:
- domain has no network/database/UI dependency;
- kernel executes one deterministic fake-provider turn;
- runtime supervises and cancels the run;
- server starts independently and exposes health/capabilities;
- cargo tree shows no circular or forbidden dependency edge.

Fail:
- kernel imports Axum/SQLx/provider SDK directly;
- UI types leak into domain;
- tests require live LLM credentials.

### POC-02 — Provider stream adapter
Use one provider adapter plus a deterministic fake provider.

Pass:
- streaming text and tool-call deltas normalize to canonical events;
- provider-specific metadata is retained where behavior requires it;
- timeout, 429, auth and invalid-request errors classify distinctly;
- secrets never enter request logs/events.

Fail:
- provider-specific model strings or SDK types leak into kernel;
- a provider failure marks the run completed.

### POC-03 — Local session/event recovery
Persist one session/run/turn/tool-call lifecycle in SQLite WAL.

Pass:
- kill server after tool start and before result;
- restart identifies unfinished operation;
- event cursor replays or resyncs without duplication;
- the UI never displays a missing result as success;
- repeated idempotency key does not create a second side effect.

Fail:
- a lost process is marked succeeded;
- event sequence gaps are silently ignored;
- duplicate retry causes an additional write.

### POC-04 — Windows process and workspace boundary
Run on supported Windows 10/11 x64 baseline.

Pass:
- workspace path traversal, symlink/junction, UNC, case-folding and reparse-point test corpus is contained;
- child process tree is tracked and stoppable;
- environment is allowlisted and credential canary does not leak;
- executor reports effective isolation controls;
- requested isolation failure is explicit.

Fail:
- shell execution falls back to unrestricted host mode without a deliberate user decision;
- a path check can be bypassed by a filesystem race;
- process survives a supposedly completed cancellation without being reported.

### POC-05 — Container/remote worker lifecycle
Use a Linux container executor and one remote-worker protocol simulator.

Pass:
- worker enrolls, receives scoped job token, claims lease, streams output, heartbeats, completes and revokes;
- replayed/stale job token is rejected;
- disconnect and reconnect reconcile rather than double-run;
- resource/network/filesystem limits are observable.

Fail:
- API server directly executes remote untrusted shell;
- worker has an organization-wide secret;
- lease expiry is treated as proof that a process stopped.

### POC-06 — WebSocket replay and TypeScript SDK
Use a deterministic event generator and two clients.

Pass:
- no replay/live gap;
- ack advances only after state reducer applies;
- stale cursor forces snapshot/resync;
- duplicate event is idempotent;
- revoked client stream closes;
- Desktop and Web consume the same generated event types.

Fail:
- frontend must poll internal database state;
- event ordering relies on timestamp only;
- one client has a different agent lifecycle.

### POC-07 — Moonframe consumer and Tauri/Web shell
Use the immutable Moonframe pin from M0-07.

Pass:
- clean React/Vite consumer imports UI, CSS, theme and skin exports;
- same shell renders in browser and Tauri;
- no duplicate React/ThemeProvider/CSS;
- default Moonpunk Dark works, Light and Classic variants render;
- keyboard/focus/reduced-motion smoke passes.

Fail:
- RockCode must duplicate Moonframe tokens or base components to render;
- package requires an undocumented local path;
- Tauri and Web use different base component behavior.

## 4. Required validation commands (to be established in CI)

These are target checks, not claims they have already run:

- OpenAPI parser/linter and breaking-change checker.
- AsyncAPI parser/linter.
- JSON Schema draft 2020-12 validation using valid and invalid fixtures.
- Mermaid diagram render check where CI tooling supports it.
- Markdown link checker for repository-relative links.
- Rust format, clippy, test and dependency audit once Rust code exists.
- TypeScript typecheck, lint, package consumer build and Playwright smoke once UI code exists.
- License/advisory checks for all direct and transitive dependencies.
- Windows CI for Tauri, local daemon, path/process/sandbox and installer.
- Linux CI for headless server, PostgreSQL and container worker.

## 5. M1 entry gate

M1 source implementation may start when:
1. M0 machine-readable contracts are valid.
2. M0 documents agree on names, status enums, route paths, IDs and ownership.
3. POC acceptance criteria have a named owner and explicit environment.
4. Windows sandbox path is selected for the first POC; if unavailable, shell remains inspect-only or explicitly trusted-local, never silently isolated.
5. Moonframe consumer test strategy is pinned.
6. No critical architecture contradiction remains open.

The M1 work order should be a small vertical slice: one local workspace, one provider adapter, one deterministic agent turn, file read/write/search, a guarded shell path, durable session/events, local agentd API, then one Web and one Desktop client.

## 6. Explicitly deferred beyond M0

- Full provider catalog and subscription login methods.
- Plugin marketplace and public signing infrastructure.
- Full multi-tenant billing/quotas.
- Browser/computer-use implementation.
- Voice, image generation and local model hosting.
- Advanced long-term memory.
- Production remote tunnel/relay.
- Theme/artwork production beyond Moonframe integration.
- Public distribution, auto-update signing and release operations.

## 7. M0 closeout report template

When M0 is validated, record:
- exact main commit SHA;
- files and contract versions;
- OpenAPI/AsyncAPI/JSON Schema validator versions and outputs;
- link/consistency audit result;
- POC results (pass/fail/not run);
- known gaps and accepted restrictions;
- explicit M1 authorization/status.

Do not label M0 fully validated if contracts have only been authored but validators/POCs have not run.
