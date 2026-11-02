#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const redoclyVersion = "2.72.1";
const ajvCliVersion = "5.0.0";
const ajvFormatsVersion = "3.0.1";

let passedChecks = 0;
let failedChecks = 0;
let failureLines = 0;

function fail(message) {
  console.error("ERROR: " + message);
  failureLines += 1;
  process.exitCode = 1;
}

// A logical check contributes exactly one PASS or FAIL, measured by whether any
// ERROR line was printed while it ran. A check that prints many ERROR lines (for
// example, several broken links) still counts once.
function runCheck(fn) {
  const before = failureLines;
  fn();
  const ok = failureLines === before;
  if (ok) passedChecks += 1;
  else failedChecks += 1;
  return ok;
}

function npxCliCandidates() {
  const nodeDir = path.dirname(process.execPath);
  return [
    path.join(nodeDir, "node_modules", "npm", "bin", "npx-cli.js"),
    path.resolve(nodeDir, "..", "lib", "node_modules", "npm", "bin", "npx-cli.js")
  ];
}

let npxCliPath;
function resolveNpxCli() {
  if (npxCliPath !== undefined) return npxCliPath;
  const candidates = npxCliCandidates();
  npxCliPath = candidates.find((candidate) => existsSync(candidate));
  if (!npxCliPath) {
    fail("Cannot locate npx-cli.js. Searched: " + candidates.join(", "));
  }
  return npxCliPath;
}

// Runs a pinned tool through npx and returns the raw spawnSync result without
// echoing or judging it. Transport follows D3: Windows runs npx-cli.js under
// process.execPath (no shell); POSIX calls `npx` directly (proven by every CI run).
function runTool(packageName, commandArgs) {
  const packages = Array.isArray(packageName) ? packageName : [packageName];
  const packageArgs = packages.flatMap((name) => ["--package", name]);

  let file;
  let args;
  if (process.platform === "win32") {
    const cli = resolveNpxCli();
    if (!cli) return { error: new Error("npx-cli.js not found") };
    file = process.execPath;
    args = [cli, "--yes", ...packageArgs, ...commandArgs];
  } else {
    file = "npx";
    args = ["--yes", ...packageArgs, ...commandArgs];
  }

  return spawnSync(file, args, {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      REDOCLY_TELEMETRY: "off",
      REDOCLY_SUPPRESS_NOTIFICATIONS: "true",
      NO_COLOR: "1"
    },
    shell: false,
    windowsHide: true,
    maxBuffer: 64 * 1024 * 1024
  });
}

// Echoes output; a non-zero exit is a failed check. Used for tools whose verdict is
// fully expressed by their exit code (Ajv).
function runNpx(packageName, commandArgs, label) {
  console.log("\n[contract-validation] " + label);
  const result = runTool(packageName, commandArgs);

  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);

  if (result.error) {
    fail(label + ": " + result.error.message);
    return;
  }
  if (result.status !== 0) {
    fail(label + ": command exited with status " + (result.status ?? "unknown"));
    return;
  }
  if (result.signal) {
    fail(label + ": command was terminated by signal " + result.signal);
  }
}

// D1: any Redocly problem at any severity is a failure. The exit code is NOT the
// signal — a warnings-only document exits 0 while still carrying findings (§1.1),
// and that hole is exactly what this closes. We still fail when the exit code is
// non-zero with no findings (P1), so a fatal error is never mistaken for a pass.
function lintSpec(packageName, specPath, label) {
  console.log("\n[contract-validation] " + label);
  const result = runTool(packageName, ["redocly", "lint", specPath, "--format", "json"]);

  if (result.error) {
    fail(label + ": " + result.error.message);
    return;
  }

  let report;
  try {
    report = JSON.parse(result.stdout);
  } catch {
    const raw = (result.stdout || result.stderr || "").split("\n").slice(0, 20).join("\n");
    fail(label + ": --format json output could not be parsed. First lines:\n" + raw);
    return;
  }

  const problems = Object.values(report.problems || {}).flat();
  if (problems.length > 0) {
    for (const problem of problems) {
      const firstLocation = Array.isArray(problem.location) ? problem.location[0] : undefined;
      const where = firstLocation?.source?.ref
        ? firstLocation.source.ref + ":" + (firstLocation.start?.line ?? "?")
        : specPath;
      fail(label + ": [" + problem.severity + "] " + problem.ruleId + " at " + where + " — " + problem.message);
    }
    return;
  }

  if (result.status !== 0 || result.signal) {
    fail(label + ": no findings reported but lint exited " + (result.status ?? "signal " + result.signal));
    return;
  }

  console.log("No findings.");
}

function walk(directory, output = []) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.name === ".git" || entry.name === "node_modules") continue;
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) walk(absolute, output);
    else if (entry.isFile()) output.push(absolute);
  }
  return output;
}

function validateMarkdownLinks() {
  console.log("\n[contract-validation] Repository-relative Markdown links");
  const markdownFiles = walk(root).filter((file) => file.endsWith(".md"));
  let checked = 0;

  for (const file of markdownFiles) {
    const source = readFileSync(file, "utf8");
    const relativeSource = path.relative(root, file).split(path.sep).join("/");
    const matches = source.matchAll(/!?\[[^\]]*\]\(([^)\s]+)(?:\s+[^)]*)?\)/g);

    for (const match of matches) {
      const href = match[1].replace(/^<|>$/g, "");
      if (
        !href ||
        href.startsWith("#") ||
        /^(?:https?:|mailto:|tel:|data:|javascript:)/i.test(href)
      ) {
        continue;
      }

      const cleanHref = decodeURIComponent(href.split("#")[0].split("?")[0]);
      if (!cleanHref) continue;

      const target = cleanHref.startsWith("/")
        ? path.join(root, cleanHref.slice(1))
        : path.resolve(path.dirname(file), cleanHref);

      checked += 1;
      if (!existsSync(target)) {
        fail(relativeSource + ': broken relative link "' + href + '"');
      }
    }

    // Catch an escaped newline accidentally committed into a Markdown table.
    if (/\\n\|/.test(source)) {
      fail(relativeSource + ': contains literal "\\\\n|" sequence; use actual line breaks');
    }
  }

  console.log("Checked " + checked + " repository-relative links across " + markdownFiles.length + " Markdown files.");
}

function validateFixtureInventory() {
  console.log("\n[contract-validation] Fixture inventory");
  const schemaDirectory = path.join(root, "contracts/json-schema/v1");
  const fixtureDirectory = path.join(root, "contracts/fixtures/v1");
  const schemaFiles = readdirSync(schemaDirectory)
    .filter((name) => name.endsWith(".schema.json"))
    .sort();

  if (schemaFiles.length === 0) {
    fail("No JSON Schema files found.");
    return [];
  }

  return schemaFiles.map((schemaName) => {
    const stem = schemaName.replace(/\.schema\.json$/, "");
    const schema = path.join(schemaDirectory, schemaName);
    const valid = path.join(fixtureDirectory, stem + ".valid.json");
    const invalid = path.join(fixtureDirectory, stem + ".invalid.json");

    for (const fixture of [valid, invalid]) {
      if (!existsSync(fixture) || !statSync(fixture).isFile()) {
        fail("Missing fixture: " + path.relative(root, fixture));
      } else {
        try {
          JSON.parse(readFileSync(fixture, "utf8"));
        } catch (error) {
          fail("Invalid JSON in " + path.relative(root, fixture) + ": " + error.message);
        }
      }
    }

    return { schema, valid, invalid, stem };
  });
}

function main() {
  const openapi = "contracts/openapi/v1/openapi.yaml";
  const asyncapi = "contracts/asyncapi/v1/session-events.yaml";

  for (const file of [openapi, asyncapi]) {
    if (!existsSync(path.join(root, file))) fail("Missing API contract: " + file);
  }

  const redocly = "@redocly/cli@" + redoclyVersion;
  const ajv = ["ajv-cli@" + ajvCliVersion, "ajv-formats@" + ajvFormatsVersion];
  const ajvLabel = ajv.join(" + ");

  runCheck(validateMarkdownLinks);
  let fixtures = [];
  runCheck(() => {
    fixtures = validateFixtureInventory();
  });

  runCheck(() => lintSpec(redocly, openapi, "OpenAPI lint (" + redocly + ")"));
  runCheck(() => lintSpec(redocly, asyncapi, "AsyncAPI lint (" + redocly + ")"));

  for (const fixture of fixtures) {
    const schemaPath = path.relative(root, fixture.schema).split(path.sep).join("/");
    const validPath = path.relative(root, fixture.valid).split(path.sep).join("/");
    const invalidPath = path.relative(root, fixture.invalid).split(path.sep).join("/");

    runCheck(() =>
      runNpx(
        ajv,
        ["ajv", "compile", "--spec=draft2020", "-c", "ajv-formats", "-s", schemaPath],
        "Compile " + fixture.stem + " schema (" + ajvLabel + ")"
      )
    );
    runCheck(() =>
      runNpx(
        ajv,
        ["ajv", "test", "--spec=draft2020", "-c", "ajv-formats", "-s", schemaPath, "-d", validPath, "--valid"],
        "Accept valid " + fixture.stem + " fixture (" + ajvLabel + ")"
      )
    );
    runCheck(() =>
      runNpx(
        ajv,
        ["ajv", "test", "--spec=draft2020", "-c", "ajv-formats", "-s", schemaPath, "-d", invalidPath, "--invalid"],
        "Reject invalid " + fixture.stem + " fixture (" + ajvLabel + ")"
      )
    );
  }

  console.log("\n[contract-validation] PASS " + passedChecks + " / FAIL " + failedChecks);

  if (failedChecks > 0 || process.exitCode === 1) {
    console.error("\nContract validation failed. Fix the source contract or fixture; do not weaken checks without documenting why.");
  } else {
    console.log("All contract validation checks passed.");
  }
}

main();
