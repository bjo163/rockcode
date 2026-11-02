# RockCode M0 — Architecture & Contract Baseline

**Status:** M0 specification package — ready for proof-of-concept planning; source implementation has not started.  
**Baseline date:** 2 November 2026  
**Governing documents:** [Final Architecture Specification](../03-final-architecture-specification.md), [UI Design System](../04-ui-design-system.md), [UI Package Architecture](../05-ui-package-architecture.md).

## Purpose

M0 turns the approved architecture direction into versioned contracts that engineers and coding agents can implement without inventing incompatible ownership, protocol, security, or UI behavior.

M0 does not create application source code, build scripts, Cargo crates, database migrations, or UI components. It freezes the smallest useful set of architectural boundaries and defines the proof-of-concept gates that must pass before M1 implementation.

## Contract documents

1. [System Context & Deployment Topology](01-system-context-and-deployment.md) — products, services, trust boundaries, deployment modes, and request flows.
2. [Rust Crate Boundaries & Dependency Rules](02-rust-crate-boundaries.md) — ownership map, initial workspace crates, optional modules, dependency graph and forbidden dependencies.
3. [API, Session & Event Protocol](03-api-session-event-protocol.md) — REST contract, WebSocket/AsyncAPI contract, idempotency, replay, compatibility and error model.
4. [Domain Model & Event Invariants](04-domain-model-and-event-invariants.md) — canonical entities, lifecycle states, invariants, event ownership and persistence rules.
5. [Threat Model & Capability Security](05-threat-model-and-capability-security.md) — trust boundaries, threat register, authorization, sandbox, secret, plugin, browser and remote-worker controls.
6. [Platform & Support Matrix](06-platform-support-matrix.md) — Windows, Linux server/worker, macOS, browser support and release evidence.
7. [Moonframe Consumption Contract](07-moonframe-consumption-contract.md) — exact dependency pin, package consumption, theme ownership and consumer smoke gates.
8. [M0 Validation & Proof-of-Concept Gates](08-validation-and-poc-gates.md) — decision closure, required experiments, acceptance evidence and M1 entry criteria.

## Machine-readable contracts

- [OpenAPI v1](../../../contracts/openapi/v1/openapi.yaml)
- [AsyncAPI v1 session channel](../../../contracts/asyncapi/v1/session-events.yaml)
- [Event envelope JSON Schema](../../../contracts/json-schema/v1/event-envelope.schema.json)
- [Session JSON Schema](../../../contracts/json-schema/v1/session.schema.json)
- [Approval request JSON Schema](../../../contracts/json-schema/v1/approval-request.schema.json)
- [Plugin manifest JSON Schema](../../../contracts/json-schema/v1/plugin-manifest.schema.json)
- [Executor capability report JSON Schema](../../../contracts/json-schema/v1/executor-capability-report.schema.json)

## Normative precedence

If two documents conflict, use this order:

1. Machine-readable protocol/schema files for wire/persisted shape.
2. M0 contract documents for ownership, trust boundary, invariants and support.
3. The Final Architecture Specification for overall architecture.
4. UI-specific specifications for visual, package and interaction behavior.
5. README/roadmap summaries.

A contradiction must be fixed at the owning source and linked documents updated in the same change. Do not silently choose whichever document is easiest to implement.

## Governing decisions

- One reusable Rust engine serves Desktop, Web, and CLI.
- Local Desktop mode uses a supervised local `agentd` server bound to loopback; the UI is a client, not the engine.
- Remote mode separates API/control plane from execution workers.
- HTTP handles resource operations and commands; WebSocket carries durable live events and interactive replies. SSE may be added as a read-only transport without changing event identity.
- Per-session event sequence is monotonic. Clients replay from a committed cursor or hydrate/resync.
- Permission policy and OS/container sandbox enforcement are independent.
- A requested isolation level may never be silently downgraded.
- SQLite is the local-first persistence adapter; PostgreSQL is the remote multi-user authority.
- Moonframe is the only base UI theme/token/primitive owner. RockCode owns product compositions and engineering-specific widgets.
- WASM/WIT plugins receive explicit, revocable capabilities only.
- Build new code; upstream projects are design/source references, not code to merge wholesale.

## Status

| Area | M0 status |
|---|---|
| Overall product architecture | Defined in parent specification |
| System/deployment topology | This package |
| Crate ownership/dependency direction | This package |
| REST/WS/event contracts | This package + machine-readable schemas |
| Domain/event invariants | This package + JSON Schemas |
| Threat model/security boundaries | This package |
| Platform support strategy | This package |
| Moonframe consumption boundary | This package |
| POC and M1 exit criteria | This package |
| Source implementation | **Not started** |

## Change control

Any change to an API route, event name, event schema, entity identity, executor capability, security boundary, crate ownership or Moonframe dependency pin requires:
- a concise rationale;
- impact on existing clients/data;
- compatibility/migration plan;
- tests or POC evidence appropriate to the change;
- updated owning contract and links.

Do not update generated clients or production source code before the owning contract change is reviewed.
