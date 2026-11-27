# Todo

Single work queue for this repository. Roadmap sections in docs/specs/ and GitHub
issues are reference mirrors, not the queue. Keep main releasable: every completed
item must have code evidence and a passing required CI run.

| # | Milestone | Item | Status | Notes |
|---|---|---|---|---|
| 1 | M1.1 | Contract-validation tooling | complete | Merged to main; PR validation passed. |
| 2 | M1.2 | Authenticated loopback server bootstrap and typed health/capabilities | complete | PR #1 merged to main; versioned v1 routes, exact bearer check, loopback-only bind and typed SDK. |
| 3 | M1.2 | Deterministic runtime lifecycle tests | complete | Success and pre-cancelled run tests pass on Ubuntu and Windows. |
| 4 | M1.3 | Provider-neutral completion contracts | complete | PR #2 merged to main; canonical message roles, completion request and response types in rockcode-kernel. |
| 5 | M1.3 | OpenAI-compatible gateway adapter | complete | PR #2 merged; HTTPS via rustls, loopback-only plaintext, env-configured endpoint/key/model, bounded timeout and response parsing. |
| 6 | M1.4 | Structured tool-calling loop | queued | Normalize tool calls, bounded turns, cancellation, durable events and explicit approval boundary. |
| 7 | M1.5 | Workspace file tools and policy approvals | queued | Read/list first; write/patch behind policy approval and path/symlink containment. |
| 8 | M1.6 | Sessions, persistence and event replay | queued | SQLite local-first, migrations, monotonic sequence and reconnect/resync. |
| 9 | M1.7 | Windows sandbox and isolated workers | queued | No arbitrary shell before enforced isolation and adversarial tests. |
| 10 | M1.8 | Tauri Desktop + React workbench | queued | Windows-first shell, project picker, transcript, task activity, diff, terminal and settings. |

## Acceptance rules

- Each task is small, testable, and committed with its implementation and tests.
- A task is not complete because a specification or placeholder crate exists.
- Rust CI must pass on Ubuntu and Windows; contract validation must pass when contracts/docs change.
- cargo deny must run as a strict required gate. reqwest is allowed only in the provider and SDK boundaries; the architecture test enforces that rule.
- Never merge a red or unverified change into main.
