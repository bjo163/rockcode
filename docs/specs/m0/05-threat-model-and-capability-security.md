# M0-05 — Threat Model & Capability Security

**Status:** Normative security design baseline; not a security certification.  
**Applies to:** Desktop, Web, CLI, Agent Server, providers, tools, plugins, browser service and workers.

## 1. Security objectives

1. Untrusted model output cannot authorize itself.
2. Untrusted repository/project content cannot override platform policy.
3. A permission decision cannot expand the sandbox.
4. A sandbox constrains actual OS/process access even when a model or tool is compromised.
5. Secrets are not exposed to browser JS, untrusted plugins, prompts, logs or unrelated workers.
6. Remote identities, workspaces, jobs and artifacts are tenant-scoped.
7. Every privileged operation has a traceable actor, target, policy decision, executor, sandbox profile and result.
8. Isolation failure is explicit; no silent downgrade.

## 2. Assets

| Asset | Security impact if compromised |
|---|---|
| Provider/API credentials | Model account theft, cost abuse, data exfiltration |
| User/device sessions | Unauthorized workspace and session access |
| Source repositories/workspaces | Code theft, tampering, supply-chain compromise |
| Host OS/filesystem/processes | Local machine compromise |
| Remote workers/containers | Lateral movement, cross-tenant access |
| Artifacts and logs | Leakage of source, tokens, secrets and proprietary data |
| Plugin registry/signing keys | Distributed malicious extension |
| Event stream and DB | Forged state, approval or result replay |
| Update/signing pipeline | Persistent code execution on client machines |

## 3. Adversary and trust assumptions

Threat actors:
- malicious or compromised repository contributor;
- prompt-injection content in README, issue, web page, package script or tool output;
- compromised LLM/provider response;
- malicious MCP server/plugin/skill;
- malicious or compromised remote worker;
- network attacker or stolen device token;
- cross-tenant authenticated user;
- compromised update/build dependency.

Assumptions:
- Model output is untrusted.
- Repository instructions are untrusted until user/project policy grants bounded trust; they never override platform policy.
- Browser pages and downloaded files are untrusted.
- Plugins execute arbitrary logic within their declared runtime model; WASM limits host access but does not prove code correctness.
- Local Desktop mode has access to a user's machine only through explicit host/executor capabilities.
- Administrator/root compromise of the host is outside the sandbox guarantee; report this clearly.

## 4. Threat register

| ID | Threat | Likelihood / impact | Primary controls | Release gate |
|---|---|---|---|---|
| TH-001 | Prompt injection asks agent to leak secrets or run destructive commands | High / High | Untrusted-content labeling, context provenance, tool policy, sandbox, approval, secret broker | P0 |
| TH-002 | Shell command escapes workspace via path traversal, symlink or junction | High / Critical | Canonical path checks, handle-based checks where possible, OS/container boundary, adversarial tests | P0 |
| TH-003 | Model emits malformed or oversized tool arguments | High / Medium | JSON Schema validation, byte/depth limits, bounded parsing, normalized errors | P0 |
| TH-004 | Permission prompt is mistaken for actual confinement | Medium / Critical | Independent sandbox enforcement, capability report, fail-closed behavior | P0 |
| TH-005 | Web XSS steals local bootstrap/provider/worker secrets | Medium / Critical | Never expose secrets to JS, CSP, secure cookies, strict origin, short-lived scoped tokens | P0 |
| TH-006 | Local API is bound publicly or accepts unauthenticated requests | Medium / Critical | Loopback-only default, auth bootstrap, bind verification, explicit remote mode | P0 |
| TH-007 | Plugin gains ambient filesystem/process/network access | High / High | WASM component, capability host calls, resource quotas, revocation, signed digest | P1 |
| TH-008 | MCP server returns malicious tool output or abuses OAuth scope | High / High | Central policy, OAuth scope review, server allowlist, no implicit grants, output limits | P0/P1 |
| TH-009 | Stolen worker token replays jobs or impersonates a worker | Medium / High | Short-lived job token, nonce/lease, revocation, mutual identity, replay rejection | P1 |
| TH-010 | Tenant A accesses Tenant B's session/artifact/workspace | Medium / Critical | Tenant-scoped queries, authorization tests, storage key scoping, separate worker leases | P1 |
| TH-011 | Secret enters event, trace, artifact or provider prompt | Medium / High | Secret classification, redaction, allowlisted environment injection, log schema tests | P0 |
| TH-012 | Browser tool accesses internal network/metadata endpoint | High / High | Egress policy, DNS/IP validation, private/link-local block, redirect revalidation, isolated browser | P1 |
| TH-013 | Event replay or duplicate command creates repeated side effects | Medium / High | Idempotency key, per-aggregate sequence, transactional event/outcome, dedupe | P0 |
| TH-014 | Malicious update/package dependency compromises app | Medium / Critical | Signed releases, provenance, lockfile, dependency/advisory/license scans | P1 |
| TH-015 | Native process/plugin hook failure accidentally allows action | Medium / High | Enforcement hooks fail closed, explicit typed result, timeout contract | P0 |
| TH-016 | Resource exhaustion from subagents, PTYs, output or WebSocket clients | High / Medium | Budgets, concurrency caps, frame/output limits, quotas, cancellation | P0 |
| TH-017 | Worker disconnect causes duplicate execution or false success | Medium / High | Durable lease, heartbeat, reconciliation, idempotency, lost state | P0 |
| TH-018 | Credential copied into child process inherited environment | Medium / Critical | Sanitized environment, explicit named secret injection, child process scope | P0 |

## 5. Capability model

Capabilities are named, typed, scoped and revocable. No wildcard capability is implicit.

| Capability | Minimum scope |
|---|---|
| workspace.read | One workspace and approved path roots |
| workspace.write | One workspace, write path allowlist and mutation mode |
| process.exec | Executor ID, executable/command policy, working directory, timeout and sandbox profile |
| process.pty | One job/session PTY ID |
| network.connect | Protocol, hostname/IP policy, port and response limits |
| browser.control | One browser profile/session and allowed navigation policy |
| artifact.read | Named artifact IDs or workspace scope |
| artifact.write | One workspace/session and bounded artifact quota |
| session.read | Named session or task-owned session scope |
| session.annotate | Named session and permitted annotation event types |
| secrets.use | Named secret reference and approved destination/injection operation; secret bytes are not returned to WASM plugins |
| notifications.send | User-visible notifications only |
| git.mutate | One workspace/repository and allowed Git operations |

A capability grant includes:
- subject/plugin/tool ID and version/digest;
- capability name;
- resource scope;
- actor who granted it;
- issued/expiry time;
- policy revision;
- revocation state;
- audit reference.

## 6. Authorization and enforcement sequence

~~~mermaid
flowchart TD
  Call[Tool or plugin request] --> Identity[Resolve actor, org, session, workspace]
  Identity --> Schema[Validate tool schema and limits]
  Schema --> Classify[Classify action, target and side effects]
  Classify --> Policy[Evaluate policy: allow / ask / deny]
  Policy -->|deny| Block[Persist denial + explain]
  Policy -->|ask| Approval[Persist approval and wait]
  Approval -->|denied/expired| Block
  Approval -->|approved| Sandbox
  Policy -->|allow| Sandbox[Select required sandbox profile]
  Sandbox --> Available{Enforcement available?}
  Available -->|no| Fail[Fail closed or explicit user-selected weaker mode]
  Available -->|yes| Grant[Build scoped process/plugin capabilities]
  Grant --> Execute[Execute under executor lease]
  Execute --> Audit[Persist result, audit and event]
  Fail --> Audit
  Block --> Audit
~~~

The UI may display this sequence but cannot skip or override a server stage.

## 7. Sandbox profiles

Initial profile names:
- **inspect_only:** read-only workspace and no arbitrary process execution.
- **workspace_write:** write access limited to the selected workspace/worktree.
- **network_restricted:** explicit egress policy for known destinations.
- **container_isolated:** container/VM boundary with explicit mounts and resource limits.
- **trusted_local:** clearly named opt-in mode where execution has user-level host authority; never label it sandboxed.
- **untrusted_project:** requires an enforceable isolated executor or remains inspect-only.

An effective sandbox report lists filesystem, process, network, identity, resource and platform controls. It must not return a single opaque sandboxed=true field as proof.

### 7.1 Fail-closed rule

If a requested profile cannot be enforced:
- reject the operation with SANDBOX_UNAVAILABLE; or
- ask the user to select an explicitly named weaker mode, explaining the exact reduced protections.

Never silently downgrade to trusted_local. Permission allow never widens the sandbox.

## 8. Windows-specific security baseline

Before enabling arbitrary project shell commands in default Windows Desktop mode:
- define a supported Windows build matrix;
- prove workspace path confinement against symlinks/junctions, reparse points, case-insensitive paths and TOCTOU races;
- constrain child process trees and resource use;
- sanitize inherited environment;
- prevent access to provider credential stores and unrelated user data;
- define network egress enforcement or clearly disclose its absence;
- test process termination and cleanup after app/daemon crash.

Potential controls include container/VM/WSL2 isolation, restricted tokens, Job Objects, ACLs and AppContainer/network restrictions, but these are candidates until implemented and adversarially tested. Do not claim equivalence to Codex or another upstream sandbox from an architecture diagram.

## 9. Plugin and MCP security

- WASM/WIT plugin host receives capability-specific functions only.
- No direct WASI filesystem/network by default; host mediates scoped access.
- Fuel/instruction, memory, table, stack, wall-time and concurrent-instance limits.
- Signed/pinned manifests and digest verification.
- Install, enable and grant are separate user decisions.
- Security hooks fail closed. Advisory hooks may fail open only if they cannot affect authorization, data access or execution outcome.
- Revocation kills active instances and invalidates capability tokens.
- MCP tools use the same schema/policy/sandbox/audit pipeline as built-in tools.
- OAuth scopes and returned resources are shown before connection.

## 10. Browser and network security

- Browser profile is isolated per workspace/session according to policy.
- Block private, loopback, link-local and cloud metadata destinations by default unless an explicit policy allows them.
- Re-resolve and validate DNS/IP after redirects and before connection to reduce DNS rebinding/SSRF risk.
- Bound downloads, response bodies, screenshots and page text.
- Cookie/session sharing with user browser is opt-in and separated from default isolated profiles.
- Browser extensions and downloads require explicit policy; downloaded executable files are untrusted.

## 11. Secret and data handling

- Local secrets use OS credential vault/DPAPI-backed storage where supported.
- Remote secrets remain server-side encrypted with rotation and access audit.
- The web client receives references and statuses, not plaintext credentials.
- A provider adapter obtains a secret only when making the scoped provider call.
- Tool child processes receive only explicitly selected named secret references.
- Redact secrets from prompts, events, artifacts, crash reports, logs and traces.
- Telemetry is off or privacy-preserving by default; prompt/file contents are not product analytics.
- Credential export requires explicit user action and separate confirmation.

## 12. Security testing requirements

- Prompt-injection corpus with malicious README, issue, webpage, tool output and package lifecycle scripts.
- Windows path/junction/symlink/UNC and case-folding traversal corpus.
- Shell escape, environment leak and child process persistence tests.
- Web XSS/CSP/CORS/session theft checks.
- Unauthorized session/artifact/worker cross-tenant tests.
- Plugin host capability escape and resource exhaustion tests.
- Worker replay, stale lease, duplicate job and revoked-token tests.
- SSRF/DNS rebinding/redirect/private-IP browser tests.
- Secret canary test through prompts, tool output, logs, events, artifacts and telemetry.
- Dependency, license, signature and update provenance checks.

## 13. M0 acceptance evidence

A threat owner is assigned to every P0/P1 row; every P0 boundary has an enforcement mechanism and failure behavior; Windows sandbox claims remain explicitly unverified until adversarial POC tests pass.
