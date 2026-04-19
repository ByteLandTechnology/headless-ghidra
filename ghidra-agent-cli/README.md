# Release Automation Asset Pack

> Embedded-repo note: in this repository, the authoritative CI files live at
> repo root: `.github/workflows/release.yml` and
> `.github/actions/setup-build-env/action.yml`. Run release-pack commands from
> `ghidra-agent-cli/`.

Repository-owned release automation for target CLI skill repositories generated
with the `cli-forge` skill family.

## Release Model

Before the first production release, run one real **npm prepublish** step to
bootstrap package visibility and local auth. After that, one semantic-release
run publishes the production release surfaces:

1. git tag `v<version>` + GitHub Release page (archive + sha256 per target)
2. **N platform npm packages** (`<name>-darwin-arm64`, `<name>-linux-x64`, ...)
   each carrying only the matching native binary, gated by `os` / `cpu`
3. **1 main npm package** (`<name>`) — a tiny JS shim whose
   `optionalDependencies` pin every platform package to the same version
4. `CHANGELOG.md` entry
5. `chore(release): <version> [skip ci]` commit (`CHANGELOG.md`,
   `Cargo.toml`, `Cargo.lock`, `npm/main/package.json`)

Users install with plain `npm install -g <name>`; npm picks the right platform
package automatically. No postinstall download.

## How To Use It

1. Copy the contents of this directory into the root of the target CLI skill
   repository.
2. Fill `release/config.json`:
   - `cliName` — the shipped CLI binary name
   - `mainPackageName` — the npm main package name (for example `foo` or
     `@cli/foo`)
   - `mainNpmScope` — `null` for unscoped, or the scope string for
     `mainPackageName`
   - `platformNpmScope` — `null` for unscoped platform packages, or a different
     scope string when platform packages should publish under another scope
   - `sourceRepository` — `owner/repo` on GitHub
3. Platform package names are derived automatically from the main package body.
   The main package and platform packages may use different scopes, but only
   the scope may differ. Example:
   - main: `@cli/foo`
   - platforms: `@cli-platform/foo-darwin-arm64`, `@cli-platform/foo-linux-x64`
4. `npm/main/package.json` is derived at release time from
   `release/config.json`. You may pre-fill it for local testing, but
   `sync-platform-packages.mjs` will overwrite `name`, `version`, `bin`, and
   `optionalDependencies` from the authoritative config during `prepare`.
5. Configure npm trusted publishing on npmjs.com for this repository's
   `release.yml`:
   - configure a publisher entry for the main package and for **each** platform
     package
   - each entry points at the same workflow file (`release.yml`)
   - keep the job's `id-token: write` permission and do not inject `NPM_TOKEN`
6. Install the release harness locally once for a dry-run:

   ```bash
   npm ci
   npm run release:rehearse
   ```

   This builds every target, generates platform packages, and runs
   `npm publish --dry-run` for each, validating the full pipeline without
   pushing tags or publishing to npm.

   To see what version semantic-release _would_ choose without exercising the
   custom hooks:

   ```bash
   npm run release:dry-run
   ```

7. Before the first production CI release, perform the real prepublish step:

   ```bash
   npm run release:prepublish
   ```

   This publishes all platform packages and the main package at a dedicated
   prepublish version such as `0.0.0-prepublish.1`. If local npm auth is missing, the
   helper runs `npm login`; when npm prints a verification URL, open it in your
   browser and finish verification before the script continues. The prepublish helper
   temporarily disables npm provenance locally so this bootstrap publish does
   not masquerade as a CI-backed release.

8. Push to `main`; the workflow drives the live release.

## Targets

Defined in `release/config.json#targets`. Defaults:

| rustTarget                   | npm package suffix | os     | cpu   |
| ---------------------------- | ------------------ | ------ | ----- |
| `aarch64-apple-darwin`       | `darwin-arm64`     | darwin | arm64 |
| `x86_64-apple-darwin`        | `darwin-x64`       | darwin | x64   |
| `aarch64-unknown-linux-musl` | `linux-arm64`      | linux  | arm64 |
| `x86_64-unknown-linux-musl`  | `linux-x64`        | linux  | x64   |
| `aarch64-pc-windows-gnullvm` | `win32-arm64`      | win32  | arm64 |
| `x86_64-pc-windows-gnullvm`  | `win32-x64`        | win32  | x64   |

All six are built on a single `macos-14` runner using `cargo` + `cargo zigbuild`

- `llvm-mingw` (set up by `.github/actions/setup-build-env`). Linux targets use
  musl for fully static binaries that run on both glibc and musl (Alpine) systems.

## Clone-First Install (optional)

When users already have a checkout and want to install the binary directly from
the tagged GitHub Release archive, the harness attaches `tar.gz` + `sha256` per
target:

```bash
git clone https://github.com/<owner>/<repo>.git
cd <repo>
git checkout v<version>
./scripts/install-current-release.sh
```

The helper requires Node.js to read `release/config.json`. Prefer
`npm install -g <name>` for everything else.

## Files

The dot-prefixed repo files listed here are the final target-repository paths.
Inside the `cli-forge-publish` skill package they are stored under
installer-safe aliases (`dot-releaserc.json` and `dot-github/...`) and must be
restored to these dot-prefixed names when the asset pack is adopted.

- `.releaserc.json` — semantic-release plugin chain
- `.github/workflows/release.yml` — single release job
- `.github/actions/setup-build-env/action.yml` — macOS cross-build toolchain
- `release/config.json` — CLI name, main package name, split-scope policy, target list
- `npm/main/` — JS shim + published main package template
- `npm/platforms/` — generated at release time by
  `scripts/release/sync-platform-packages.mjs`
- `scripts/release/build-binaries.mjs` — semantic-release `prepare` hook that bumps
  `Cargo.toml#version`, builds all targets, and creates dist archives + provenance
- `scripts/release/validate-config.mjs` — shared config validation (fields, split-scope
  consistency, repository match, placeholder check); called by release.yml and
  sync-platform-packages.mjs
- `scripts/release/sync-platform-packages.mjs` — semantic-release `prepare` hook
- `scripts/release/publish-npm-packages.mjs` — semantic-release `publish` hook
  that publishes every platform package and the main package, each guarded by
  an `npm view` existence check for idempotent reruns
- `scripts/release/rehearse.mjs` — local rehearsal: build + sync +
  `npm publish --dry-run` for every package (no tag, no real publish)
- `scripts/release/prepublish.mjs` — local real publish for bootstrap
  versions only; publishes platform packages first, then the main package
- `scripts/release/ensure-npm-login.mjs` — helper that pauses for interactive
  `npm login`, relays the verification URL, and resumes prepublish only after
  local auth succeeds
- `scripts/install-current-release.sh` — clone-first install helper
- `package.json` — devDependencies (semantic-release + plugins) only
- `CHANGELOG.md` — maintained by semantic-release
