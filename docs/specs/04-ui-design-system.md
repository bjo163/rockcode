# 04 — RockCode UI Design System Specification

**Project:** RockCode — Rust-first Autonomous Coding Agent Platform  
**Version:** 1.0  
**Date:** 2 November 2026  
**Status:** Approved design direction; implementation and product-specific artwork remain pending  
**Visual direction:** Industrial Grunge × Manga Linework × Funky Gen-Z × Neo-Brutalist Engineering  
**Canonical design-system dependency:** [Moonframe UI](https://github.com/bjo163/moonframe)  
**Audited Moonframe source baseline:** `0e395f21edef0f01c42caeea34840bdcb71eeabf` (2 November 2026, Asia/Jakarta)

---

## 1. Purpose and governing decision

This document defines the visual and interaction language for RockCode Desktop, Web, and shared workbench surfaces. It fills the visual-design gap left by the system architecture specification.

### 1.1 Primary decision

**RockCode will use Moonframe as its canonical design system.** It will not fork, duplicate, or independently recreate Moonframe's tokens, theme runtime, shared primitives, skin catalog, component behavior, or asset rules.

Moonframe is an existing first-party repository:

- Repository: [bjo163/moonframe](https://github.com/bjo163/moonframe)
- Package name in its root manifest: `@moonwitness/moonframe`
- Package version observed in the audited root manifest: `0.1.1`
- Public entry points observed: root components, `./styles.css`, `./tailwind.css`, `./skins`, and `./theme`
- Runtime evidence documents report 70+ React component exports, 15 skin IDs, Light/Dark appearances, a theme runtime, and a Next.js showcase.
- The audited repository documents local tarball consumption and says npm publication has not been verified. Therefore RockCode must not assume the package is available from the public npm registry.

For the initial integration, consume Moonframe through a pinned Git dependency or another reproducible source integration at an exact commit. Do not point production builds at a floating `main` branch. Reconsider a pinned semver package only after registry availability, provenance, package contents, peer dependencies, and a clean consumer install are verified.

**Source baseline to validate before integration:** [Moonframe commit](https://github.com/bjo163/moonframe/commit/0e395f21edef0f01c42caeea34840bdcb71eeabf). This SHA is a research pin, not a guarantee that all future RockCode builds already consume it.

### 1.2 No competing design system

The following are owned by Moonframe and must not be reimplemented in RockCode:

- Primitive and semantic design tokens.
- Light/Dark/System preference resolution.
- Skin catalog and skin visual grammars.
- Core components such as Button, Field, Dialog, Tabs, Table, Badge, Input, Toast, and command palette primitives.
- Shared component accessibility and keyboard semantics.
- Core motion and optional SFX runtime contracts.
- Skin selectors, mode selectors, and common layout primitives.

RockCode owns only product-specific brand assets, engineering-domain components, workbench compositions, and application behavior.

---

## 2. Product visual identity

### 2.1 Art direction

**Industrial Grunge × Manga Linework × Funky Gen-Z** is the RockCode product expression.

The interface should feel like a high-performance engineering workshop built from:
- rugged industrial work surfaces;
- manga panel framing and expressive ink marks;
- neo-brutalist geometry;
- raw but intentional editorial labels;
- energetic, high-contrast accent moments;
- developer-tool precision and information density.

The personality is bold, slightly rebellious, technical, and playful. The interface must still feel professional enough for long coding sessions, serious repository work, incident recovery, and enterprise engineering.

### 2.2 Visual pillars

| Pillar | Visual behavior | Where it belongs |
|---|---|---|
| Industrial structure | Framed work zones, equipment-like separators, measured numbering, strong rails, plate-like grouping | Workbench shell, worker status, settings, execution panels |
| Manga linework | Original ink strokes, panel cuts, screentone, directional marks, action lines | Empty states, onboarding, hero surfaces, section transitions, selected decorative slots |
| Funky Gen-Z energy | Bold display labels, playful microcopy, asymmetric composition, accent marks, expressive but controlled art | Brand moments, onboarding, empty states, non-critical callouts |
| Grunge materiality | Restrained grain, photocopy character, imperfect edge accents, hard shadow language | Background/surface decoration and selected feature headers |
| Engineering clarity | Consistent spacing, readable body text, strong contrast, compact data patterns, stable controls | Every interactive and information-bearing region |

### 2.3 Non-goals

RockCode must not become:
- a generic blue/purple SaaS dashboard;
- a full-screen cyberpunk poster that hides source code and logs;
- an arcade game interface where every interaction is animated;
- a dashboard made from hundreds of decorative cards;
- a terminal emulator pretending to be a complete product;
- a visual clone of MoonWitness or any unrelated Moon ecosystem consumer application.

Do not use MoonWitness characters, logos, narrative symbols, or consumer-specific artwork in RockCode. Create original RockCode engineering artwork.

---

## 3. Moonframe skin strategy

Moonframe's skin is a complete visual grammar, not a palette switch. RockCode must preserve its canonical skin IDs and use the existing theme runtime.

### 3.1 Default and alternatives

| Role | Moonframe skin | Usage |
|---|---|---|
| Default product expression | `moonpunk` | Default new-install visual identity: acid-lime manga punk, thick outlines, hard offset shadows, asymmetric framed zones |
| Industrial alternative | `mooncrash` | Optional complete alternative skin for users who prefer mechanical plates, equipment-rack navigation and industrial maintenance composition |
| High-density professional alternative | `moonledger` | Optional complete alternative for compact operational and data-heavy work |
| Neutral fallback | `classic` | Safe fallback for unknown or invalid preferences and a restrained visual option |

All 15 Moonframe skins remain available through the shared skin selector unless product research later demonstrates a concrete usability reason to hide a subset. RockCode must not create a sixteenth skin, rename Moonframe IDs, or combine the visual recipes of different skins inside one active appearance.

### 3.2 Default appearance

- First-run default skin: `moonpunk`.
- First-run appearance mode: `dark` (developer workstation bias).
- User may choose `light` or `system`.
- Persist skin, mode, density, motion, and contrast as separate preference axes.
- Respect OS forced-colors and reduced-motion preferences.
- Switching skin or mode must not alter application data, session state, permissions, or keyboard behavior.

Moonframe's theme resolver remains the source of truth. RockCode must not create a second competing ThemeProvider, a second localStorage preference key, or its own CSS skin resolver.

### 3.3 Industrial identity without skin mixing

Industrial character can be added through RockCode-specific product assets and domain patterns: original machine-label marks, tool-call numbering, worker identifiers, executor plates, workspace IDs, and status rails.

Those product marks must use semantic Moonframe colors and approved RockCode asset slots. They may not override Moonframe's active skin grammar or re-color all skins to match the default Moonpunk appearance.

---

## 4. Color, surface, and token policy

### 4.1 Token ownership

Moonframe owns all canonical token values. RockCode must consume semantic variables and public theme APIs rather than defining a second palette in application CSS.

Use semantic roles such as:
- canvas and surface levels;
- primary and secondary text;
- muted and disabled text;
- primary/secondary/destructive action;
- subtle/strong borders;
- focus and selection;
- success, warning, danger, info;
- code, diff additions, diff removals, and execution states;
- chart series and telemetry roles.

If RockCode needs a genuinely new semantic role (for example, executor state or code-diff state), propose it through a documented Moonframe-compatible token extension or a RockCode-scoped semantic alias. Do not hardcode hex values in components.

### 4.2 Visual treatment

- Prefer matte, ink-like surfaces with visible boundaries over glassmorphism.
- Use strong borders where they clarify pane boundaries and active controls.
- Use hard-offset shadows only where the active Moonframe skin provides that recipe.
- Keep texture opacity low enough that source code, logs, form labels, tables, and terminal output remain crisp.
- Reserve warning stripes and hazard colors for actual warning/attention semantics, not generic decoration.
- Never use accent color alone to communicate execution, permission, failure, or completion state.
- Charts and code diffs must have accessible text/legend equivalents.

### 4.3 Decoration budget

Decorative layers are subordinate to real content:
- Decorative linework must not intersect body text, code, terminal output, inputs, buttons, diff rows, or status messages.
- Halftone, grain, photocopy texture, and impact marks must be sparse and removable.
- Use decorative overlays behind content with pointer events disabled.
- Provide a global `decorations off` mode and test all screens in that mode.
- In reduced-effects mode, remove nonessential glitch, parallax, scanline, animated texture and background motion.
- Failed or missing artwork must fall back to a stable layout, not a broken image box.

---

## 5. Typography and information hierarchy

Moonframe owns the base type tokens and approved font policy. RockCode may supply an original display treatment only through a reviewed font/asset extension with a recorded license and fallback.

### 5.1 Roles

| Role | Requirement |
|---|---|
| Display / brand headline | Condensed, expressive, high-impact; use sparingly |
| Page heading | Strong geometric sans, concise and scannable |
| Body / UI text | Neutral, highly readable sans; stable line-height |
| Metadata / labels | Compact, clear, medium weight; never excessively tracked |
| Code / terminal / IDs | Monospaced font with reliable glyph coverage and distinguishable characters |
| Numeric telemetry | Tabular numerals where alignment matters |

### 5.2 Rules

- Decorative type is allowed for product section headers and onboarding, never for code, error details, settings labels, or high-frequency controls.
- Body text and labels must remain selectable, searchable DOM text.
- Never bake important words, values, or actions into an image.
- Avoid uppercase for long sentences; short industrial labels may use uppercase.
- Preserve readable line length in documentation and side panels.
- Long model names, paths, branch names, command lines, and tool arguments must wrap, truncate with disclosure, or scroll intentionally without breaking the layout.

---

## 6. Geometry, linework, icons, and brand assets

### 6.1 Geometry

Use the active Moonframe skin's approved component geometry. RockCode application layouts may create asymmetric workbench arrangements, but interactive controls must retain predictable hit areas, focus boundaries, and alignment.

### 6.2 Manga linework

Create original vector assets in a dedicated RockCode brand-asset area:
- directional strokes and speed lines;
- manga panel corner marks;
- original screentone patterns;
- mechanical registration marks;
- abstract impact bursts;
- numbered engineering plates;
- tool/executor glyphs;
- original character-free mascot/robotic engineering motifs, only where they help recognition.

Asset usage:
- onboarding, empty states, first-run welcome, product landing page, help/about, non-blocking success moments;
- occasional low-opacity separators or decorative edge treatment;
- no background art behind terminal/editor/code/diff surfaces.

### 6.3 Functional icons

- Use one consistent outline-icon family and stable icon sizing.
- Prefer Moonframe's existing icon/brand APIs where available; otherwise use a thin RockCode icon facade over an approved icon set.
- Do not replace common functional icons with custom illustrations.
- Custom icons require a stable semantic name, accessible label when meaningful, decorative handling when redundant, and original source SVG.
- Avoid arbitrary emoji as status indicators in production controls.

### 6.4 Asset governance

Every new asset must have:
- stable asset ID and filename;
- creator/source and provenance;
- license/rights status;
- intended skin and context;
- dimensions/viewBox and light/dark variants;
- accessible description or decorative classification;
- optimization and checksum;
- approval status.

Do not copy artwork, characters, logos, textures, or typography from upstream products. Generated assets are not automatically rights-cleared; record provenance and review originality.

---

## 7. Layout and application composition

RockCode is a multi-pane engineering workspace. It is not a landing-page-first product after onboarding.

### 7.1 Desktop information architecture

The primary desktop shell has five functional regions:

1. **Global rail:** app navigation, workspaces, recent sessions, server identity and user/device entry.
2. **Workspace sidebar:** projects, repositories, sessions, agents, tasks, worktrees and pinned resources.
3. **Primary workbench:** conversation/transcript, editor, diff, browser, terminal or task graph.
4. **Inspector/utility panel:** context, model usage, tool details, diagnostics, permissions, artifacts and selected item metadata.
5. **Bottom utility dock/status strip:** terminal/processes, task queue, worker health, connection state and current execution target.

Regions must be resizable, collapsible, reorderable within supported constraints, and restore layout per workspace. Avoid permanently visible empty panes.

### 7.2 Recommended default desktop layout

```text
┌──────────────────────────────────────────────────────────────────────┐
│ App / Workspace / Server / Branch / Run controls / Window actions   │
├───────┬────────────────┬─────────────────────────────┬───────────────┤
│ Global│ Workspace      │ Primary Workbench           │ Inspector     │
│ rail  │ sidebar        │                             │ / Context     │
│       │                │ Agent transcript / Editor   │ / Model       │
│       │ Projects       │ Diff / Browser / Task graph │ / Diagnostics │
│       │ Sessions       │                             │ / Tool detail │
│       │ Worktrees      │                             │               │
├───────┴────────────────┴─────────────────────────────┴───────────────┤
│ Bottom dock: Terminal / Processes / Queue / Problems / Logs         │
├──────────────────────────────────────────────────────────────────────┤
│ Status: Workspace • Executor • Branch • Agent • Connection • Usage  │
└──────────────────────────────────────────────────────────────────────┘
```

The exact pane arrangement is responsive and user-configurable. The diagram describes information ownership, not a fixed screenshot.

### 7.3 Web layout

- Web uses the same semantic application shell and workbench compositions as Desktop.
- Local/remote server selector and workspace selector are explicit.
- Web layout may collapse global rail and inspector into drawers.
- A browser client must never silently acquire host-machine filesystem or shell access.
- Wide code workspace remains available on desktop browsers; narrow view prioritizes session review, approvals, task status and remote monitoring.
- On mobile, full source editing is not a requirement for the first version; safe inspection and control flows are.

### 7.4 Responsive targets

| Context | Layout behavior |
|---|---|
| Wide desktop ≥ 1440 px | Full rail + sidebar + primary workbench + inspector + bottom dock |
| Standard desktop 1280–1439 px | Inspector collapsible; preserve workbench and sidebar |
| Small desktop/tablet 768–1279 px | Rail + one main pane; secondary panels become overlays/drawers |
| Mobile < 768 px | Session timeline, approvals, task monitoring and settings; no forced multi-column workbench |

All breakpoints must be validated using real content, long paths, long model names, tool payloads, translations, and active execution.

---

## 8. Density, navigation, and interaction grammar

### 8.1 Density preferences

Use Moonframe's independent density axis:
- `comfortable`: first-run default.
- `compact`: high-information code/ops workflows.
- `spacious`: accessibility or presentation preference.

Density changes padding, row height and grouping—not information semantics, hidden fields, permission visibility, or hit-target requirements.

### 8.2 Navigation

- Global navigation names remain stable and text-labelled when space permits.
- Icon-only navigation requires accessible names and tooltips.
- Active location must be visible through more than color (shape, marker, text weight or border).
- Use breadcrumbs for deep resource paths and command palette for cross-workspace navigation.
- Avoid more than two permanent navigation levels in the primary desktop sidebar.
- Search must distinguish project, file, session, message, tool, agent, command and setting result types.

### 8.3 Command palette and shortcuts

The command palette is a first-class interaction surface:
- fuzzy search over actions and navigable resources;
- category and context-aware availability;
- keyboard-first operation;
- visible shortcut hints;
- no action hidden solely behind hover;
- disabled actions explain why they are unavailable;
- destructive actions require a clear confirmation path.

Shortcuts must be configurable where practical, conflict-checked, documented, and tested on Windows keyboards. Do not override common OS/browser shortcuts without a compelling reason.

### 8.4 Agent status grammar

Execution states must use consistent, semantic status labels:
- queued;
- planning;
- running;
- waiting for approval;
- waiting for input;
- retrying;
- cancelling;
- cancelled;
- completed;
- failed;
- disconnected;
- recovering.

Status badges must include text or accessible labels; color and motion are supplementary. Never animate a task as completed until the engine reports completion.

### 8.5 Approval UI

Permission/approval requests are among the most important screens:
- show the exact action, tool, target path/URL/command, workspace, executor, risk explanation and policy decision;
- distinguish approve-once, approve-for-session, deny, and cancel when those semantics are supported by the engine;
- make the active trust boundary clear (local machine, container, or remote worker);
- never hide command text behind a decorative label;
- denial and timeout states are explicit;
- keyboard focus is contained in modal approval flows and restored afterward.

---

## 9. Screen and feature catalogue

These are product screens/compositions, not promises that backend functions already exist.

| Screen / area | Main UI responsibilities | Key reusable components |
|---|---|---|
| Welcome / onboarding | Explain local vs remote, provider setup, workspace permission, theme choice | Onboarding steps, provider cards, permission summary |
| Home / command center | Recent workspaces, recent sessions, active tasks, quick actions | Workspace cards, activity feed, queue summary |
| Workspace overview | Repositories, branches, worktrees, instructions, status | Workspace header, repository list, branch selector |
| Agent session | Transcript, streaming, tool calls, attachments, composer, run controls | Session timeline, message, tool activity, composer |
| Agent task graph | Parent/child tasks, dependencies, worker assignment, status | Task graph, task card, agent avatar/identity, progress |
| File explorer | Project tree, search, selection, file metadata | Tree view, search, context menu |
| Code editor | Source editing, tabs, diagnostics, selection and navigation | Editor adapter, tabs, breadcrumb, diagnostics |
| Diff review | Before/after, file list, hunk selection, apply/revert | Diff viewer, review controls, change stats |
| Terminal / processes | Multiple PTYs, active commands, process lifecycle | Terminal panel, process list, command result |
| Browser session | Tabs, address, screenshot/viewport, agent/user control | Browser toolbar, page view, event strip |
| Context / model inspector | Provider, model, context budget, usage, token/cost stats | Context meter, model card, usage panel |
| Tool inspector | Inputs/outputs, duration, status, artifacts, audit link | Tool call detail, JSON viewer, copy/download |
| Permissions center | Rules, scopes, pending requests, history | Policy table, rule editor, approval dialog |
| Providers / model router | Provider setup, account, model capability, routing, fallback | Provider card, model picker, capability matrix |
| MCP / ACP integrations | Server connections, OAuth, tools, protocol status | Integration card, tool catalogue, connection form |
| Plugins / skills | Install/source/trust state, enablement, configuration | Plugin list, manifest viewer, capability grant |
| Remote workers | Worker health, heartbeat, executor type, resource usage | Worker card, queue list, machine detail |
| Automation | Schedules, triggers, run history, retry, logs | Schedule form, run timeline, webhook detail |
| Settings | Appearance, key bindings, editor, terminal, providers, workspace, privacy | Settings navigation, grouped forms |
| Diagnostics / updates | Logs, health, version, migrations, update state | Diagnostic panel, update status, copy report |

Every screen must specify loading, empty, error, offline, permission-denied, stale, partial-data and narrow-screen states.

---

## 10. Motion and SFX

### 10.1 Motion

- Use Moonframe's motion preference and recipe system.
- Motion expresses orientation, state feedback, progress and hierarchy; it never controls business state.
- Default transitions are short and purposeful.
- No perpetual scanlines, constant glitching, flicker, cursor-following parallax, scroll hijacking, or animated noise behind code.
- A cancelled or interrupted animation must settle to the correct semantic state.
- Reduced-motion and forced-colors modes must be tested.

### 10.2 Sound

- SFX off by default until the user deliberately enables them.
- Never play sounds on initial render or every click.
- Useful optional events: task completed, approval required, failure needing attention, remote worker disconnected.
- Provide global mute, category preferences, volume, rate limits and notification deduplication.
- Sound is never the only notification channel.
- Reuse Moonframe's opt-in SFX interface where appropriate; do not invent a second SFX event namespace.

---

## 11. Accessibility and usability requirements

Target **WCAG 2.2 AA** as the engineering acceptance target; do not claim certification without a documented audit.

Minimum requirements:
- Full keyboard navigation and visible focus.
- Semantic heading hierarchy and landmark regions.
- Correct native control semantics and accessible names.
- Dialog focus trap, Escape behavior and focus restoration.
- Screen-reader announcements for streaming state changes, approval prompts, task completion/failure, and connection loss without excessive chatter.
- Non-color-only status/validation/diff indicators.
- Contrast validation in all 15 skins and both appearances for the components RockCode uses.
- Forced-colors support.
- Reduced motion and no-animation fallback.
- Zoom/reflow support and long-content resilience.
- Hit targets suitable for touch where the UI exposes touch interactions.
- Code, terminal and diff content must remain selectable and compatible with assistive tooling as far as the selected editor/terminal engine supports.
- Do not place required text in SVG-only art or canvas.

### 11.1 Required appearance and state test matrix

For RockCode's primary component set, validate:
- Moonpunk Light and Dark;
- Mooncrash Light and Dark;
- Moonledger Light and Dark;
- Classic Light and Dark;
- all supported density modes;
- reduced motion;
- high contrast / forced colors;
- decorations off;
- keyboard-only;
- screen reader spot checks;
- wide, standard and narrow viewports.

A future release review must also test the entire supported 15-skin × 2-mode catalog because Moonframe is the theme owner.

---

## 12. Performance and reliability

- Lazy-load editor, terminal, browser, charts, graph visualizations and heavy asset packs.
- Avoid rendering thousands of transcript messages at once; use virtualization with stable anchors and correct accessibility fallbacks.
- Preserve scroll position during streaming and event replay.
- Avoid remounting the editor or terminal on theme changes.
- Do not couple event ingestion to heavy React render work.
- Keep UI responsive when the backend is disconnected or recovering.
- Provide skeletons only for known layout; otherwise use a stable progress/status message.
- Large tool results should render bounded previews with explicit expansion/download.
- Errors in optional visual assets must never break the core workspace.

Initial targets to verify in prototypes (not current measurements):
- App shell interactive without loading editor/browser bundles.
- Navigation and command palette remain responsive during token streaming.
- Theme switching does not recreate active session state.
- 10k+ transcript event stress test does not cause unbounded DOM growth.
- Long tool output, large diff, and large directory tests have predictable memory and scroll behavior.

---

## 13. UI acceptance gates

### UI-G0 — Visual contract
- Design system dependency pinned.
- Theme owner confirmed.
- Product visual profile selected.
- No duplicate token/theme runtime introduced.
- Screen map and pane ownership reviewed.

### UI-G1 — Core component adaptation
- Shared shell, navigation, buttons/forms/overlays, data table and toast use Moonframe primitives.
- No duplicate primitive library.
- Keyboard and accessibility checks pass for shared behavior.

### UI-G2 — Engineering components
- Session timeline, composer, task status, tool card, approval prompt, diff, terminal and worker components have typed props and documented state variants.
- Components consume view models and commands rather than owning agent business logic.

### UI-G3 — Workbench experience
- Resizable/collapsible panes, saved layouts, command palette, responsive adaptation and reconnect states work.
- Empty/error/offline/partial states are complete.

### UI-G4 — Visual QA
- Screenshots for default Moonpunk Light/Dark plus Mooncrash and Classic samples.
- Realistic repository content, long identifiers and streaming states.
- Decorations-off and reduced-motion review.
- Visual regression baseline is reviewed, not blindly auto-approved.

### UI-G5 — Release readiness
- Package smoke test against a consumer app.
- Accessibility and keyboard report.
- Bundle/performance report.
- Asset/license/provenance manifest.
- Cross-client parity validation for Desktop and Web.

---

## 14. Explicit exclusions and change control

- This specification does not authorize copying Moonframe source into RockCode.
- This specification does not authorize creating production artwork, fonts, SFX, or logos without provenance review.
- This specification does not lock a specific editor or terminal vendor; those are separate adapters.
- This specification does not change agent API, runtime or security contracts.
- Any request to change the canonical Moonframe token/theme/primitive contract must be proposed in the Moonframe repository and then consumed by RockCode through a reviewed dependency update.

---

## 15. Source references

**Canonical RockCode decision:** this document.

**Moonframe sources audited:**
- [Moonframe repository](https://github.com/bjo163/moonframe)
- [Moonframe Design System Bible v1.3](https://github.com/bjo163/moonframe/blob/main/docs/moonframe-ui-design-system-bible-v1.3.md)
- [Moonframe Package Architecture & DX Bible](https://github.com/bjo163/moonframe/blob/main/docs/architecture/package-architecture-dx-bible.md)
- [Moonframe Runtime Usage](https://github.com/bjo163/moonframe/blob/main/docs/runtime-usage.md)
- [Moonframe Implementation Status](https://github.com/bjo163/moonframe/blob/main/docs/implementation-status.md)
- [Moonframe Component API Behavior Bible](https://github.com/bjo163/moonframe/blob/main/docs/component-api-behavior-bible.md)
- [Moonframe Skin Grammar Contract](https://github.com/bjo163/moonframe/blob/main/contracts/moonframe-skin-grammar-contract-v1.0.json)
- [Moonframe Theme Contract](https://github.com/bjo163/moonframe/blob/main/contracts/moonframe-theme-contract-v1.3.json)
- [Moonframe Motion / Interaction / SFX Bible](https://github.com/bjo163/moonframe/blob/main/docs/motion/motion-interaction-sfx-bible.md)
- [Audited Moonframe commit](https://github.com/bjo163/moonframe/commit/0e395f21edef0f01c42caeea34840bdcb71eeabf)

**Important status note:** Moonframe repository documents contain both implementation evidence and broader specification/approval records. RockCode relies on the specific runtime entry points and implementation evidence only after a consumer smoke test; it does not infer that all planned assets, marketplace features, or human visual approvals are complete.
