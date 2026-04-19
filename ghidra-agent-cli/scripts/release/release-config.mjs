#!/usr/bin/env node
// Shared helpers for release/config.json so local scripts, CI helpers, and the
// published main-package shim all derive package names the same way.

import { readFileSync } from "node:fs";
import path from "node:path";

export const MAX_NPM_PACKAGE_NAME_LENGTH = 214;

export function normalizeScope(value) {
  if (value == null || value === "") return null;
  return String(value);
}

function isValidPackageNameLength(value) {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    value.length <= MAX_NPM_PACKAGE_NAME_LENGTH
  );
}

function isValidPackageSegment(value) {
  return (
    typeof value === "string" &&
    /^(?![._])[a-z0-9._-]+$/.test(value) &&
    value !== "node_modules" &&
    value !== "favicon.ico"
  );
}

export function parsePackageName(name) {
  if (!name) {
    return { fullName: name, scope: null, body: null, scoped: false };
  }

  const fullName = String(name);
  const validLength = isValidPackageNameLength(fullName);
  if (fullName.startsWith("@")) {
    const match = /^@([^/]+)\/(.+)$/.exec(fullName);
    if (!match) {
      return {
        fullName,
        scope: null,
        body: null,
        scoped: true,
        valid: false,
        validLength,
      };
    }
    const scope = match[1];
    const body = match[2];
    return {
      fullName,
      scope,
      body,
      scoped: true,
      valid:
        validLength &&
        isValidPackageSegment(scope) &&
        isValidPackageSegment(body),
      validLength,
    };
  }

  if (fullName.includes("/")) {
    return {
      fullName,
      scope: null,
      body: null,
      scoped: false,
      valid: false,
      validLength,
    };
  }

  return {
    fullName,
    scope: null,
    body: fullName,
    scoped: false,
    valid: validLength && isValidPackageSegment(fullName),
    validLength,
  };
}

export function formatPackageName(scope, body) {
  return scope ? `@${scope}/${body}` : body;
}

export function readRawReleaseConfig(rootDir) {
  return JSON.parse(
    readFileSync(path.join(rootDir, "release/config.json"), "utf8"),
  );
}

export function resolveReleaseConfig(rawConfig) {
  const mainPackageName = rawConfig.mainPackageName ?? rawConfig.packageName;
  const parsedMain = parsePackageName(mainPackageName);
  const mainNpmScope = normalizeScope(
    rawConfig.mainNpmScope ?? rawConfig.npmScope ?? parsedMain.scope,
  );
  const platformNpmScope = normalizeScope(
    rawConfig.platformNpmScope ?? mainNpmScope,
  );
  const packageBaseName = parsedMain.body;

  return {
    ...rawConfig,
    mainPackageName,
    mainNpmScope,
    platformNpmScope,
    packageBaseName,
  };
}

export function readReleaseConfig(rootDir) {
  return resolveReleaseConfig(readRawReleaseConfig(rootDir));
}

export function buildMainPackageName(config) {
  return formatPackageName(config.mainNpmScope, config.packageBaseName);
}

export function buildPlatformPackageName(config, target) {
  return formatPackageName(
    config.platformNpmScope,
    `${config.packageBaseName}-${target.packageSuffix}`,
  );
}

export function buildPlatformPackageNames(config) {
  return config.targets.map((target) => buildPlatformPackageName(config, target));
}

export function buildAllPackageNames(config) {
  return [buildMainPackageName(config), ...buildPlatformPackageNames(config)];
}
