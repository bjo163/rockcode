# M0-04 — Domain Model & Event Invariants

**Status:** Normative domain contract for v1.  
**Machine-readable contracts:** Event, session and approval JSON Schemas under contracts/json-schema/v1/.

## 1. Domain modeling principles

- Domain IDs are opaque, globally unique and stable. Public IDs use a short type prefix plus UUIDv7 or another approved collision-resistant identifier; clients must not parse ID internals.
- All timestamps are RFC 3339 UTC. A timestamp is descriptive, not an ordering guarantee.
- Ordering is guaranteed only by a sequence within an aggregate stream.
- Durable state is changed through domain commands/use cases and appended events; UI state is never authoritative.
- Large/binary values are artifacts referenced by ID and digest, not embedded in event rows.
- Every entity has an explicit owner/tenant scope where remote mode applies.
- Deletion and archive semantics are distinct. Audit and event retention are explicit policy decisions.
- Model output, tool output, repository content and plugin content are untrusted inputs even when stored in a session.

## 2. Entity ownership

| Entity | Aggregate owner | Canonical responsibility |
|---|---|---|
| Organization | Organization | Remote tenant boundary and default policy |
| User | Organization | Human identity and role membership |
| Device | User | Trusted client identity, public key, revocation |
| Workspace | Workspace | Root, executor binding, isolation profile and lease |
| Project | Workspace | Repository metadata, project instructions and settings version |
| Session | Session | Conversation and run context |
| Message | Session | User/assistant/system/tool content and visibility |
| Run | Session | One user/automation/parent-agent execution lifecycle |
| Turn | Run | One model/tool iteration group |
| Step | Turn | One state-machine operation/effect unit |
| ToolCall | Step | Tool request, authorization, execution and result |
| Approval | Session or scoped resource | Human/policy decision with actor and expiry |
| Question | Session | Persisted user-input request |
| AgentTask | Task | Parent/child/dependency graph node |
| ExecutionJob | Workspace | Process/browser/container/remote execution lifecycle |
| Executor | Server/worker | Capability report and health identity |
| Plugin | Installation scope | Manifest, digest, grants, trust and lifecycle |
| Skill | Project/user/organization scope | Declarative instructions and references |
| Artifact | Workspace/session | Content-addressed large output |
| Event | Aggregate | Immutable state transition/audit fact |
| Snapshot | Aggregate | Hydration checkpoint through a known sequence |

## 3. Relationships

~~~mermaid
erDiagram
  ORGANIZATION ||--o{ USER : contains
  USER ||--o{ DEVICE : enrolls
  ORGANIZATION ||--o{ WORKSPACE : owns
  WORKSPACE ||--o{ PROJECT : contains
  WORKSPACE ||--o{ SESSION : hosts
  SESSION ||--o{ RUN : contains
  RUN ||--o{ TURN : contains
  TURN ||--o{ STEP : contains
  STEP ||--o{ TOOL_CALL : invokes
  SESSION ||--o{ APPROVAL : requests
  SESSION ||--o{ QUESTION : asks
  SESSION ||--o{ TASK : coordinates
  TASK ||--o{ TASK : parent_child
  WORKSPACE ||--o{ EXECUTION_JOB : runs
  EXECUTION_JOB }o--|| EXECUTOR : assigned_to
  SESSION ||--o{ ARTIFACT : references
  SESSION ||--o{ EVENT : emits
  SESSION ||--o{ SNAPSHOT : checkpoints
~~~

The diagram is conceptual; implement normalized tables/collections and indexes based on query and concurrency needs. It does not prescribe one SQL schema.

## 4. Entity invariants

### 4.1 Organization, user and device
- A user can access an organization only through an active membership.
- Every remote workspace/session/artifact query is tenant-scoped server-side.
- A device token is revocable; revocation closes active streams and rejects new requests.
- Device credentials are stored as hashes/public-key material as appropriate, never as reusable plaintext bearer tokens in the database.

### 4.2 Workspace and project
- A workspace has one canonical executor and root URI at a time.
- Workspace root cannot be changed while an execution job holds an active write lease.
- A project instruction file is untrusted project content. It cannot grant new tool, filesystem, network or plugin capabilities.
- Mutating parallel tasks receive isolated worktrees/copies by default.
- Lease expiry is not proof a process stopped. Reconcile executor state before deleting data.

### 4.3 Session, run, turn and step
- A session can contain multiple runs.
- A run is bound to one workspace, executor, agent profile revision and policy revision.
- A run has exactly one terminal outcome among completed, failed or cancelled; lost is a recovery/uncertainty outcome, not success.
- Turn sequence is monotonic within its run; step sequence is monotonic within its turn.
- A new user turn cannot silently start while a non-interruptible turn is still active.
- Cancel request and cancelled outcome are distinct states.
- A client disconnect does not cancel the run unless an explicit policy says so.
- Session archive does not erase its audit/event history.

### 4.4 Tool calls and execution jobs
- A ToolCall exists before execution begins and is not complete until success/failure/timeout/cancellation is durably recorded.
- Each mutating invocation has a stable idempotency key.
- A ToolCall can produce zero or more ExecutionJobs and Artifacts.
- A retry must identify whether it is retrying the same idempotent operation or creating a new operation.
- The executor's reported result must be authenticated and bound to the assigned job/lease.
- A worker that disappears cannot be assumed to have stopped a process; status becomes suspect/lost until reconciled.

### 4.5 Approval and question
- Approval request contains exact action, target, workspace, executor, risk and policy scope.
- Approval actor is derived from authenticated server context, never accepted as a trusted client field.
- Approval is one-shot unless an explicit rule grant is created and persisted separately.
- Approval expires and can be revoked.
- A UI timeout does not mean the server approved or denied.
- Questions are durable and correlated to a run/step; absent answer at expiry follows explicit policy.

### 4.6 Plugin and skill
- Installing a plugin does not automatically enable it.
- Project manifests may request a plugin but cannot execute it or self-grant capabilities.
- Plugin version/digest/API compatibility is pinned for an active run.
- Skill content is declarative and does not grant execution capability.
- Untrusted skill instructions never override platform/system policy.

## 5. Canonical lifecycle states

### Session
created, idle, running, waiting_approval, waiting_input, waiting_child, queued, disconnected, recovering, completed, failed, cancelled, archived.

### Run
queued, starting, running, waiting_approval, waiting_input, waiting_child, cancelling, recovering, completed, failed, cancelled, lost.

### ToolCall / ExecutionJob
pending, validating, awaiting_policy, awaiting_approval, queued, starting, running, streaming, succeeded, failed, timed_out, cancel_requested, cancelled, lost, recovered.

### Approval
pending, approved, denied, expired, revoked, cancelled.

### Workspace
created, provisioning, ready, busy, suspended, disconnected, closing, closed, failed, lost.

### Executor
registered, connecting, healthy, degraded, draining, disconnected, revoked.

A transition table is required in implementation tests. Invalid transitions must return a stable conflict error and must not append misleading events.

## 6. Aggregate and event ownership

- Session events are ordered by the session aggregate sequence.
- Workspace events are ordered by workspace sequence.
- Executor events are ordered by executor sequence.
- A cross-aggregate causal relationship uses correlation_id and causation_id; do not invent a global total order.
- If one command mutates two aggregates, use a transaction/outbox or explicit saga with compensating/recovery events; never imply atomicity across separate databases/workers.
- Event append, aggregate version increment and idempotency outcome should commit transactionally for local/remote relational adapters.
- Event payloads are immutable after append. Corrections are new events.
- Schema changes use event upcasters; historical event IDs and sequence numbers are preserved.

## 7. Message content and artifacts

Message content supports typed parts: text, image reference, file reference, tool-call reference, structured data and provider-native metadata reference. Do not assume all messages are plain strings.

Large tool outputs, screenshots, browser captures and patches are stored as artifacts with:
- ID;
- workspace/session ownership;
- kind and media type;
- digest/checksum;
- byte size;
- created timestamp;
- retention class;
- access scope;
- storage reference.

Artifacts are never public just because their URL is hard to guess. Download routes enforce authorization and may issue short-lived signed URLs.

## 8. Persistence and migration

- SQLite local schema and PostgreSQL remote schema share domain/repository semantics, not necessarily identical physical schema.
- Every migration is numbered, forward-applied and tested from the previous supported schema.
- Do not edit applied migration files.
- Snapshot format is independently versioned from database schema and event payload schema.
- Recovery reconciles database state with executor leases; do not infer external process completion from database alone.
- Local database backup/export includes a version manifest and artifact references.
- User deletion, organization deletion and retention policies must state which events/artifacts are retained, anonymized or deleted.

## 9. M0 acceptance evidence

- Every entity has an owner, tenant scope and lifecycle.
- State transition tables are converted to unit tests before runtime implementation is considered complete.
- Duplicate command and duplicate event tests are specified.
- Event ordering and snapshot coverage rules are unambiguous.
- Artifact access is separately authorized.
- Event payloads contain no credentials or unnecessary raw workspace secrets.
