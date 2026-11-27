# Todo

Single work queue for this repository. Roadmap sections in docs/specs/ and GitHub
issues are reference mirrors, not the queue. Keep main releasable: every completed
item must have code evidence and a passing required CI run.

| # | Milestone | Item | Status | Notes |
|---|---|---|---|---|
| 1 | M1.1 | Contract-validation tooling | complete | Strict contract validator merged to main. |
| 2 | M1.2 | Authenticated loopback server bootstrap and typed health/capabilities | complete | PR #1 merged; versioned v1 routes, exact bearer check, loopback-only bind and typed SDK. |
| 3 | M1.2 | Deterministic runtime lifecycle tests | complete | Success and pre-cancelled run tests pass on Ubuntu and Windows. |
| 4 | M1.3 | Provider-neutral completion contracts | complete | PR #2 merged; canonical message roles, completion request and response types. |
| 5 | M1.3 | OpenAI-compatible gateway adapter | complete | PR #2 merged; HTTPS via native roots, loopback-only plaintext, env configuration, 90s timeout and response parsing. |
| 6 | M1.4 | Structured tool-call types and OpenAI-compatible wire mapping | in-review | Tool definitions, IDs, JSON argument validation, assistant tool-call and tool-result messages. |
| 7 | M1.4 | Bounded runtime orchestration and policy decisions | in-review | Max 16 turns, cancellation, lifecycle events; tools execute only on explicit Allow, default is approval-required. |
| 8 | M1.5 | Approval persistence and resumption | queued | Persist approval requests and resume exactly the paused tool call after approval. |
| 9 | M1.6 | Workspace file tools and path policy | queued | Read/list first; writes and patches behind approval, workspace containment and symlink checks. |
| 10 | M1.7 | Sessions, SQLite persistence and event replay | queued | Versioned migrations, monotonic sequence, reconnection and resync. |
| 11 | M1.8 | Windows sandbox and isolated workers | queued | No arbitrary shell before enforced OS/container isolation and adversarial tests. |
| 12 | M1.9 | Tauri Desktop + React workbench | queued | Windows-first UI, project picker, transcript, task activity, diff, terminal and settings. |

## Acceptance rules

- Each task is small, testable, and committed with its implementation and tests.
- A task is not complete because a specification or placeholder crate exists.
- Rust CI must pass on Ubuntu and Windows; contract validation must pass when contracts/docs change.
- cargo deny is a strict pinned required gate; do not suppress advisories, source checks or license findings.
- Never merge a red or unverified change into main.
