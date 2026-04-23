# REPLACE_WITH_PACKAGE_NAME

npm wrapper for the `REPLACE_WITH_CLI_NAME` CLI.

## Install

```bash
npm install -g REPLACE_WITH_PACKAGE_NAME
```

The matching native binary ships in a per-platform npm package that is selected
automatically via `optionalDependencies`. No postinstall download required.
Those platform packages are runtime-only payloads; only this main package
exposes the `REPLACE_WITH_CLI_NAME` command so the wrapper can configure the
bundled Ghidra scripts before launching the native binary.

## Supported platforms

- darwin-arm64, darwin-x64
- linux-arm64, linux-x64
- win32-arm64, win32-x64
