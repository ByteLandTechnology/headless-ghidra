#!/usr/bin/env node
// Runs during semantic-release `publish`. Publishes every per-platform
// package, then the main wrapper package. Both are idempotent on re-run for
// the same version: each package is probed via `npm view <pkg>@<version>`
// before publishing so a partial failure can be retried without re-failing on
// packages that already made it to the registry.
//
// `.releaserc.json` sets @semantic-release/npm `npmPublish: false`; that
// plugin only handles version bumping during `prepare`, and this script owns
// every `npm publish` invocation for the release.

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildMainPackageName,
  readReleaseConfig,
} from "./release-config.mjs";

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
const version = process.argv[2];
if (!version) {
  throw new Error("Usage: publish-npm-packages.mjs <version>");
}

const config = readReleaseConfig(rootDir);
const platformsDir = path.join(rootDir, "npm/platforms");
const mainPkgDir = path.join(rootDir, "npm/main");

function alreadyPublished(pkgName, pkgVersion) {
  const r = spawnSync("npm", ["view", `${pkgName}@${pkgVersion}`, "version"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    shell: true,
  });
  if (r.status === 0) return r.stdout.trim() === pkgVersion;

  const stderr = (r.stderr ?? "").toLowerCase();
  // Only treat explicit "not found" indicators as "not published".
  const notFound =
    stderr.includes("e404") ||
    stderr.includes("etarget") ||
    stderr.includes("not found");
  if (notFound) return false;

  // Anything else (network, auth, rate-limit) is ambiguous — fail fast.
  throw new Error(
    `npm view ${pkgName}@${pkgVersion} failed with unexpected error:\n${r.stderr?.trim() ?? "(no stderr)"}`,
  );
}

function publishPackage(pkgDir, label) {
  const pkgName = JSON.parse(
    readFileSync(path.join(pkgDir, "package.json"), "utf8"),
  ).name;

  if (alreadyPublished(pkgName, version)) {
    console.log(`skip ${label} ${pkgName}@${version} (already on registry)`);
    return "skipped";
  }

  const result = spawnSync("npm", ["publish", "--access=public"], {
    cwd: pkgDir,
    stdio: "inherit",
    shell: true,
  });
  if (result.status !== 0) {
    throw new Error(
      `npm publish failed for ${pkgName}@${version} (exit ${result.status}).`,
    );
  }
  return "published";
}

let platformPublished = 0;
let platformSkipped = 0;

for (const target of config.targets) {
  const pkgDir = path.join(platformsDir, target.packageSuffix);
  if (!existsSync(pkgDir)) {
    throw new Error(
      `Platform package missing: ${pkgDir}. Did sync-platform-packages.mjs run?`,
    );
  }
  const outcome = publishPackage(pkgDir, "platform");
  if (outcome === "published") platformPublished += 1;
  else platformSkipped += 1;
}

const mainOutcome = publishPackage(mainPkgDir, "main");

console.log(
  `npm publish: main=${buildMainPackageName(config)} => ${mainOutcome}; platform=${platformPublished} published, ${platformSkipped} skipped.`,
);
