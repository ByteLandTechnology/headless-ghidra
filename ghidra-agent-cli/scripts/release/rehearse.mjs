#!/usr/bin/env node
// Rehearses the full release pipeline locally without pushing tags, publishing
// to npm, or creating a GitHub Release. Builds the binaries for every
// configured target, syncs the platform packages, then runs `npm publish
// --dry-run` for each one. Exits 0 when every step succeeds.
//
// On success or failure, all generated artifacts and all mutated files are
// restored so the working tree is identical to the pre-rehearsal state.
// This includes dist/, npm/platforms/, npm/main/package.json,
// npm/main/README.md. Build outputs go to an isolated temporary directory
// (CARGO_TARGET_DIR) so the real target/ is never touched.
//
// Prerequisites: cargo, rustup, and the cross-build toolchain must already be
// installed locally (zig + cargo-zigbuild for Linux targets, llvm-mingw for
// Windows targets). CI gets these from .github/actions/setup-build-env.
//
// Usage: node scripts/release/rehearse.mjs

import { existsSync, readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildMainPackageName,
  readReleaseConfig,
} from "./release-config.mjs";
import { createLocalReleaseWorkspace } from "./local-build-utils.mjs";

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
const config = readReleaseConfig(rootDir);
const pkgName = buildMainPackageName(config);
const rehearsalVersion = "0.0.0-rehearsal";
const workspace = createLocalReleaseWorkspace(rootDir);

// --- Step 1: Build ---------------------------------------------------------
try {
  workspace.validateConfig();
  workspace.runBuildPreflight();
  workspace.buildDist();

  // --- Step 2: Sync platform packages ----------------------------------------
  console.log("\n=== Step 2: Sync platform packages ===\n");
  const syncResult = spawnSync(
    process.execPath,
    ["scripts/release/sync-platform-packages.mjs", rehearsalVersion],
    { cwd: rootDir, stdio: "inherit" },
  );
  if (syncResult.status !== 0) {
    throw new Error(
      `sync-platform-packages failed (exit ${syncResult.status}).`,
    );
  }

  // --- Step 3: npm publish --dry-run for every package -----------------------
  console.log("\n=== Step 3: npm publish --dry-run ===\n");

  for (const target of config.targets) {
    const pkgDir = path.join(workspace.platformsDir, target.packageSuffix);
    if (!existsSync(pkgDir)) {
      throw new Error(`Platform package missing: ${pkgDir}`);
    }
    const name = JSON.parse(
      readFileSync(path.join(pkgDir, "package.json"), "utf8"),
    ).name;
    console.log(`  ${name}@${rehearsalVersion}`);
    const r = spawnSync("npm", ["publish", "--dry-run", "--access=public"], {
      cwd: pkgDir,
      stdio: "inherit",
      shell: true,
    });
    if (r.status !== 0) {
      throw new Error(
        `npm publish --dry-run failed for ${name} (exit ${r.status}).`,
      );
    }
  }

  // Main package
  console.log(`  ${pkgName}@${rehearsalVersion}`);
  const mainR = spawnSync("npm", ["publish", "--dry-run", "--access=public"], {
    cwd: path.join(rootDir, "npm/main"),
    stdio: "inherit",
    shell: true,
  });
  if (mainR.status !== 0) {
    throw new Error(
      `npm publish --dry-run failed for main package (exit ${mainR.status}).`,
    );
  }

  console.log(
    "\n=== Rehearsal complete. No tags, no npm publishes, no GitHub Release. ===\n",
  );
} finally {
  workspace.cleanup();
}
