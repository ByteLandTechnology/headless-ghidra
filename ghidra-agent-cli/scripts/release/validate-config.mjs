#!/usr/bin/env node
// Shared config validation for the release pipeline. Checks required fields,
// split-scope consistency, sourceRepository match against GITHUB_REPOSITORY
// (CI), and npm/main/package.json placeholder status. Exits 0 on pass, 1 on
// fail.
//
// Called from release.yml "Verify release config" step and from
// sync-platform-packages.mjs so both locations use identical logic.

import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildMainPackageName,
  buildPlatformPackageNames,
  parsePackageName,
  readRawReleaseConfig,
  resolveReleaseConfig,
} from "./release-config.mjs";

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);

const rawConfig = readRawReleaseConfig(rootDir);
const config = resolveReleaseConfig(rawConfig);

function fail(message) {
  console.error(message);
  process.exit(1);
}

function validateScopeString(fieldName, value) {
  if (value == null) return;
  if (/^@/.test(value) || value.includes("/") || /\s/.test(value)) {
    fail(
      `release/config.json#${fieldName} must be a bare scope string without @, slashes, or whitespace (found ${JSON.stringify(value)}).`,
    );
  }
  if (/REPLACE_WITH_/.test(value)) {
    fail(
      `release/config.json#${fieldName} must be set (found ${JSON.stringify(value)}).`,
    );
  }
}

// --- Required fields ---
for (const field of ["cliName", "mainPackageName", "sourceRepository"]) {
  const value =
    rawConfig[field] ?? (field === "mainPackageName" ? rawConfig.packageName : null);
  if (!value || /REPLACE_WITH_/.test(String(value))) {
    fail(
      `release/config.json#${field} must be set (found ${JSON.stringify(value)}).`,
    );
  }
}

const parsedMainPackage = parsePackageName(config.mainPackageName);
if (!parsedMainPackage.valid || !parsedMainPackage.body) {
  fail(
    `release/config.json#mainPackageName (${JSON.stringify(config.mainPackageName)}) is not a valid npm package name.`,
  );
}

const derivedPlatformPackages = buildPlatformPackageNames(config);
for (const pkgName of derivedPlatformPackages) {
  const parsedPackage = parsePackageName(pkgName);
  if (!parsedPackage.valid || !parsedPackage.body) {
    fail(
      `Derived platform package name ${JSON.stringify(pkgName)} is not a valid npm package name. Shorten release/config.json#mainPackageName or adjust target packageSuffix values.`,
    );
  }
}

// --- Scope consistency ---
validateScopeString("mainNpmScope", config.mainNpmScope);
validateScopeString("platformNpmScope", config.platformNpmScope);

if (parsedMainPackage.scoped) {
  if (config.mainNpmScope == null) {
    fail(
      `release/config.json#mainPackageName is scoped (${config.mainPackageName}) but mainNpmScope is null.`,
    );
  }
  if (config.mainNpmScope !== parsedMainPackage.scope) {
    fail(
      `release/config.json#mainNpmScope (${config.mainNpmScope}) does not match mainPackageName scope (${parsedMainPackage.scope}).`,
    );
  }
} else if (config.mainNpmScope != null) {
  fail(
    `release/config.json#mainNpmScope is ${JSON.stringify(config.mainNpmScope)} but mainPackageName is unscoped (${config.mainPackageName}).`,
  );
}

// --- sourceRepository matches GITHUB_REPOSITORY (CI only) ---
const ghRepo = process.env.GITHUB_REPOSITORY ?? "";
if (ghRepo && config.sourceRepository !== ghRepo) {
  fail(
    `release/config.json#sourceRepository (${config.sourceRepository}) does not match GITHUB_REPOSITORY (${ghRepo}).`,
  );
}

// --- Main package.json placeholders ---
const mainPkgPath = path.join(rootDir, "npm/main/package.json");
if (existsSync(mainPkgPath)) {
  const pkg = JSON.parse(readFileSync(mainPkgPath, "utf8"));
  for (const field of ["name", "description", "bin"]) {
    const val =
      typeof pkg[field] === "object"
        ? JSON.stringify(pkg[field])
        : String(pkg[field]);
    if (/REPLACE_WITH_/.test(val)) {
      fail(`npm/main/package.json#${field} still has placeholders.`);
    }
  }
  console.log(`Main package OK: ${pkg.name}`);
}

console.log(
  `Config OK: main=${buildMainPackageName(config)} platformScope=${JSON.stringify(config.platformNpmScope)} repo=${config.sourceRepository}`,
);
console.log(
  `Derived platform packages: ${derivedPlatformPackages.join(", ")}`,
);
