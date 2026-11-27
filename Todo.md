# Todo

Single work queue for this repository. Roadmap sections in docs/specs/ and GitHub
issues are reference mirrors, not the queue. Keep main releasable: every completed
item must have code evidence and a passing required CI run.

| # | Milestone | Item | Status | Notes |
|---|---|---|---|---|
| 1 | M1.1 | Contract-validation tooling | complete | Latest recorded main run: Contract Validation and Rust both passed on 2026-11-03. |
| 2 | M1.2 | Authenticated loopback server bootstrap and typed health/capabilities | in-review | Branch feat/m1-2-server-bootstrap-closeout; exact bearer check, fixed default port, loopback-only binding, SDK bearer support and protocol-aligned responses. |
| 3 | M1.2 | Deterministic runtime lifecycle tests | in-review | Fake provider, ordered lifecycle events and pre-cancelled run behavior; CI must confirm Linux and Windows. |
| 4 | M1.2 | Complete POC-01 vertical slice | queued | Wire kernel, runtime, server and SDK into one executable scenario with success, failure and cancellation evidence. |
| 5 | M1.3 | Provider adapter and structured model responses | queued | OpenAI-compatible first; secrets only from OS credential store/env, never UI storage or logs. |
| 6 | M1.4 | Workspace tools and policy approvals | queued | File read/write, diff, patch, terminal and Git; writes/commands require explicit approval and sandbox enforcement. |
| 7 | M1.5 | Sessions, persistence and event replay | queued | SQLite local-first, versioned migrations, monotonic event sequence and reconnect/resync. |
| 8 | M1.6 | Tauri Desktop + React workbench | queued | Windows-first shell, project picker, transcript, task activity, diff viewer, terminal and settings. |
| 9 | M1.7 | Windows sandbox and release hardening | queued | No arbitrary shell before isolation, red-team tests, installer, upgrades and release evidence. |

## Acceptance rules

- Each task is small, testable, and committed with its implementation and tests.
- A task is not complete because a specification or placeholder crate exists.
- Rust CI must pass on Ubuntu and Windows; contract validation must pass when contracts/docs change.
- cargo deny must not be silently ignored; resolve the policy file and make it a required gate before production release.
- Never merge a red or unverified change into main.
