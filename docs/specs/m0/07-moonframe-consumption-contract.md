# M0-07 — Moonframe Consumption Contract

**Status:** Normative UI dependency contract for RockCode.  
**Canonical upstream:** [bjo163/moonframe](https://github.com/bjo163/moonframe).  
**Audited baseline:** 0e395f21edef0f01c42caeea34840bdcb71eeabf.

## 1. Ownership decision

Moonframe is the canonical design system for RockCode. RockCode consumes Moonframe and adds engineering-product compositions. RockCode does not fork or redefine Moonframe's:
- semantic/primitive tokens;
- theme resolver and persistence semantics;
- skin catalog and visual grammar;
- shared base component semantics;
- shared accessibility/keyboard behavior;
- common motion and optional SFX runtime.

RockCode owns:
- its product shell and information architecture;
- engineering-specific Agent UI and workbench components;
- editor and terminal adapters;
- RockCode logo/marks and original industrial/manga artwork;
- product navigation and layout preferences;
- app-level data and event view models.

The authoritative product design is documented in [04 UI Design System](../04-ui-design-system.md) and [05 UI Package Architecture](../05-ui-package-architecture.md).

## 2. Audited Moonframe source evidence

At the audited commit:
- root package name is @moonwitness/moonframe;
- root package version is 0.1.1;
- package manifest exports the root component entry, styles.css, tailwind.css, skins and theme subpaths;
- peer dependencies include React and React DOM 19;
- docs/runtime-usage.md and docs/implementation-status.md report a reusable React component catalog, fifteen skin IDs, Light/Dark appearance modes, and a theme runtime;
- docs/package-architecture-dx-bible.md and runtime guidance state that public npm publication was not verified in the audit and describe local tarball consumption.

These facts are source-snapshot facts, not proof that an npm package is currently published or that every planned Moonframe feature is implemented. The immutable source pin must be consumer-tested before adoption.

## 3. Dependency strategy

Preferred first integration:
- consume Moonframe from the exact audited Git commit;
- record the immutable SHA in the lockfile and dependency inventory;
- never use a floating main/master branch or mutable tag for production builds;
- use the repository's actual root package name;
- verify whether the Git dependency installs successfully as a package and whether its package files include all referenced CSS/JS/type exports.

Do not assume a public npm version exists. If registry publication becomes available, verify:
1. the package owner and provenance;
2. exact version and source commit relationship;
3. tarball contents and export map;
4. peer dependencies and React compatibility;
5. license and NOTICE/asset metadata;
6. clean install and consumer build from the registry artifact.

## 4. Initial consumption API

The expected public API surface is:

| Export | Use in RockCode |
|---|---|
| Root package | Shared React primitives and components documented by Moonframe |
| styles.css | Global Moonframe semantic CSS, imported once by each app entry |
| tailwind.css | Optional Tailwind v4 bridge when Tailwind is used |
| skins | Skin catalog and IDs for preference validation/display |
| theme | Theme provider, preference resolver and root theme attributes |

The integration must verify the exact signatures on the pinned source. Do not copy example imports or assume component props without checking exported declarations.

## 5. Theme policy

RockCode defaults:
- skin: moonpunk;
- appearance: dark;
- density: comfortable;
- motion: system/reduced-motion aware;
- contrast: normal unless OS/user requests high contrast.

Users may choose all Moonframe skins. Mooncrash and Moonledger are complete alternatives, not token overlays mixed into Moonpunk. Classic remains a restrained fallback.

RockCode must:
- use Moonframe's public ThemeProvider/resolver and root attributes;
- import the Moonframe stylesheet only once per app root;
- preserve system mode and independent density/motion/contrast preferences;
- avoid a second theme store/provider with competing keys;
- never hardcode a second global color palette;
- keep workspace/server policy separate from local accessibility preferences.

## 6. RockCode package boundaries

Proposed workspace packages:
- @rockcode/ui — product shell, navigation, brand integration and reusable layout.
- @rockcode/agent-ui — session, agent, tool, approval, task and model components.
- @rockcode/workbench — pane registration, docking, tabs and persisted view layout.
- @rockcode/editor — optional editor host adapter.
- @rockcode/terminal — optional terminal/PTY host adapter.
- @rockcode/brand-assets — original RockCode brand and decorative assets.
- @rockcode/ui-testkit — internal fixtures and accessibility helpers.

These are logical boundaries. Split the editor, terminal, asset and testkit packages only when dependency weight, lifecycle or ownership makes the split useful. Do not split every component family into a package.

## 7. CSS and asset integration

- Import Moonframe styles once in the app entry.
- Do not let each RockCode package import global Moonframe CSS.
- Product CSS must be namespaced and use Moonframe semantic variables.
- No raw hex colors in product components.
- No duplicated skin-specific stylesheet authored by RockCode.
- Heavy artwork and optional SFX are lazy/opt-in.
- All RockCode artwork has a provenance/license manifest.
- Decoration-off, reduced-motion, forced-colors and no-art fallbacks remain functional.
- No MoonWitness character, logo or consumer-specific visual asset is included in RockCode.

## 8. Required clean-consumer proof

Before M1 UI implementation:
1. Create a minimal disposable React 19 + Vite consumer.
2. Install Moonframe from the pinned Git SHA.
3. Import the root package, styles.css, and theme/skins subpaths.
4. Render a Button, Input, Dialog, Tabs, Table, Toast and theme selector.
5. Render Moonpunk Dark/Light and Classic Dark/Light.
6. Test SSR is not required for the Vite client, but confirm no browser-only import breaks the target runtime.
7. Build a minimal Tauri 2 shell around the same consumer.
8. Verify no duplicate React or duplicate global stylesheet.
9. Run keyboard/focus/reduced-motion smoke tests.
10. Record exact command output, resolved dependency tree and commit SHA.

If Git dependency installation fails because the upstream repository is a workspace or package root not prepared for consumer installs, do not patch around it with undocumented copies. Coordinate a clean distribution contract in Moonframe or choose a reviewed immutable tarball/source strategy.

## 9. M0 acceptance evidence

- Moonframe source pin is recorded.
- Package install and export-map smoke test passes.
- Root ThemeProvider is single-owner.
- No duplicate base component/token package is introduced in RockCode.
- The RockCode visual profile is linked from root README and specification index.
- The remaining Moonframe compatibility risks are explicit before UI package implementation.
