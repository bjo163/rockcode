# 02 — Gap Analysis

**Project:** RockCode — Rust-first Autonomous Coding Agent Platform  
**Snapshot:** 2 November 2026  
**Status:** Architecture planning; no production implementation authorized by this document.

## 1. Executive assessment

The upstream landscape has strong individual implementations, but no audited repository is a clean, complete match for the target:

- Goose, Codex, Drift, and Agent Code provide valuable Rust implementation references.
- Drift is the closest local Windows Desktop + remote browser experience.
- Goose offers a reusable Rust agent ecosystem and a composable state-machine approach.
- Codex offers the deepest visible execution, protocol, policy, and OS sandbox references.
- OpenHands provides the strongest Web control-plane, remote backend, workspace, and automation architecture, but its main SDK/server is Python rather than Rust.
- OpenCode is a valuable product/UX and coding workflow reference, not a Rust engine foundation.
- DeepSeek Harness is a useful plugin/capability composition reference, but its preview release track is too volatile to adopt as a stable dependency without pinning and compatibility tests.

The central gap is therefore **integration architecture**, not lack of individual features. The new platform must make one Rust runtime portable across local desktop, local headless server, remote server, and isolated execution workers.

## 2. Target definition

### 2.1 Product outcomes

1. Windows 10/11 desktop application as the first-class local product.
2. Web application that can control local or remote agent servers.
3. Rust CLI/TUI for power users and automation.
4. One shared engine and protocol for every client.
5. Local, container, and remote execution backends.
6. Multi-agent delegation and background job execution.
7. Extensible tools, providers, skills, MCP/ACP, and sandboxed plugins.
8. Coding workspace capabilities: terminal, filesystem, Git, worktrees, diff, undo, LSP, test/lint, browser, and review.
9. Local-first single-user mode that can later scale to authenticated multi-user hosting.

### 2.2 Non-goals for the first production milestone

- Building a general-purpose no-code automation platform before the coding-agent core is stable.
- Implementing every upstream provider and integration at launch.
- Building a proprietary model or inference engine.
- Treating prompt instructions as security controls.
- Giving project-supplied plugins or skills implicit host authority.
- Requiring a hosted backend for normal desktop usage.
- Forking or merging upstream repositories wholesale.

## 3. Gap register

Priority definitions:

- **P0 — Blocking:** required before a usable, trustworthy first release.
- **P1 — Important:** required for a competitive advanced coding agent, after P0.
- **P2 — Expansion:** required for platform/ecosystem scale, not MVP-blocking.

| ID | Gap | Why upstream alone does not solve it | Priority | Planned resolution | Acceptance evidence |
|---|---|---|---|---|---|
| GAP-001 | Single engine across desktop/web/CLI | Some upstream projects have UI-specific or server-specific paths; not all expose a clean reusable engine boundary | P0 | Rust library + `agentd` server + shared protocol/SDK | Same session can be started, continued, cancelled, and resumed from all clients |
| GAP-002 | Stable versioned protocol | Each upstream uses different REST, WebSocket, JSON-RPC, ACP, or custom contracts | P0 | Versioned REST + WebSocket event protocol; adapters for ACP/MCP | Contract tests and generated clients pass across versions |
| GAP-003 | Crash-safe event replay | Session persistence does not automatically guarantee correct UI reconstruction after lost sockets | P0 | Durable event sequence, cursor replay, snapshot/hydrate, resync | Kill/restart server during active turn; client recovers without duplicate or missing terminal state |
| GAP-004 | Windows-grade execution isolation | Linux/macOS sandbox techniques cannot be assumed equivalent on Windows | P0 | Platform-specific sandbox providers; container fallback; fail closed when requested isolation cannot be provided | Adversarial tests prove filesystem/process/network boundaries on supported Windows configurations |
| GAP-005 | Permission vs sandbox separation | Approval policy is not process/filesystem/network confinement | P0 | Independent policy decision and OS/container enforcement layers | Policy approval never bypasses sandbox constraints |
| GAP-006 | Secure credentials | Local and remote credential handling differ; frontend must never become the secret vault | P0 | OS keychain/DPAPI locally; server-side encrypted vault remotely; redaction | Secrets absent from logs, browser storage, telemetry, and tool output where redaction applies |
| GAP-007 | Local/remote execution abstraction | Desktop local process execution and hosted/container execution have different lifecycles | P0 | `Executor` trait and worker protocol | Same tool request runs on local and remote workspace with identical event semantics |
| GAP-008 | Typed provider capability layer | Generic OpenAI compatibility does not preserve all provider-specific semantics | P0 | Native provider adapters + capability metadata + protocol dialects | Provider conformance suite validates streaming, tools, vision, reasoning and usage per adapter |
| GAP-009 | Portable context management | Context windows, usage, compaction, summaries and provider caching differ | P0 | Context budget service with pluggable token estimators and summarizers | Long sessions preserve required context and recover from context-length/provider errors |
| GAP-010 | Tool registry and safe execution pipeline | Tools, MCP calls, internal tools and worker tools need one policy/audit flow | P0 | Schema validation → authorization → sandbox → execute → normalize → persist | Every tool call produces traceable lifecycle and policy decision |
| GAP-011 | Workspace isolation and Git concurrency | Parallel agents modifying the same working tree produce race/conflict risk | P1 | Workspace leases + worktree-per-agent/task by default | Parallel subagents cannot silently overwrite each other's changes |
| GAP-012 | Durable multi-agent orchestration | Simple subagent spawn is insufficient for retries, dependencies, cancellation and parent-child result delivery | P1 | Task DAG, child sessions, inbox/message bus, concurrency limits and budgets | Parent can cancel/resume/inspect child tasks; child outcomes persist after restart |
| GAP-013 | LSP/test feedback loop | A coding agent should receive diagnostics produced by edits and verify work | P1 | LSP service + formatter/test/check adapters | Diagnostics are attached to relevant steps; test results are structured and reproducible |
| GAP-014 | Browser automation integrated into workspace | External browser MCPs do not provide unified lifecycle, screenshot streaming, remote access and session ownership | P1 | Browser service with CDP/BiDi drivers and browser session panel | Agent and user inspect/control the same isolated browser session |
| GAP-015 | Safe plugin ecosystem | Native plugins can inherit host process privileges; registries introduce supply-chain risk | P1 | WASM Component Model + capability-limited host API + signed artifact registry | Untrusted plugin cannot access undeclared host capabilities; malicious package tests fail safely |
| GAP-016 | MCP/ACP interoperability | Internal tool schema and external protocol schemas differ | P1 | Internal canonical tool model + MCP and ACP adapters | Compatibility suite covers stdio/HTTP MCP and ACP client/server roles selected for product |
| GAP-017 | Web multi-user tenancy | A desktop remote-access gateway is not a multi-tenant SaaS boundary | P1 | Org/user/device/workspace RBAC, worker isolation, quotas and audit | Cross-tenant isolation tests and authorization matrix pass |
| GAP-018 | Automation server | Scheduling and event-trigger orchestration should not run inside interactive UI process | P2 | Separate durable scheduler/dispatcher service | Scheduled and webhook jobs survive restart and provide run history |
| GAP-019 | Evaluation and regression suite | Unit tests do not establish coding-agent quality or tool correctness | P1 | Task benchmark, fake providers, replay tests, tool conformance, safety test corpus | Repeatable evaluation report for each release |
| GAP-020 | Plugin marketplace governance | A registry must handle provenance, signatures, versioning, permissions and rollback | P2 | Signed manifests, hashes, source policy, review status and update pinning | Tampered package rejected; rollback works; trust state visible to user |
| GAP-021 | Observability | Interactive sessions, remote workers and tool calls require correlated tracing | P1 | OpenTelemetry traces, event correlation IDs, redacted logs and metrics | Trace follows request → agent → tool → executor → result |
| GAP-022 | Release/platform matrix | Upstream support varies by GUI and operating system | P0 | Windows x64 first; Linux server and macOS later behind CI matrix | Reproducible build/install/upgrade/smoke test per supported target |
| GAP-023 | Data migration/versioning | Local schema, remote schema, events and snapshots need independent versioning | P0 | SQL migrations + event schema version + snapshot version + migration tests | Upgrade N-1 to N preserves sessions, jobs, credentials references and workspaces |
| GAP-024 | Remote connectivity and device trust | Exposing a local port or bearer key is insufficient for secure device pairing | P1 | TLS, device enrollment, revocable tokens, scoped credentials, optional tunnel | Revoking a device terminates active streams and blocks reconnection |
| GAP-025 | Voice/computer-use advanced UX | These are expensive platform features that can destabilize the first release | P2 | Add behind capability interfaces after execution core is stable | Capability-specific tests and user-granted access scopes |

## 4. Highest-risk design traps

### 4.1 Monolithic Rust crate disguised as modular architecture

Drift shows how much can be delivered in a Rust engine, but a single large crate with many internal modules can still create compile-time coupling and unclear ownership. Do not split every file into its own crate; do enforce package boundaries around stable contracts and expensive/optional capabilities.

**Rule:** the core kernel must not depend on UI, provider implementations, a concrete database, Tauri, or a concrete executor.

### 4.2 Frontend owns agent state

If React or Tauri owns agent-loop state, retries, tool state, and session mutation, desktop and web behavior will diverge.

**Rule:** frontend renders durable server/engine state and sends commands. It may hold ephemeral view state only.

### 4.3 Network-facing web UI gets local machine authority

A remote user must not inherit the server host's unrestricted shell/filesystem authority.

**Rule:** every remote execution request must be bound to a tenant, user, workspace, executor, capability scope, and policy decision.

### 4.4 Approval is mistaken for sandbox

A human-approved command can still escape a workspace if the process has broad host privileges. Conversely, an OS sandbox does not decide whether a particular action requires user confirmation.

**Rule:** authorization and confinement are separate mandatory stages.

### 4.5 Plugin failure silently grants access

A security plugin that times out must not turn a pending decision into an allow. Advisory UI hooks may fail open; enforcement hooks must fail closed.

### 4.6 Common provider API erases real capability differences

A normalized request shape can hide differences in tool calling, cache control, reasoning, multimodal payloads, usage and retry semantics.

**Rule:** keep a canonical internal message model but preserve provider dialect adapters and capability flags.

### 4.7 One database model for desktop and hosted service

SQLite is appropriate for local-first and controlled single-node deployments; it should not be the default persistence layer for a multi-worker, multi-tenant service with concurrent distributed state changes.

**Rule:** local SQLite and remote PostgreSQL implement the same repository contracts, but remote execution uses server-owned PostgreSQL and object storage.

### 4.8 Feature count outruns verification

**Rule:** each capability needs conformance tests, cancellation tests, failure recovery, audit evidence and resource limits before being enabled by default.

## 5. Priority roadmap and exit gates

### Phase 0 — Architecture and contracts (no production feature work)

Deliverables:
- System context and container diagrams.
- Crate dependency graph and ownership map.
- Protocol specification and versioning policy.
- Canonical data/event schema.
- Threat model and capability model.
- Windows/desktop/server support matrix.
- Feature parity baseline and explicit not-in-scope list.

Exit gate: no unresolved ownership conflict between runtime, API, persistence, executor, UI and plugin layers.

### Phase 1 — Rust kernel and local vertical slice (P0)

Deliverables:
- Provider adapter interface and one streaming provider.
- Agent run/turn/step lifecycle.
- File tools, shell tool, tool registry and permission pipeline.
- A minimum enforceable Windows local isolation profile before untrusted shell execution is enabled; otherwise shell execution remains disabled or explicitly restricted to a documented trusted-local mode.
- Local workspace and SQLite persistence.
- Session resume, cancellation and event history.
- `agentd` local server and typed client.

Exit gate: an end-to-end coding task edits a sample project, streams output, executes tools only within the advertised enforcement boundary, persists history, can be cancelled and resumed, and passes deterministic integration tests.

### Phase 2 — Windows Desktop + Web shared client (P0)

Deliverables:
- Tauri 2 Windows shell.
- React/Vite web app.
- Shared TypeScript SDK and UI primitives.
- Same HTTP/WS protocol in both clients.
- Workspace, session, transcript, terminal, file tree and diff views.

Exit gate: create a session in one client and continue it in another without duplicated state or protocol-specific behavior.

### Phase 3 — Safe execution and Git workflow (P0/P1)

Deliverables:
- Hardened Windows executor and expanded sandbox strategy beyond the minimum M1 local profile.
- Container executor for remote/server operation.
- Git worktrees, diff snapshots, undo/redo.
- LSP and test/formatter/check adapters.
- Executor leases, resource limits, cleanup and audit.

Exit gate: adversarial filesystem/process tests and workspace concurrency tests pass on supported configurations.

### Phase 4 — Multi-agent and plugin platform (P1)

Deliverables:
- Agent profiles/roles and task DAG.
- Parent-child session coordination.
- Background workers and task recovery.
- MCP/ACP adapters.
- Skills and commands.
- WASM component plugin host and host capabilities.

Exit gate: parallel agent tasks complete, fail, cancel and resume deterministically; plugins cannot exceed declared capabilities.

### Phase 5 — Remote workspaces and web tenancy (P1)

Deliverables:
- Server auth and organization/user/device models.
- PostgreSQL repository implementation.
- Remote worker registration/heartbeat/lease.
- Docker execution and artifact storage.
- Browser-based backend selection and device revocation.

Exit gate: tenant isolation, auth, credential boundary, worker reconnection and quota tests pass.

### Phase 6 — Platform ecosystem (P2)

Deliverables:
- Automation scheduler and webhook dispatcher.
- Plugin registry, signatures and update flow.
- Advanced memory, voice and computer use.
- Evaluation dashboard and release telemetry.

Exit gate: each subsystem can be disabled independently without destabilizing the core coding workflow.

## 6. Upstream lessons to adopt — and not adopt blindly

| Upstream lesson | Adopt | Do not blindly copy |
|---|---|---|
| Goose operation/effect state machine | Composable operation steps, durable effects, cancellation cleanup | Alpha GDK APIs without a pinned compatibility layer |
| Codex app-server/exec-server | Separate controller from execution host, typed protocol, OS sandbox adapters | OpenAI-specific auth/product coupling or whole workspace complexity |
| Drift event replay | Monotonic sequence, cursor replay, resync, single engine API for local/remote UI | Any fail-open behavior for policy enforcement hooks |
| Drift WASM plugins | WIT interface and capability-limited host functions | Give a plugin filesystem/process/HTTP access by default |
| Agent Code library | Engine library independent of TUI/client | Assume Windows GUI parity or complete plugin marketplace without verification |
| OpenHands control plane | Multiple backends, Docker/remote workspaces, automation service boundary | Expose a host-local agent server to public networks without isolation and auth |
| OpenCode product UX | Goal workflow, worktrees, tool activity, polished desktop interaction | Treat UI maturity as proof of backend isolation or Rust portability |
| DeepSeek Harness composition | Dynamic profiles, slots/hooks and capability boundaries | Bind new core to a rapidly changing preview API |

## 7. Release acceptance policy

A milestone is not complete because the feature appears in the UI. Each feature must have:

1. Source-level implementation and owner.
2. API/event/schema contract.
3. Unit tests for deterministic logic.
4. Integration tests for subsystem boundaries.
5. Failure/cancellation/restart tests.
6. Security tests where the feature touches tools, credentials, networking, plugins or execution.
7. User-visible status and error behavior.
8. Documentation and migration notes.
9. Windows compatibility check where desktop/local execution is involved.

## 8. External sources

- [Goose releases](https://github.com/aaif-goose/goose/releases)
- [Codex releases](https://github.com/openai/codex/releases)
- [Drift engine rewrite](https://github.com/kylepelham/Drift/blob/master/docs/engine-rewrite.md)
- [Drift plugin WIT contract](https://github.com/kylepelham/Drift/blob/master/crates/drift-engine/wit/drift.wit)
- [Agent Code agent loop](https://github.com/avala-ai/agent-code/tree/main/docs)
- [OpenHands Agent Canvas architecture](https://github.com/OpenHands/OpenHands/blob/main/docs/architecture.md)
- [OpenHands SDK](https://github.com/OpenHands/software-agent-sdk)
- [OpenHands Automation](https://github.com/OpenHands/automation)
- [OpenCode releases and changelog](https://github.com/anomalyco/opencode/releases)
- [DeepSeek Harness releases](https://github.com/deepseek-ai/deepseek-harness/releases)
