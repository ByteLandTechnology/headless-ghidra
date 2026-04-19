#!/usr/bin/env node
// Local bootstrap publication for first-time npm setup. Publishes every
// platform package, then the main wrapper package, at a dedicated prerelease
// version without creating a git tag, changelog entry, or GitHub Release.

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { createLocalReleaseWorkspace } from "./local-build-utils.mjs";
import {
  buildAllPackageNames,
  buildMainPackageName,
  readReleaseConfig,
} from "./release-config.mjs";

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
const explicitVersion = process.argv[2] ?? null;
const PREPUBLISH_VERSION_RE = /^0\.0\.0-prepublish\.\d+$/;

function validateConfigNow() {
  const validateResult = spawnSync(
    process.execPath,
    [path.join(rootDir, "scripts/release/validate-config.mjs")],
    { cwd: rootDir, stdio: "inherit" },
  );
  if (validateResult.status !== 0) {
    throw new Error("Config validation failed.");
  }
}

function registryHasVersion(pkgName, pkgVersion) {
  const result = spawnSync("npm", ["view", `${pkgName}@${pkgVersion}`, "version"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    shell: true,
  });
  if (result.status === 0) {
    return result.stdout.trim() === pkgVersion;
  }

  const stderr = (result.stderr ?? "").toLowerCase();
  const notFound =
    stderr.includes("e404") ||
    stderr.includes("etarget") ||
    stderr.includes("not found");
  if (notFound) return false;

  throw new Error(
    `npm view ${pkgName}@${pkgVersion} failed with unexpected error:\n${result.stderr?.trim() ?? "(no stderr)"}`,
  );
}

function choosePrepublishVersion(packageNames) {
  if (explicitVersion) {
    if (!PREPUBLISH_VERSION_RE.test(explicitVersion)) {
      throw new Error(
        `Explicit prepublish version ${JSON.stringify(explicitVersion)} is not allowed. Use the reserved prepublish lane: 0.0.0-prepublish.N`,
      );
    }
    return explicitVersion;
  }

  for (let n = 1; n < 10_000; n += 1) {
    const candidate = `0.0.0-prepublish.${n}`;
    if (packageNames.every((pkgName) => !registryHasVersion(pkgName, candidate))) {
      return candidate;
    }
  }

  throw new Error(
    "Could not find an unused prepublish version. Pass one explicitly, for example: node scripts/release/prepublish.mjs 0.0.0-prepublish.42",
  );
}

function assertVersionAvailable(packageNames, version) {
  const conflicts = packageNames.filter((pkgName) =>
    registryHasVersion(pkgName, version),
  );
  if (!conflicts.length) return;

  throw new Error(
    `Prepublish version ${version} is already present for: ${conflicts.join(", ")}. Choose a new prerelease version and retry.`,
  );
}

function publishPackage(pkgDir, label, version) {
  const pkgName = JSON.parse(
    readFileSync(path.join(pkgDir, "package.json"), "utf8"),
  ).name;

  if (registryHasVersion(pkgName, version)) {
    throw new Error(
      `Refusing to publish prepublish package ${pkgName}@${version} because that version already exists on npm.`,
    );
  }

  const result = spawnSync("npm", ["publish", "--access=public", "--tag", "prepublish"], {
    cwd: pkgDir,
    stdio: "inherit",
    shell: true,
  });
  if (result.status !== 0) {
    throw new Error(
      `npm publish failed for ${label} package ${pkgName}@${version} (exit ${result.status}).`,
    );
  }
}

function disableLocalProvenance(pkgDir) {
  const pkgPath = path.join(pkgDir, "package.json");
  const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
  pkg.publishConfig = { ...(pkg.publishConfig ?? {}), access: "public" };
  delete pkg.publishConfig.provenance;
  writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`, "utf8");
}

validateConfigNow();
const config = readReleaseConfig(rootDir);
const packageNames = buildAllPackageNames(config);
const prepublishVersion = choosePrepublishVersion(packageNames);

console.log(`Selected prepublish version: ${prepublishVersion}`);

const loginResult = spawnSync(
  process.execPath,
  [path.join(rootDir, "scripts/release/ensure-npm-login.mjs")],
  { cwd: rootDir, stdio: "inherit" },
);
if (loginResult.status !== 0) {
  throw new Error("npm login is required before prepublish can continue.");
}

assertVersionAvailable(packageNames, prepublishVersion);

const workspace = createLocalReleaseWorkspace(rootDir);
try {
  workspace.runBuildPreflight();
  workspace.stageCargoVersion(prepublishVersion);
  workspace.buildDist();

  console.log("\n=== Step 2: Sync platform packages ===\n");
  const syncResult = spawnSync(
    process.execPath,
    ["scripts/release/sync-platform-packages.mjs", prepublishVersion],
    { cwd: rootDir, stdio: "inherit" },
  );
  if (syncResult.status !== 0) {
    throw new Error(
      `sync-platform-packages failed (exit ${syncResult.status}).`,
    );
  }

  console.log("\n=== Step 3: npm publish --access=public ===\n");
  for (const target of config.targets) {
    const pkgDir = path.join(workspace.platformsDir, target.packageSuffix);
    if (!existsSync(pkgDir)) {
      throw new Error(`Platform package missing: ${pkgDir}`);
    }
    // Local prepublish runs are interactive bootstrap events, not CI-backed
    // provenance-bearing releases.
    disableLocalProvenance(pkgDir);
    publishPackage(pkgDir, "platform", prepublishVersion);
  }

  const mainPkgDir = path.join(rootDir, "npm/main");
  disableLocalProvenance(mainPkgDir);
  publishPackage(mainPkgDir, "main", prepublishVersion);

  console.log(
    `\n=== Prepublish complete: ${buildMainPackageName(config)}@${prepublishVersion} plus ${config.targets.length} platform packages. ===\n`,
  );
} finally {
  workspace.cleanup();
}
