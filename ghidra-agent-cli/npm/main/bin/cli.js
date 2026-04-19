#!/usr/bin/env node
import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const pkg = JSON.parse(
  readFileSync(path.join(here, "..", "package.json"), "utf8"),
);

const platformSuffix = `-${process.platform}-${process.arch}`;
const platformCandidates = Object.keys(pkg.optionalDependencies || {}).filter(
  (name) => name.endsWith(platformSuffix),
);
if (platformCandidates.length !== 1) {
  console.error(
    `Unsupported platform ${process.platform}-${process.arch}. ` +
      `Expected exactly one optionalDependency ending in ${platformSuffix}, found ${platformCandidates.length}.`,
  );
  process.exit(1);
}
const platformPackage = platformCandidates[0];
const binEntries = Object.keys(pkg.bin || {});
if (!binEntries.length) {
  console.error("package.json has no bin entry.");
  process.exit(1);
}
const cliName = binEntries[0];
const binaryName = `${cliName}${process.platform === "win32" ? ".exe" : ""}`;

const require = createRequire(import.meta.url);
let binaryPath;
try {
  binaryPath = require.resolve(`${platformPackage}/bin/${binaryName}`);
} catch {
  console.error(
    `Unsupported platform ${process.platform}-${process.arch}. ` +
      `Install ${platformPackage} directly or run on a supported platform.`,
  );
  process.exit(1);
}

// Keep Node.js alive on SIGINT so the child process (which receives the
// process-group signal) can clean up first. Without this, Node exits
// immediately and the child may print cleanup output over the shell prompt.
process.on("SIGINT", () => {});

const child = spawn(binaryPath, process.argv.slice(2), { stdio: "inherit" });
child.on("exit", (code, signal) => {
  if (signal) {
    // Remove the no-op SIGINT listener so the self-kill is not swallowed,
    // preserving the correct exit status (e.g. 130 for SIGINT).
    process.removeAllListeners(signal);
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});
child.on("error", (error) => {
  console.error(`Failed to start ${cliName}: ${error.message}`);
  process.exit(1);
});
