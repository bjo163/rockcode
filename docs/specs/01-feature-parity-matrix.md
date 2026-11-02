# 01 — Feature Parity Matrix

**Project:** RockCode — Rust-first Autonomous Coding Agent Platform  
**Snapshot:** 2 November 2026  
**Purpose:** Compare upstream projects as evidence sources for a new Windows Desktop + Web + CLI coding-agent platform.

## 1. How to read this matrix

This is a source/documentation audit, not a claim that every repository was built, executed, or security-tested during this review.

| Mark | Meaning |
|---|---|
| **●** | First-party capability is documented and identifiable in repository code/docs. |
| **◐** | Partial, platform-limited, experimental, or implemented in an adjacent component/repository. |
| **◇** | Available primarily by integrating another agent, backend, protocol, or external service. |
| **—** | Not a primary capability or not evidenced in the inspected repository. |
| **?** | Needs a dedicated source/test check before relying on it. |

Abbreviations: **DSH** = DeepSeek Harness; **OC** = OpenCode; **G** = Goose; **CX** = Codex; **DR** = Drift; **AC** = Agent Code; **OH** = OpenHands (Canvas + Software Agent SDK + Automation where relevant).

A dot means evidence was found; it does not mean equal maturity, quality, security, or production suitability.

## 2. Upstream snapshot

| Project | Upstream / canonical repository | Latest release evidence at snapshot | Primary implementation | License noted in inspected sources | Status / caution |
|---|---|---|---|---|---|
| DeepSeek Harness | [deepseek-ai/deepseek-harness](https://github.com/deepseek-ai/deepseek-harness) | `v0.2.1-rc.5` dated 30 Oct; release page also lists an alpha train | TypeScript / Node | Verify repository `LICENSE` before adoption | Developer-preview maturity; treat APIs as volatile |
| OpenCode | [anomalyco/opencode](https://github.com/anomalyco/opencode) | `v2.2.5` dated 30 Oct | TypeScript / Bun; Solid-based app surfaces | Verify repository `LICENSE` before adoption | Strong product reference; not a Rust engine |
| Goose | [aaif-goose/goose](https://github.com/aaif-goose/goose) | `v1.57.0` dated 28 Oct | Rust core + Electron/React desktop | Apache-2.0 | Active Rust agent ecosystem; extensions still require review |
| Codex | [openai/codex](https://github.com/openai/codex) | `0.166.0` stable dated 29 Oct; `0.167.0-alpha.2` dated 1 Nov | Rust | Apache-2.0 | Very large, fast-moving workspace; client products are not all in this repository |
| Drift | [kylepelham/Drift](https://github.com/kylepelham/Drift) | `v2.4.5` dated 30 Oct | Rust + Tauri 2 + TypeScript/SolidJS | MIT | Windows-first local/remote product; small adoption footprint; inspect security and support matrix carefully |
| Agent Code | [avala-ai/agent-code](https://github.com/avala-ai/agent-code) | `v0.30.0` dated 29 Jul is the latest release found in this review | Rust + Flutter/Dart | MIT | Strong compact engine reference; desktop client platform coverage is uneven |
| OpenHands Canvas | [OpenHands/OpenHands](https://github.com/OpenHands/OpenHands) | `v1.34.0` dated 27 Oct | React + TypeScript + Electron | MIT | Frontend/control plane; runtime is elsewhere |
| OpenHands SDK / Agent Server | [OpenHands/software-agent-sdk](https://github.com/OpenHands/software-agent-sdk) | `v1.59.0` dated 28 Oct | Python SDK/server + TypeScript client | MIT | Separate backend repository; canonical agent/server behavior |
| OpenHands Automation | [OpenHands/automation](https://github.com/OpenHands/automation) | `v1.26.1` dated 28 Oct | Python service | Verify repository `LICENSE` before adoption | Separate scheduler/webhook/dispatch subsystem |

All nine repositories were reachable under the canonical owner/repository names checked on 2 November 2026 and were not reported archived by the repository lookup. This is not a guarantee of long-term maintenance. Release numbers above should be rechecked before dependency selection.

## 3. Capability matrix

### 3.1 Product surfaces and runtime topology

| Capability | DSH | OC | G | CX | DR | AC | OH |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Native Rust agent runtime | — | — | ● | ● | ● | ● | — |
| Reusable headless engine / server | ● | ● | ◐ | ● | ● | ● | ● |
| Windows desktop product | ◐ | ● | ● | ◐ | ● | ◐ | ◐ |
| macOS desktop product | ● | ● | ● | ◐ | ◐ | ● | ◐ |
| Linux desktop product | ● | ● | ● | ◐ | ◐ | ● | ◐ |
| Browser-based Web client | ● | ● | ◇ | ◐ | ● | ● | ● |
| First-party CLI / TUI | ● | ● | ● | ● | ◐ | ● | ◐ |
| Shared engine across multiple clients | ● | ● | ◐ | ● | ● | ● | ● |
| Remote agent/backend selection | ◐ | ● | ◐ | ● | ● | ◐ | ● |
| Self-hosted server/control-plane pattern | ● | ● | ◐ | ● | ◐ | ◐ | ● |

**Interpretation:** Drift, Agent Code, Goose, and Codex are the best Rust-side references. OpenHands is the strongest reference for the Web control plane and remote execution topology. Do not count OpenHands Canvas as the agent engine: its canonical server/SDK lives in `software-agent-sdk`, while scheduling and webhook dispatch live in `automation`.

### 3.2 Agent runtime and orchestration

| Capability | DSH | OC | G | CX | DR | AC | OH |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Explicit agent loop / state machine | ● | ● | ● | ● | ● | ● | ● |
| Typed lifecycle events | ● | ● | ● | ● | ● | ● | ● |
| Streaming model output | ● | ● | ● | ● | ● | ● | ● |
| Cancellation / interruption | ● | ● | ● | ● | ● | ● | ● |
| Subagent delegation | ● | ● | ● | ● | ● | ● | ● |
| Parallel subagents / concurrency controls | ◐ | ● | ● | ● | ● | ● | ● |
| Background task / worker lifecycle | ◐ | ● | ● | ● | ● | ● | ● |
| Agent roles / specialized profiles | ● | ● | ● | ● | ● | ● | ● |
| Goal-driven continuation / reviewer loop | ◐ | ● | ◐ | ● | ◐ | ◐ | ◐ |
| Human-in-the-loop questions / approvals | ● | ● | ● | ● | ● | ● | ● |
| Budget / token / cost controls | ◐ | ● | ● | ● | ◐ | ● | ● |
| Context compaction / summarization | ● | ● | ● | ● | ● | ● | ● |
| Persistent memory beyond a single thread | ◐ | ◐ | ◐ | ● | ◐ | ◐ | ● |
| Agent-to-agent messaging / shared task board | ● | ◐ | ◐ | ● | ◐ | ◐ | ● |

**Interpretation:** Goose has especially valuable composable operation/state-machine concepts. Codex has deep lifecycle, executor, thread and policy boundaries. DeepSeek Harness is valuable for dynamic composition and agent teams, but its preview status and rapidly changing profile/session contracts must be respected.

### 3.3 Coding tools and developer workflow

| Capability | DSH | OC | G | CX | DR | AC | OH |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| File read/write/edit/patch | ● | ● | ● | ● | ● | ● | ● |
| Search / glob / grep | ● | ● | ● | ● | ● | ● | ● |
| Shell / terminal / PTY | ● | ● | ● | ● | ● | ● | ● |
| Long-running process management | ◐ | ● | ● | ● | ● | ● | ● |
| Git operations | ● | ● | ● | ● | ● | ● | ● |
| Git worktree isolation | ● | ● | ● | ● | ● | ● | ◐ |
| Diff / undo / rewind | ◐ | ● | ◐ | ● | ● | ● | ● |
| LSP diagnostics | ● | ● | ◐ | ● | ● | ● | ◐ |
| Formatters / test / lint hooks | ◐ | ● | ◐ | ● | ● | ◐ | ● |
| Notebook / structured document tools | ◐ | ◐ | ● | ◐ | ◐ | ● | ◐ |
| Code review / repository analysis | ◐ | ● | ◐ | ● | ● | ● | ● |
| Web search / fetch | ● | ● | ◐ | ◐ | ● | ● | ◐ |
| Browser automation | ◐ | ● | ◐ | ◐ | ● | ◐ | ● |
| Computer use / desktop control | ◐ | ◐ | ◐ | ◐ | ◐ | ◐ | ◐ |
| Image / multimodal input | ● | ● | ● | ● | ● | ● | ● |
| Image generation | ◐ | ● | ◐ | ◇ | ● | ◐ | ◐ |

**Interpretation:** Drift provides unusually deep browser-driver integration for a desktop coding agent. OpenHands offers the strongest visible browser/terminal/workspace control-plane experience. Zed is an additional UI/IDE reference, but is not part of the core nine-repository parity set.

### 3.4 Providers, protocols and extensions

| Capability | DSH | OC | G | CX | DR | AC | OH |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Multiple LLM providers | ● | ● | ● | ● | ● | ● | ● |
| OpenAI-compatible endpoints | ● | ● | ● | ◐ | ● | ● | ● |
| Local model support | ◐ | ◐ | ● | ◐ | ● | ◐ | ◐ |
| Provider capability metadata | ● | ● | ● | ● | ● | ◐ | ● |
| MCP client integration | ● | ● | ● | ● | ● | ● | ● |
| MCP server / tool exposure | ● | ◐ | ● | ● | ◐ | ◐ | ◐ |
| ACP integration | ● | ● | ● | ◐ | ● | ● | ● |
| Skills / reusable instruction packs | ● | ● | ● | ● | ● | ● | ● |
| Plugin / extension system | ● | ● | ● | ● | ● | ● | ● |
| Wasm sandboxed plugin runtime | ◐ | — | ◐ | ◐ | ● | — | ◐ |
| Plugin marketplace / registry | ◐ | ● | ● | ◐ | ● | ◐ | ● |
| Custom agents / prompts / commands | ● | ● | ● | ● | ● | ● | ● |
| Remote tool / connector integrations | ● | ● | ● | ● | ◐ | ● | ● |

**Interpretation:** Drift's WIT/WASM plugin boundary is the strongest direct inspiration for a capability-limited plugin host. Goose is the strongest Rust provider/MCP/SDK ecosystem reference. OpenHands is the strongest marketplace/control-plane integration reference.

### 3.5 Persistence, collaboration and operations

| Capability | DSH | OC | G | CX | DR | AC | OH |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Session persistence | ● | ● | ● | ● | ● | ● | ● |
| Session resume / fork | ● | ● | ● | ● | ● | ● | ● |
| Event cursor / replay recovery | ◐ | ● | ◐ | ● | ● | ◐ | ● |
| SQLite/local-first storage | ● | ◐ | ● | ● | ● | ● | ◐ |
| PostgreSQL/server persistence | ◐ | ● | ◇ | ◐ | — | — | ● |
| Multi-user / organization concepts | ◐ | ◐ | ◐ | ◐ | ◐ | — | ● |
| Remote execution workers | ◐ | ● | ◐ | ● | ◐ | ◐ | ● |
| Scheduled automation | ● | ◐ | ● | ◐ | ◐ | ● | ● |
| Webhook/event-trigger automation | ◐ | ◐ | ◐ | ◐ | ◐ | ◐ | ● |
| OpenTelemetry / tracing | ◐ | ● | ● | ● | ◐ | ◐ | ● |
| Evaluation / regression infrastructure | ◐ | ● | ◐ | ● | ◐ | ● | ● |
| Windows CI / packaging evidence | ◐ | ● | ● | ● | ● | ◐ | ◐ |

**Interpretation:** OpenHands provides the clearest reference for multi-backend server operations and automation separation. Codex is the strongest reference for execution server and policy subsystem depth. Drift is the strongest local-first desktop-to-remote-browser reference.

## 4. Reference ranking by target subsystem

| Target subsystem | Primary reference | Secondary reference | What to study |
|---|---|---|---|
| Rust Agent Kernel | Goose | Codex | State machine, operations/effects, provider contracts, subagent loop |
| Session/event engine | Drift | Codex, Goose | Monotonic sequence, cursor replay, resync, cancellation recovery |
| Execution and sandbox | Codex | Agent Code, OpenHands | OS isolation, process/PTY server, policy vs enforcement |
| Local Windows Desktop | Drift | Goose | Tauri host, local engine lifecycle, pairing, secure loopback |
| Web control plane | OpenHands Canvas | OpenCode | Backend selection, workspace UI, terminal/browser panels, APIs |
| Remote workers | Codex | OpenHands | Executor protocol, worker registration, reconnect, environment leases |
| Plugins | Drift | DeepSeek Harness, Goose | WIT capabilities, lifecycle, dynamic hooks, extension process isolation |
| Provider adapters/router | Goose | Drift, OpenHands | Native dialects, capability registry, auth, limits, pricing |
| Coding UX | OpenCode | Zed, Drift | Worktree, model picker, goal/task UX, review, shortcuts |
| Automation | OpenHands Automation | Goose | Schedules, webhooks, trigger dispatch, run history |

## 5. Parity target for the new platform

The new platform should not chase every feature immediately. The parity target is divided into three levels:

### P0 — Foundation parity

- Shared Rust engine for Desktop, Web, and CLI.
- Providers, streaming, tools, structured sessions, cancellation, and context compaction.
- Safe local filesystem and terminal execution.
- Permission policy, independent sandbox enforcement, audit trail.
- SQLite local persistence and crash-safe session recovery.
- HTTP API + WebSocket event stream with cursor replay.
- React Web UI and Tauri Windows Desktop consuming the same SDK/protocol.

### P1 — Advanced coding-agent parity

- Git worktrees, diff/undo, LSP, formatters, tests, repository review.
- Subagents, background workers, task graph, task budgets.
- MCP, ACP, skills, custom agents and commands.
- Browser automation and isolated browser sessions.
- Remote execution workers and remote workspace lifecycle.
- WASM plugin runtime with capability-scoped host interfaces.

### P2 — Platform/ecosystem parity

- Multi-user / organization tenancy, RBAC and server-side credential vault.
- Plugin registry and signed distribution.
- Scheduled tasks, webhooks, connectors and automation runs.
- Durable memory, team/agent message board, evaluation platform.
- Voice, advanced computer use, multimodal extensions, image generation.
- Fleet management, observability dashboards, billing/resource quotas.

## 6. Evidence and upstream links

- [DeepSeek Harness releases](https://github.com/deepseek-ai/deepseek-harness/releases) · [architecture docs](https://github.com/deepseek-ai/deepseek-harness/tree/master/docs)
- [OpenCode releases](https://github.com/anomalyco/opencode/releases) · [package/source tree](https://github.com/anomalyco/opencode/tree/dev/packages)
- [Goose releases](https://github.com/aaif-goose/goose/releases) · [agent state machine](https://github.com/aaif-goose/goose/tree/main/crates/goose-agent) · [security policy](https://github.com/aaif-goose/goose/blob/main/SECURITY.md)
- [Codex releases](https://github.com/openai/codex/releases) · [core](https://github.com/openai/codex/tree/main/codex-rs/core) · [app-server](https://github.com/openai/codex/tree/main/codex-rs/app-server) · [exec-server](https://github.com/openai/codex/tree/main/codex-rs/exec-server)
- [Drift releases](https://github.com/kylepelham/Drift/releases) · [engine](https://github.com/kylepelham/Drift/tree/master/crates/drift-engine) · [engine design](https://github.com/kylepelham/Drift/blob/master/docs/engine-rewrite.md) · [WIT plugin contract](https://github.com/kylepelham/Drift/blob/master/crates/drift-engine/wit/drift.wit)
- [Agent Code releases](https://github.com/avala-ai/agent-code/releases) · [Rust library](https://github.com/avala-ai/agent-code/tree/main/crates/lib) · [plugin contract docs](https://github.com/avala-ai/agent-code/tree/main/docs/extending)
- [OpenHands Canvas releases](https://github.com/OpenHands/OpenHands/releases) · [architecture](https://github.com/OpenHands/OpenHands/blob/main/docs/architecture.md)
- [OpenHands SDK releases](https://github.com/OpenHands/software-agent-sdk/releases) · [Agent Server](https://github.com/OpenHands/software-agent-sdk/tree/main/openhands-agent-server) · [SDK](https://github.com/OpenHands/software-agent-sdk/tree/main/openhands-sdk)
- [OpenHands Automation releases](https://github.com/OpenHands/automation/releases)

---

**Audit limitation:** The matrix records observed/documented capabilities. Before adopting code, inspect exact commit/license, run platform builds/tests, and execute independent security reviews for the selected modules.
