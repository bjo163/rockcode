# M0-09 — Validation Status & Audit Log

**Repository:** [bjo163/rockcode](https://github.com/bjo163/rockcode)  
**Branch:** main  
**Audit date:** 2 November 2026  
**Purpose:** Record exactly what was verified while preparing M0 and what remains unverified.

## 1. Repository and artifact verification

| Check | Result | Evidence / limitation |
|---|---|---|
| M0 contract documents present | PASS | Eight numbered contract docs plus this log are present under docs/specs/m0/ |
| Root README links M0 package | PASS | Link resolves to docs/specs/m0/README.md |
| Specification index links M0 package | PASS | Link resolves from docs/specs/README.md |
| Architecture specification references M0 | PASS | M0 amendment is appended to 03-final-architecture-specification.md |
| Machine contract files present | PASS | OpenAPI, AsyncAPI and five JSON Schema files are in contracts/ |
| JSON syntax | PASS | All five JSON Schema documents were fetched from main and parsed with JSON.parse |
| JSON Schema required/property structure | PASS | Every root required key has a matching root property; this is a structural check only |
| Local Markdown links | PASS | 12 main documentation files were checked against the repository tree; zero missing relative targets were found |
| OpenAPI internal component references | PASS — structural inspection | Referenced component names were checked against the authored document; no missing component names were detected |
| External schema references | PASS — path inspection | OpenAPI and AsyncAPI relative refs resolve to files present in the repository tree |
| OpenAPI YAML parser/linter | NOT RUN | Requires a dedicated OpenAPI/YAML validator |
| AsyncAPI YAML parser/linter | NOT RUN | Requires a dedicated AsyncAPI/YAML validator |
| Semantic JSON Schema validator | NOT RUN | JSON parsing does not prove draft-2020-12 schema validity or fixture behavior |
| Cargo build/tests | NOT RUN | No Rust source or Cargo workspace has been created |
| React/Tauri consumer build | NOT RUN | No RockCode UI packages/apps have been created |
| Moonframe consumer install | NOT RUN | Integration dependency is specified and pinned as a proposed source baseline; clean consumer proof remains pending |
| Windows sandbox adversarial tests | NOT RUN | No executor or sandbox code exists yet |
| Linux container/remote worker POC | NOT RUN | No worker implementation exists yet |

## 2. JSON schema files inspected

- contracts/json-schema/v1/event-envelope.schema.json
- contracts/json-schema/v1/session.schema.json
- contracts/json-schema/v1/approval-request.schema.json
- contracts/json-schema/v1/plugin-manifest.schema.json
- contracts/json-schema/v1/executor-capability-report.schema.json

An event-envelope invariant was corrected during authoring: the envelope must contain exactly one of inline payload or payload_ref. The corrected file is on main.

## 3. Known remaining validation gaps

1. Run an OpenAPI 3.1 parser/linter against contracts/openapi/v1/openapi.yaml.
2. Run an AsyncAPI 3.0 parser/linter against contracts/asyncapi/v1/session-events.yaml.
3. Run a JSON Schema draft-2020-12 validator with positive and negative fixtures for all five schemas.
4. Add generated TypeScript type validation and compatibility checks.
5. Run a full repository-relative link/anchor check in CI.
6. Run POC-01 through POC-07 as the first M1 coding sequence.
7. Reconfirm exact Windows 10 servicing baseline and available sandbox implementation before enabling arbitrary shell execution.
8. Run the Moonframe clean consumer install and Tauri/Web shell test at the audited source SHA.

## 4. Stage decision

**M0 specification authoring: COMPLETE.**  
**M0 machine-contract validation: PARTIAL / NOT CLOSED.**  
**M1 source coding: NOT STARTED.**

The architecture has reached the coding-ready handoff boundary, but coding must start with contract validation tooling and the bounded proof-of-concept vertical slice. This log does not claim build, unit tests, E2E, sandbox, or security tests passed.


---

## 5. M1.1 contract-validation tooling scaffold

**Added:** 2 November 2026

| Artifact | Status |
|---|---|
| scripts/validate-contracts.mjs | Committed |
| .github/workflows/contracts.yml | Committed; push/PR/manual triggers |
| .redocly.yaml | Committed |
| Five positive JSON fixtures | Committed |
| Five negative JSON fixtures | Committed |
| Pinned Redocly CLI 2.72.1 | Configured; runtime not yet verified |
| Pinned Ajv CLI 5.0.0 | Configured; runtime not yet verified |
| Successful GitHub Actions workflow run | Pending |

The workflow validates OpenAPI, AsyncAPI, five Draft 2020-12 schemas with positive/negative fixtures, and repository-relative Markdown links. The workflow source has been committed, but a committed workflow is not evidence that the runner passed. M1.1 is **implemented as a CI scaffold, awaiting execution evidence**.
