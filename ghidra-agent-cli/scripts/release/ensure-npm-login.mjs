#!/usr/bin/env node
// Interactive npm auth helper for local prepublish publication. This is intentionally
// not used by CI, which should rely on trusted publishing instead.

import { spawn, spawnSync } from "node:child_process";

const registry = "https://registry.npmjs.org";

function npmWhoAmI() {
  const result = spawnSync("npm", ["whoami", "--registry", registry], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    shell: true,
  });
  if (result.status === 0) {
    return result.stdout.trim();
  }
  return null;
}

function extractUrls(text) {
  return [...text.matchAll(/https:\/\/[^\s)]+/g)].map((match) => match[0]);
}

const currentUser = npmWhoAmI();
if (currentUser) {
  console.log(`npm auth OK: ${currentUser}`);
  process.exit(0);
}

console.log("npm login is required before local prepublish.");
console.log("Starting: npm login --registry=https://registry.npmjs.org");

const seenUrls = new Set();
let loginOutput = "";

const child = spawn("npm", ["login", "--registry", registry], {
  stdio: ["inherit", "pipe", "pipe"],
  shell: true,
});

function relay(stream, target) {
  stream.on("data", (chunk) => {
    const text = chunk.toString();
    loginOutput += text;
    target.write(text);

    for (const url of extractUrls(text)) {
      if (seenUrls.has(url)) continue;
      seenUrls.add(url);
      process.stdout.write(
        `\nOpen this verification URL in your browser and finish npm login:\n${url}\n\n`,
      );
    }
  });
}

relay(child.stdout, process.stdout);
relay(child.stderr, process.stderr);

child.on("close", (code) => {
  if (code !== 0) {
    console.error(
      "npm login did not complete successfully. Finish browser verification and retry prepublish.",
    );
    process.exit(code ?? 1);
  }

  const loggedInUser = npmWhoAmI();
  if (!loggedInUser) {
    console.error(
      "npm login exited without an authenticated session. Confirm the browser verification completed, then rerun the prepublish flow.",
    );
    if (!seenUrls.size && loginOutput.trim()) {
      console.error("npm login output:");
      console.error(loginOutput.trim());
    }
    process.exit(1);
  }

  console.log(`npm auth OK: ${loggedInUser}`);
});
