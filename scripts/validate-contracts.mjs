#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const redoclyVersion = "2.72.1";
const ajvCliVersion = "5.0.0";

function fail(message) {
  console.error("ERROR: " + message);
  process.exitCode = 1;
}

function runNpx(packageName, commandArgs, label) {
  const executable = process.platform === "win32" ? "npx.cmd" : "npx";
  console.log("\n[contract-validation] " + label);
  const result = spawnSync(
    executable,
    ["--yes", "--package", packageName, ...commandArgs],
    {
      cwd: root,
      env: {
        ...process.env,
        REDOCLY_TELEMETRY: "off",
        REDOCLY_SUPPRESS_NOTIFICATIONS: "true",
        NO_COLOR: "1"
      },
      stdio: "inherit",
      windowsHide: true
    }
  );

  if (result.error) {
    fail(label + ": " + result.error.message);
    return false;
  }
  if (result.status !== 0) {
    fail(label + ": command exited with status " + (result.status ?? "unknown"));
    return false;
  }
  return true;
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

  validateMarkdownLinks();
  const fixtures = validateFixtureInventory();

  const redocly = "@redocly/cli@" + redoclyVersion;
  const ajv = "ajv-cli@" + ajvCliVersion;

  runNpx(redocly, ["redocly", "lint", openapi], "OpenAPI lint (" + redocly + ")");
  runNpx(redocly, ["redocly", "lint", asyncapi], "AsyncAPI lint (" + redocly + ")");

  for (const fixture of fixtures) {
    const schemaPath = path.relative(root, fixture.schema).split(path.sep).join("/");
    const validPath = path.relative(root, fixture.valid).split(path.sep).join("/");
    const invalidPath = path.relative(root, fixture.invalid).split(path.sep).join("/");

    runNpx(
      ajv,
      ["ajv", "compile", "--spec=draft2020", "-s", schemaPath],
      "Compile " + fixture.stem + " schema (" + ajv + ")"
    );
    runNpx(
      ajv,
      ["ajv", "test", "--spec=draft2020", "-s", schemaPath, "-d", validPath, "--valid"],
      "Accept valid " + fixture.stem + " fixture"
    );
    runNpx(
      ajv,
      ["ajv", "test", "--spec=draft2020", "-s", schemaPath, "-d", invalidPath, "--invalid"],
      "Reject invalid " + fixture.stem + " fixture"
    );
  }

  if (process.exitCode === 1) {
    console.error("\nContract validation failed. Fix the source contract or fixture; do not weaken checks without documenting why.");
  } else {
    console.log("\nAll contract validation checks passed.");
  }
}

main();
