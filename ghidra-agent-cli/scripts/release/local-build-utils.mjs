#!/usr/bin/env node
// Shared local-only build helpers for rehearsal and prepublish publication.

import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { tmpdir } from "node:os";

import { readReleaseConfig } from "./release-config.mjs";

function snapshotDir(dir) {
  const existed = existsSync(dir);
  const entries = new Map();
  if (!existed) return { existed, entries };

  const queue = [""];
  while (queue.length) {
    const rel = queue.shift();
    const full = rel ? path.join(dir, rel) : dir;
    const st = statSync(full);
    if (st.isDirectory()) {
      for (const child of readdirSync(full)) {
        queue.push(rel ? `${rel}/${child}` : child);
      }
    } else {
      entries.set(rel, readFileSync(full));
    }
  }

  return { existed, entries };
}

function restoreDir(dir, snapshot) {
  rmSync(dir, { recursive: true, force: true });
  if (!snapshot.existed) return;

  mkdirSync(dir, { recursive: true });
  for (const [rel, content] of snapshot.entries) {
    const full = path.join(dir, rel);
    mkdirSync(path.dirname(full), { recursive: true });
    writeFileSync(full, content);
  }
}

export function createLocalReleaseWorkspace(rootDir) {
  const config = readReleaseConfig(rootDir);
  const distDir = path.join(rootDir, "dist");
  const platformsDir = path.join(rootDir, "npm/platforms");
  const mainBundleDir = path.join(rootDir, "npm/main/ghidra-script-bundle");
  const cargoTomlPath = path.join(rootDir, "Cargo.toml");
  const cargoLockPath = path.join(rootDir, "Cargo.lock");
  const mainPkgPath = path.join(rootDir, "npm/main/package.json");
  const mainReadmePath = path.join(rootDir, "npm/main/README.md");
  const skillPath = path.join(rootDir, "SKILL.md");
  const isolatedTargetDir = mkdtempSync(
    path.join(tmpdir(), "cli-forge-local-release-"),
  );

  const fileSnapshots = new Map();
  const trackedFilePaths = [
    cargoTomlPath,
    cargoLockPath,
    mainPkgPath,
    mainReadmePath,
    skillPath,
  ];
  for (const filePath of trackedFilePaths) {
    if (existsSync(filePath)) {
      fileSnapshots.set(filePath, readFileSync(filePath));
    }
  }

  const distSnapshot = snapshotDir(distDir);
  const platformsSnapshot = snapshotDir(platformsDir);
  const mainBundleSnapshot = snapshotDir(mainBundleDir);

  let restored = false;
  const sigintHandler = () => {
    restore();
    process.exit(130);
  };

  function restore() {
    if (restored) return;
    restored = true;

    for (const filePath of trackedFilePaths) {
      const content = fileSnapshots.get(filePath);
      if (content) {
        writeFileSync(filePath, content);
      } else {
        rmSync(filePath, { force: true });
      }
    }
    restoreDir(distDir, distSnapshot);
    restoreDir(platformsDir, platformsSnapshot);
    restoreDir(mainBundleDir, mainBundleSnapshot);
    rmSync(isolatedTargetDir, { recursive: true, force: true });
  }

  function cleanup() {
    process.removeListener("SIGINT", sigintHandler);
    restore();
  }

  function validateConfig() {
    const validateResult = spawnSync(
      process.execPath,
      [path.join(rootDir, "scripts/release/validate-config.mjs")],
      { cwd: rootDir, stdio: "inherit" },
    );
    if (validateResult.status !== 0) {
      throw new Error("Config validation failed.");
    }
  }

  function runBuildPreflight() {
    console.log("\n=== Preflight ===\n");

    const hasLinux = config.targets.some((target) =>
      target.rustTarget.includes("linux"),
    );
    const hasWindows = config.targets.some((target) =>
      target.rustTarget.includes("windows"),
    );

    if (
      spawnSync("cargo", ["--version"], { encoding: "utf8", shell: true })
        .status !== 0
    ) {
      throw new Error("cargo not found. Install Rust: https://rustup.rs");
    }
    console.log("  cargo: OK");

    if (hasLinux) {
      const zigcheck = spawnSync("cargo-zigbuild", ["--version"], {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
        shell: true,
      });
      if (zigcheck.status !== 0) {
        throw new Error(
          "cargo-zigbuild not found. Install: cargo install cargo-zigbuild && brew install zig (macOS)",
        );
      }
      console.log("  cargo-zigbuild: OK");
    }

    const installedTargets =
      spawnSync("rustup", ["target", "list", "--installed"], {
        encoding: "utf8",
        shell: true,
      }).stdout ?? "";

    if (hasWindows) {
      const winTargets = config.targets
        .filter((target) => target.rustTarget.includes("windows"))
        .map((target) => target.rustTarget);
      for (const rustTarget of winTargets) {
        if (!installedTargets.includes(rustTarget)) {
          throw new Error(
            `Rust target ${rustTarget} not installed. Run: rustup target add ${rustTarget}`,
          );
        }
      }
      console.log("  windows targets: OK");
    }

    for (const target of config.targets) {
      if (!installedTargets.includes(target.rustTarget)) {
        throw new Error(
          `Rust target ${target.rustTarget} not installed. Run: rustup target add ${target.rustTarget}`,
        );
      }
    }
    console.log("  all rust targets: OK\n");

    resolveGhidraInstallDir();
    console.log("  ghidra: OK\n");
  }

  function resolveGhidraInstallDir() {
    const discoverScript = path.join(
      rootDir,
      "..",
      "headless-ghidra",
      "scripts",
      "discover-ghidra.sh",
    );
    if (!existsSync(discoverScript)) {
      throw new Error(`Ghidra discovery script not found: ${discoverScript}`);
    }

    const result = spawnSync(
      "bash",
      [discoverScript, "--print-install-dir"],
      {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      },
    );
    if (result.status !== 0) {
      const stderr = result.stderr?.trim() || "unknown error";
      throw new Error(
        `Unable to locate a Ghidra installation for script bundle compilation: ${stderr}`,
      );
    }

    return result.stdout.trim();
  }

  function buildGhidraScriptBundle() {
    const ghidraDir = resolveGhidraInstallDir();
    const sourceDir = path.join(rootDir, "ghidra-scripts");
    const outputDir = path.join(distDir, "ghidra-script-bundle");
    const bundleBuilder = path.join(sourceDir, "build-bundle.sh");

    if (!existsSync(bundleBuilder)) {
      throw new Error(`Bundle builder not found at ${bundleBuilder}.`);
    }

    rmSync(outputDir, { recursive: true, force: true });
    mkdirSync(outputDir, { recursive: true });

    console.log(`Building Ghidra script bundle with ${ghidraDir}...`);
    const result = spawnSync(
      "bash",
      [
        bundleBuilder,
        "--ghidra-dir",
        ghidraDir,
        "--source-dir",
        sourceDir,
        "--output-dir",
        outputDir,
      ],
      {
        stdio: "inherit",
      },
    );
    if (result.error) {
      throw new Error(
        `script bundle build failed: ${result.error.message}`,
      );
    }
    if (result.status !== 0) {
      throw new Error(
        `script bundle build failed (exit ${result.status}).`,
      );
    }
  }

  function buildDist() {
    console.log("=== Step 1: Build ===\n");
    mkdirSync(distDir, { recursive: true });
    buildGhidraScriptBundle();

    for (const target of config.targets) {
      const rustTarget = target.rustTarget;
      console.log(`Building ${rustTarget}...`);

      const isWindows = rustTarget.includes("windows");
      const isLinux = rustTarget.includes("linux");
      const binaryName = `${config.cliName}${isWindows ? ".exe" : ""}`;
      const outDir = path.join(distDir, rustTarget);
      mkdirSync(outDir, { recursive: true });

      const buildArgs = ["build", "--release", "--target", rustTarget];
      if (isLinux) {
        buildArgs[0] = "zigbuild";
      }

      const buildEnv = { ...process.env, CARGO_TARGET_DIR: isolatedTargetDir };
      if (isLinux) {
        // Match release CI: serialise Linux/musl builds to avoid an OpenSSL
        // provider archive race in vendored openssl-src.
        buildEnv.CARGO_BUILD_JOBS = "1";
      }
      if (isWindows) {
        const homeDir = process.env.HOME || process.env.USERPROFILE;
        const llvmMingwBin = path.join(homeDir, "llvm-mingw", "bin");
        buildEnv.PATH = `${llvmMingwBin}${path.delimiter}${process.env.PATH}`;
      }

      const buildResult = spawnSync("cargo", buildArgs, {
        stdio: "inherit",
        env: buildEnv,
        shell: true,
      });
      if (buildResult.status !== 0) {
        throw new Error(
          `cargo build failed for ${rustTarget} (exit ${buildResult.status}).`,
        );
      }

      const src = path.join(isolatedTargetDir, rustTarget, "release", binaryName);
      if (!existsSync(src)) {
        throw new Error(`Built binary not found at ${src}.`);
      }
      const dst = path.join(outDir, binaryName);
      copyFileSync(src, dst);
      console.log(`  -> ${dst}`);
    }
  }

  function stageCargoVersion(version) {
    if (!existsSync(cargoTomlPath)) {
      throw new Error(`Missing ${cargoTomlPath}.`);
    }

    const cargoLines = readFileSync(cargoTomlPath, "utf8").split("\n");
    let inPackage = false;
    let bumped = false;
    for (let i = 0; i < cargoLines.length; i += 1) {
      const trimmed = cargoLines[i].trim();
      if (trimmed === "[package]") {
        inPackage = true;
        continue;
      }
      if (inPackage && trimmed.startsWith("[")) {
        break;
      }
      if (inPackage && /^version\s*=/.test(trimmed)) {
        cargoLines[i] = `version = "${version}"`;
        bumped = true;
        break;
      }
    }

    if (!bumped) {
      throw new Error("Could not find [package].version in Cargo.toml.");
    }

    writeFileSync(cargoTomlPath, cargoLines.join("\n"), "utf8");
    console.log(`Staged Cargo.toml version ${version} for local release build.`);

    const lockResult = spawnSync("cargo", ["generate-lockfile"], {
      stdio: "inherit",
      shell: true,
    });
    if (lockResult.error) {
      throw new Error(
        `cargo generate-lockfile failed: ${lockResult.error.message}`,
      );
    }
    if (lockResult.status !== 0) {
      throw new Error(
        `cargo generate-lockfile failed (exit ${lockResult.status}).`,
      );
    }
  }

  process.on("SIGINT", sigintHandler);

  return {
    config,
    distDir,
    platformsDir,
    rootDir,
    validateConfig,
    runBuildPreflight,
    stageCargoVersion,
    buildDist,
    cleanup,
  };
}
