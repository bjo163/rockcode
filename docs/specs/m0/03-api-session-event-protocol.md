# M0-03 — API, Session & Event Protocol

**Status:** Normative v1 contract for the first implementation slice.  
**Machine-readable authority:** [OpenAPI v1](../../../contracts/openapi/v1/openapi.yaml), [AsyncAPI v1](../../../contracts/asyncapi/v1/session-events.yaml), [event schema](../../../contracts/json-schema/v1/event-envelope.schema.json).

## 1. Protocol principles

- Every client uses one public protocol: Desktop, Web, CLI, tests and future automation.
- Agent logic is invoked through use cases; clients cannot call internal runtime functions.
- HTTP is the resource/command interface.
- WebSocket is the bidirectional interactive event stream.
- SSE is optional and read-only; event IDs and cursor semantics remain identical.
- Public HTTP routes use `/v1`.
- JSON is UTF-8; timestamps are RFC 3339 UTC; IDs are opaque strings with stable prefixes.
- Every mutating command accepts an idempotency key.
- The server is authoritative for session/run/task/approval/executor state.
- Unknown additive fields and unknown future event types must not crash clients; protocol major-version incompatibility must be explicit.

## 2. Base URL and authentication

### Local Desktop
- Tauri host starts `agentd` on loopback.
- A fresh short-lived bootstrap secret is delivered over protected IPC/inherited handle.
- Never place bootstrap secrets in command-line arguments, URLs, application logs or browser localStorage.

### Local browser development
- User pairs explicitly with the local server.
- Enforce exact allowed origins and a short-lived browser session.
- Do not enable wildcard CORS with credentials.

### Remote
- TLS is mandatory.
- Use user/device authentication; session token scopes are enforced on every route and event subscription.
- Rotate/revoke device tokens; close open WebSocket/SSE connections after revocation.
- Provider and worker credentials are not returned by API resources.

### Headers
- `Authorization: Bearer <scoped-token>` for authenticated calls.
- `Idempotency-Key: <opaque-key>` for mutating requests.
- `X-Request-Id` may be supplied for correlation; server creates one if absent.
- Do not use secrets in query parameters.

## 3. Resource and command routes

All routes below are `/v1` relative. OpenAPI defines the concrete shapes.

| Method | Route | Purpose | Idempotent |
|---|---|---|---|
| GET | `/health/live` | Process liveness | Yes |
| GET | `/health/ready` | Dependency/readiness state | Yes |
| GET | `/capabilities` | Protocol, feature and executor capability discovery | Yes |
| GET | `/models` | Available model metadata (never credentials) | Yes |
| GET | `/tools` | Tool catalog and required grants | Yes |
| GET | `/workspaces` | List authorized workspaces | Yes |
| POST | `/workspaces` | Create workspace metadata/provisioning request | Key required |
| GET | `/workspaces/{workspace_id}` | Get workspace | Yes |
| POST | `/workspaces/{workspace_id}/close` | Close/release workspace | Key required |
| GET | `/executors` | List executor capabilities visible to actor | Yes |
| GET | `/sessions` | List authorized sessions | Yes |
| POST | `/sessions` | Create a session bound to workspace/profile/executor | Key required |
| GET | `/sessions/{session_id}` | Get session and current durable state | Yes |
| POST | `/sessions/{session_id}/turns` | Submit a user/automation turn | Key required |
| POST | `/sessions/{session_id}/cancel` | Cancel active run | Key required |
| POST | `/sessions/{session_id}/steer` | Add steering input to an active run | Key required |
| POST | `/sessions/{session_id}/resume` | Resume a recoverable run/session | Key required |
| GET | `/sessions/{session_id}/events` | WebSocket upgrade for replay/live events | N/A |
| GET | `/sessions/{session_id}/snapshot` | Hydration snapshot and cursor | Yes |
| POST | `/approvals/{approval_id}/decision` | Approve/deny with actor identity | Key required |
| POST | `/questions/{question_id}/answer` | Answer a persisted agent question | Key required |
| GET | `/artifacts/{artifact_id}` | Metadata/authorized artifact download URL | Yes |
| GET | `/sessions/{session_id}/tasks` | Task/subagent hierarchy | Yes |
| GET | `/sessions/{session_id}/audit` | Authorized policy/tool decision trail | Yes |

A first release may implement a strict subset, but must not invent alternate routes that bypass this ownership. Routes can be marked unsupported in `/capabilities`.

## 4. Command envelope

Mutating commands carry:
- client-generated `request_id`;
- `idempotency_key`;
- command-specific payload;
- optional `expected_version` for optimistic concurrency.

The same idempotency key and identical payload must return the original outcome. Reuse with a different payload returns `IDEMPOTENCY_KEY_REUSED`. Expired idempotency records have a documented retention policy; they are not silently treated as new within the retention window.

## 5. Canonical event envelope

See [event-envelope.schema.json](../../../contracts/json-schema/v1/event-envelope.schema.json).

Fields:
- `event_id`: globally unique event identifier;
- `aggregate_type`: session, workspace, task, executor or organization;
- `aggregate_id`: owning aggregate;
- `seq`: monotonic sequence within the aggregate stream;
- `schema_version`: integer version of the event payload contract;
- `type`: stable namespaced event type;
- `occurred_at`: server timestamp;
- `correlation_id`: request/run trace group;
- `causation_id`: event/command that caused this event, nullable;
- `payload`: bounded typed JSON or artifact references.

A session stream has one authoritative ordered sequence. Cross-aggregate ordering is not guaranteed. Clients must not construct global ordering from wall-clock timestamps.

## 6. WebSocket contract

### Connection
`GET /v1/sessions/{session_id}/events?cursor=<last_applied_seq>` with WebSocket upgrade.

### Server control frames
- `hello`: protocol version, server instance ID, aggregate ID, current sequence, replay policy.
- `resync`: cursor expired/invalid; client must GET snapshot before applying later events.
- `heartbeat`: liveness and last acknowledged server sequence.
- `command.result`: correlated result for a client command sent over the socket, if enabled.

### Server event frame
- `kind = "event"`
- complete canonical event envelope.

### Client frames
- `ack`: latest event sequence durably applied by the client.
- `approval.reply`: approval ID and permitted decision; actor is derived from authentication, never trusted from body.
- `question.reply`: question ID and bounded answer.
- `pong`: heartbeat response.

User turn, cancel and steer commands should use HTTP in v1. This keeps retry/idempotency semantics consistent. WebSocket replies are limited to interactive requests and cursor acknowledgements.

### Replay handshake

1. Client connects with its last committed cursor.
2. Server authenticates and authorizes access to the session.
3. Server attaches to live broadcast before reading replay high-water mark to avoid a replay/live gap.
4. Server sends `hello`.
5. Server replays every retained event after the cursor in strict sequence.
6. Server then streams live events.
7. If the cursor predates retained history or belongs to another server instance, server sends `resync`; client fetches a snapshot and reconnects from snapshot cursor.
8. Client advances cursor only after reducer/state application is committed.
9. Duplicate sequence/event ID is ignored; a sequence gap triggers resync.
10. A lagging client cannot block persistence or other clients.

Event history may be compacted only if a snapshot covers the removed sequence range.

## 7. Error model

Every non-success response uses:

```json
{
  "error": {
    "code": "WORKSPACE_NOT_FOUND",
    "message": "The requested workspace does not exist or is not visible.",
    "retryable": false,
    "request_id": "req_...",
    "details": {}
  }
}
```

Stable code families:
- `AUTH_*`, `FORBIDDEN_*`
- `VALIDATION_*`, `CONFLICT_*`
- `SESSION_*`, `RUN_*`, `TASK_*`
- `APPROVAL_*`, `QUESTION_*`
- `WORKSPACE_*`, `EXECUTOR_*`, `SANDBOX_*`
- `PROVIDER_*`, `TOOL_*`, `PLUGIN_*`
- `ARTIFACT_*`, `INTERNAL_*`

Do not expose provider secrets, local absolute paths outside authorized workspace scope, raw OS environment, stack traces or sensitive payloads in public error messages.

## 8. Event families

At minimum:
- session: created, updated, archived, recovered;
- run: queued, started, progress, completed, failed, cancellation_requested, cancelled, lost;
- turn: started, completed, failed;
- message: created, delta, completed;
- tool: call_created, execution_started, output, completed, failed, cancelled;
- approval: requested, resolved, expired, revoked;
- question: requested, answered, expired;
- task: created, updated, blocked, started, completed, failed, cancelled;
- workspace: created, provisioning, ready, lease_changed, disconnected, closed;
- executor: connected, heartbeat, disconnected, capabilities_changed, job_assigned, job_finished;
- artifact: created, available, expired;
- diagnostic: created, updated, cleared.

These names must be encoded in the AsyncAPI/schema source before code generation.

## 9. Versioning and compatibility

- HTTP path major version: `/v1`.
- Event envelope has independent integer `schema_version`.
- Additive optional fields may be added in a compatible minor release.
- Renaming/removing fields, changing meaning, changing enum semantics or ordering guarantees is breaking.
- Unknown event types are preserved/ignored by generic clients with a visible diagnostic; they do not crash a whole session.
- Server advertises supported protocol versions and minimum client version through `/capabilities`.
- Keep at least current and previous supported client compatibility in release tests.
- Database event migrations must preserve old event identity and provide upcasters; never rewrite event IDs.

## 10. Limits and flow control

- Bound WebSocket frame size, event payload size, pending replies, replay batches and subscriptions per user/device.
- Large stdout, screenshots, patches and binary content become artifacts; events carry references and bounded previews.
- Tool output streaming uses flow-control/backpressure; no unbounded in-memory channel.
- Heartbeat, reconnect backoff and idle timeout are configurable and documented.
- Server enforces per-user/per-workspace session, run, worker and connection quotas in remote mode.

## 11. M0 acceptance evidence

- OpenAPI validates.
- AsyncAPI validates.
- Event schema validates sample valid/invalid fixtures.
- Generated TypeScript types compile.
- Replay tests cover no-gap attach, duplicate event, missing cursor, stale cursor, snapshot hydration and disconnect.
- Idempotency test proves command retry does not duplicate a run, tool call or approval.
- Auth test proves a user cannot subscribe to another user's session.
