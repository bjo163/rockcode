# RockCode — Architecture Specification Pack

Snapshot date: 2 November 2026

This folder contains five coordinated planning documents for RockCode, a new Rust-first coding-agent platform targeting Windows Desktop, Web, CLI, local execution, remote execution, multi-agent orchestration and extensibility.

## Documents

1. `01-feature-parity-matrix.md` — evidence-oriented comparison of nine upstream repositories and target parity tiers.
2. `02-gap-analysis.md` — missing capabilities, risks, priorities, upstream lessons and milestone exit gates.
3. `03-final-architecture-specification.md` — final technical architecture baseline, module boundaries, domain model, protocols, security, persistence, execution, plugins, testing and milestones.
4. `04-ui-design-system.md` — RockCode visual identity and interaction design: industrial grunge, manga linework, funky Gen-Z, Moonframe skin strategy, workbench layout, screens, motion, accessibility and visual QA.
5. `05-ui-package-architecture.md` — Moonframe consumption and RockCode-specific UI package boundaries, component inventory, workbench composition, editor/terminal adapters, API contracts and UI milestones.

## M0 architecture and contract baseline

The detailed M0 package is in [`m0/README.md`](m0/README.md). It includes eight contract documents and machine-readable API/event/domain/security schemas. Its authored state is not equivalent to a validated implementation: validators and proof-of-concept gates remain outstanding.

## Governing decisions

- Build a new platform; do not merge or fork whole upstream repositories as the default approach.
- One reusable Rust engine powers Desktop, Web and CLI.
- Tauri 2 + React 19 for Windows-first Desktop; React 19 + Vite for Web; Rust CLI/TUI.
- Versioned REST + WebSocket protocol with durable event cursors and replay.
- SQLite for local-first use; PostgreSQL and object storage for remote multi-user use.
- Local, container and remote workers are distinct executor implementations.
- Permission decisions and OS/container sandbox enforcement are separate mandatory layers.
- WASM/WIT plugins are capability-scoped; skills remain declarative.
- OpenHands is a control-plane and remote execution reference, not the Rust agent core.
- [Moonframe](https://github.com/bjo163/moonframe) is the canonical UI design system; RockCode builds product-specific UI packages on top of it and must not recreate its tokens, theme runtime or base primitives.
- Default RockCode visual direction: Moonframe `moonpunk` with original RockCode industrial-engineering brand assets; `mooncrash` remains a complete alternative skin rather than a mixed skin recipe.
- The UI package stack is shared by Desktop and Web; agent state and authorization remain backend-owned.

## Status and limits

These are architecture-planning artifacts, not implementation commits. The Moonframe runtime is a separate existing repository; RockCode UI integration and consumer compatibility still require a pinned-dependency smoke test. No source repository was modified, no code was forked, and no upstream build/test/security suite was run as part of creating these specifications. Platform and license claims must be revalidated at the exact source commit before code adoption.
