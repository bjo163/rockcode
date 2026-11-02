# M0-06 — Platform & Support Matrix

**Status:** Normative support policy proposal; exact minimum OS/browser versions are a release-time validation item.  
**Product priority:** Windows x64 Desktop first, Web cross-platform, Rust server/workers headless.

## 1. Support tiers

- **Tier 1 — First-class:** required in release CI, build, install, update, smoke and core feature tests.
- **Tier 2 — Supported server/client:** core workflows tested, but not every native integration is available.
- **Tier 3 — Experimental / best effort:** must be explicitly labelled; no parity claim.
- **Not supported:** do not advertise or create release artifacts as supported.

## 2. Platform matrix

| Surface / platform | M0 target | Required CI / evidence | Restrictions / notes |
|---|---|---|---|
| Desktop Windows 11 x64 | Tier 1 | Tauri build, installer/signing pipeline validation, clean install/update/uninstall, local daemon, file dialog, key vault, PTY, crash recovery | Primary desktop target |
| Desktop Windows 10 x64 | Tier 1 candidate; exact supported servicing baseline to be locked before release | Same as Windows 11 plus legacy API compatibility and sandbox tests | Do not promise all Windows 10 releases; define tested baseline explicitly |
| Desktop Windows ARM64 | Tier 3 initially | Cross-compile/native smoke, dependency audit and performance test | Promote only after native editor/terminal/sandbox dependencies are validated |
| Desktop macOS ARM64/x64 | Tier 3 initially | Tauri packaging, Keychain, shell/PTY and sandbox conformance | Later milestone; no parity claim until CI and security review pass |
| Desktop Linux x64 | Tier 3 initially | Package/build, Secret Service, PTY and sandbox smoke | Desktop distribution follows server/worker stability |
| Web on current Chrome/Edge | Tier 1 | Functional, keyboard, streaming, reconnect, editor/terminal adapter, CSP/CORS and visual tests | Primary browser target |
| Web on current Firefox | Tier 2 | Core app, streaming and workspace smoke; document editor/terminal differences | No browser-specific security relaxation |
| Web on current Safari | Tier 2 | Core app, auth, event stream fallback and responsive smoke | Confirm WebSocket/secure-context behavior at release |
| Mobile browser | Tier 3 monitoring/control | Session review, approvals, task status and responsive shell | Full source editing not required for first release |
| Linux x64 Agent Server | Tier 1 | Container build, service startup, PostgreSQL, Docker/OCI executor, worker enrollment, observability | Primary remote server target |
| Linux ARM64 Agent Server | Tier 3 initially | Native image and worker tests | Promote after dependency/platform validation |
| Windows local headless agentd | Tier 1 with Desktop | Loopback bind/auth, service supervision, SQLite recovery, native process lifecycle | No public listener by default |
| Windows remote worker | Tier 2 after sandbox POC | Enrollment, leases, process tree cleanup, filesystem boundary, network policy | Higher-risk jobs should use container/VM/WSL2 boundary until native enforcement is proven |
| macOS remote worker | Tier 3 | Worker protocol, sandbox and keychain integration | Later |
| Container/OCI worker | Tier 1 for remote Linux deployment | Non-root, read-only base image, explicit writable mounts, cgroup/resource/network limits, cleanup and image provenance | Default isolated backend for hosted execution |

## 3. Windows-first commitments

- Windows is the first-class Desktop development and release environment.
- Architecture must not rely on Unix-only APIs in the domain/kernel/protocol crates.
- All paths crossing the API use normalized workspace-relative identifiers; never accept arbitrary host absolute paths from a remote client.
- Handle Windows path semantics: drive letters, UNC, reserved names, case-insensitive lookup, junctions/reparse points, symlinks and long paths.
- Child processes are tracked as process trees/jobs; shutdown behavior is deterministic.
- Native notifications, file/folder picker, key vault and deep links are Tauri host adapters.
- Local server binds loopback only unless an explicitly configured remote gateway owns exposure.
- Windows sandbox capability reporting must describe actual enforcement, not intended enforcement.

## 4. Windows isolation policy

Until the selected native/WSL2/container isolation backend passes P0 adversarial tests:
- inspect_only is always available.
- trusted_local may be offered only with explicit, clearly worded opt-in and must never be called sandboxed.
- Untrusted project tasks that require arbitrary shell execution must use a verified container/VM/WSL2 executor or remain blocked.
- Missing Docker/WSL2 or unavailable policy features must produce an explicit error or user-selected weaker mode.
- No automatic fallback from isolated mode to unrestricted host execution.

## 5. Browser support policy

- Use feature detection, not user-agent branching, except for documented compatibility workarounds.
- WebSocket is the primary live transport; any SSE fallback must keep the same event IDs and cursor semantics.
- Web storage contains no provider secret, worker token or local bootstrap secret.
- Clipboard, file system access, notifications and other privileged browser APIs require secure context and explicit feature handling.
- Browser support must include keyboard and screen-reader smoke, not only screenshot rendering.

## 6. Build and release matrix

Every supported release records:
- OS version/build and architecture;
- Rust toolchain and target triple;
- Node.js/pnpm versions for UI build;
- Tauri/native runtime versions where applicable;
- editor/terminal/browser engine versions;
- sandbox capabilities detected and effective;
- artifact checksum, signing status and provenance;
- test workflow URL and failure/exception notes.

## 7. Promotion rules

A Tier 3 platform cannot become Tier 2/1 solely because it compiles. Require:
1. clean install/build;
2. runtime smoke;
3. session streaming/recovery;
4. workspace and tool behavior;
5. platform-specific credential and process handling;
6. security/sandbox tests appropriate to its executor;
7. update/uninstall/recovery validation;
8. signed release artifact and support documentation.

## 8. M0 acceptance evidence

The first release must explicitly name supported Windows baseline and browser versions. CI matrix must be agreed before M1 code; no cross-platform claim may be inferred from Rust compilation alone.
