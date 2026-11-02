# RockCode — Architecture Specification Pack

Snapshot date: 2 November 2026

This folder contains three coordinated planning documents for RockCode, a new Rust-first coding-agent platform targeting Windows Desktop, Web, CLI, local execution, remote execution, multi-agent orchestration and extensibility.

## Documents

1. `01-feature-parity-matrix.md` — evidence-oriented comparison of nine upstream repositories and target parity tiers.
2. `02-gap-analysis.md` — missing capabilities, risks, priorities, upstream lessons and milestone exit gates.
3. `03-final-architecture-specification.md` — final technical architecture baseline, module boundaries, domain model, protocols, security, persistence, execution, plugin model, UI, testing and milestones.

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

## Status and limits

These are architecture-planning artifacts, not implementation commits. No source repository was modified, no code was forked, and no upstream build/test/security suite was run as part of creating these specifications. Platform and license claims must be revalidated at the exact source commit before code adoption.
