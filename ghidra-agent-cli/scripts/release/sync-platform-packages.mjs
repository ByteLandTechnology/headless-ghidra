#!/usr/bin/env node
// Runs during semantic-release `prepare`. Treats release/config.json as the
// authoritative source for CLI name, main package name, and npm scope.
//
// For each configured target:
//   - materialize npm/platforms/<pkg-suffix>/ with name, os, cpu, bin, version
//   - copy the prebuilt binary from dist/<rustTarget>/ into the platform package
// Then rewrites npm/main/package.json so its name / bin / optionalDependencies
// are derived from release/config.json, with hard failure on scope mismatch.

import {
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildMainPackageName,
  buildPlatformPackageName,
  readReleaseConfig,
} from "./release-config.mjs";

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
const version = process.argv[2];
if (!version) {
  throw new Error("Usage: sync-platform-packages.mjs <version>");
}

const config = readReleaseConfig(rootDir);
const repoUrl = `https://github.com/${config.sourceRepository}`;
const BUNDLED_ENTRY_SCRIPT = "GhidraAgentCliEntry.java";
const BUNDLED_SCRIPT_JAR = "ghidra-agent-cli-ghidra-scripts.jar";
const BUNDLED_DEPENDENCY_JARS = ["snakeyaml-2.6.jar"];

// Delegate all field/scope validation to the shared script so the checks
// are identical in CI (release.yml) and local runs (sync-platform-packages).
const validateResult = spawnSync(
  process.execPath,
  [path.join(rootDir, "scripts/release/validate-config.mjs")],
  { cwd: rootDir, stdio: "inherit" },
);
if (validateResult.status !== 0) {
  throw new Error("Config validation failed.");
}

const { cliName } = config;
const mainPackageName = buildMainPackageName(config);

const mainPkgPath = path.join(rootDir, "npm/main/package.json");
const mainPkg = JSON.parse(readFileSync(mainPkgPath, "utf8"));

const platformsDir = path.join(rootDir, "npm/platforms");
const distDir = path.join(rootDir, "dist");
mkdirSync(platformsDir, { recursive: true });

const optionalDeps = {};

function assertBundleContents(bundleDir) {
  const required = [
    BUNDLED_ENTRY_SCRIPT,
    BUNDLED_SCRIPT_JAR,
    ...BUNDLED_DEPENDENCY_JARS,
  ];
  for (const relPath of required) {
    const fullPath = path.join(bundleDir, relPath);
    if (!existsSync(fullPath)) {
      throw new Error(`Incomplete Ghidra script bundle: missing ${fullPath}`);
    }
  }
}

function assertMainPackageBundlesScripts(mainPkgDir) {
  const npmCacheDir = mkdtempSync(
    path.join(tmpdir(), "ghidra-agent-cli-npm-pack-"),
  );
  const result = spawnSync("npm", ["pack", "--dry-run", "--json"], {
    cwd: mainPkgDir,
    encoding: "utf8",
    env: { ...process.env, npm_config_cache: npmCacheDir },
    stdio: ["ignore", "pipe", "pipe"],
  });
  try {
    if (result.status !== 0) {
      throw new Error(
        `npm pack --dry-run failed for ${mainPkgDir}:\n${result.stderr?.trim() ?? "(no stderr)"}`,
      );
    }

    let packOutput;
    try {
      packOutput = JSON.parse(result.stdout);
    } catch (error) {
      throw new Error(
        `Unable to parse npm pack --json output: ${error.message}`,
      );
    }

    const files = Array.isArray(packOutput) && packOutput.length > 0
      ? packOutput[0].files ?? []
      : [];
    const packedPaths = new Set(files.map((entry) => entry.path));
    for (const relPath of [
      `ghidra-script-bundle/${BUNDLED_ENTRY_SCRIPT}`,
      `ghidra-script-bundle/${BUNDLED_SCRIPT_JAR}`,
      ...BUNDLED_DEPENDENCY_JARS.map((name) => `ghidra-script-bundle/${name}`),
    ]) {
      if (!packedPaths.has(relPath)) {
        throw new Error(
          `Main npm package is missing bundled script artifact ${relPath}`,
        );
      }
    }
  } finally {
    rmSync(npmCacheDir, { recursive: true, force: true });
  }
}

for (const target of config.targets) {
  const pkgName = buildPlatformPackageName(config, target);
  const pkgDir = path.join(platformsDir, target.packageSuffix);
  const binDir = path.join(pkgDir, "bin");

  if (existsSync(pkgDir)) rmSync(pkgDir, { recursive: true });
  mkdirSync(binDir, { recursive: true });

  const binaryBaseName = `${cliName}${target.os === "win32" ? ".exe" : ""}`;
  const src = path.join(distDir, target.rustTarget, binaryBaseName);
  if (!existsSync(src)) {
    throw new Error(`Missing prebuilt binary: ${src}`);
  }
  copyFileSync(src, path.join(binDir, binaryBaseName));

  const pkgManifest = {
    name: pkgName,
    version,
    description:
      target.os === "linux"
        ? `${target.os}-${target.cpu} (static) binary for ${cliName}`
        : `${target.os}-${target.cpu} binary for ${cliName}`,
    license: mainPkg.license ?? "UNLICENSED",
    os: [target.os],
    cpu: [target.cpu],
    bin: { [cliName]: `bin/${binaryBaseName}` },
    files: ["bin/"],
    publishConfig: mainPkg.publishConfig ?? {
      access: "public",
      provenance: true,
    },
    repository: { type: "git", url: repoUrl },
  };
  writeFileSync(
    path.join(pkgDir, "package.json"),
    `${JSON.stringify(pkgManifest, null, 2)}\n`,
    "utf8",
  );
  writeFileSync(
    path.join(pkgDir, "README.md"),
    `# ${pkgName}\n\n${target.os}-${target.cpu} binary for \`${cliName}\`.\nRuntime dependency of \`${mainPackageName}\`.\n`,
    "utf8",
  );

  optionalDeps[pkgName] = version;
}

mainPkg.name = mainPackageName;
mainPkg.version = version;
mainPkg.bin = { [cliName]: "bin/cli.js" };
mainPkg.optionalDependencies = optionalDeps;
writeFileSync(mainPkgPath, `${JSON.stringify(mainPkg, null, 2)}\n`, "utf8");

// Copy the prebuilt Ghidra script bundle into npm/main/.
const srcBundleDir = path.join(distDir, "ghidra-script-bundle");
const dstBundleDir = path.join(rootDir, "npm", "main", "ghidra-script-bundle");
if (existsSync(dstBundleDir)) {
  rmSync(dstBundleDir, { recursive: true });
}
if (existsSync(path.join(rootDir, "npm", "main", "ghidra-scripts"))) {
  rmSync(path.join(rootDir, "npm", "main", "ghidra-scripts"), {
    recursive: true,
  });
}
if (existsSync(srcBundleDir)) {
  assertBundleContents(srcBundleDir);
  cpSync(srcBundleDir, dstBundleDir, { recursive: true });
  console.log("Copied ghidra-script-bundle/ into npm/main/.");
} else {
  throw new Error(
    `Missing prebuilt Ghidra script bundle: ${srcBundleDir}`,
  );
}

// Fill npm/main/README.md placeholders
const mainReadmePath = path.join(rootDir, "npm/main/README.md");
if (existsSync(mainReadmePath)) {
  let readme = readFileSync(mainReadmePath, "utf8");
  readme = readme
    .replace(/REPLACE_WITH_PACKAGE_NAME/g, mainPackageName)
    .replace(/REPLACE_WITH_CLI_NAME/g, cliName);
  if (/REPLACE_WITH_/.test(readme)) {
    throw new Error(
      "npm/main/README.md still contains REPLACE_WITH_ placeholders after substitution.",
    );
  }
  writeFileSync(mainReadmePath, readme, "utf8");
}

assertMainPackageBundlesScripts(path.join(rootDir, "npm", "main"));

// Update SKILL.md npm install version to match the release
const skillPath = path.join(rootDir, "SKILL.md");
if (existsSync(skillPath)) {
  let skill = readFileSync(skillPath, "utf8");
  skill = skill.replace(
    /ghidra-agent-cli@\d+\.\d+\.\d+[\w.-]*/,
    `ghidra-agent-cli@${version}`,
  );
  writeFileSync(skillPath, skill, "utf8");
}

console.log(
  `Synced ${config.targets.length} platform packages for ${mainPackageName}@${version}.`,
);
