# ghidra-agent-cli

Bundled npm wrapper for the `ghidra-agent-cli` helper used by the Headless
Ghidra skill family.

## Normal Use

Normal skill users do not install or run this package directly. Install the
Headless Ghidra skill family, then ask your agent to use the `headless-ghidra`
skill. The skill invokes this wrapper when it needs the helper CLI.

The package selects the matching native binary for your platform through npm
optional dependencies. No postinstall download is required.

## Helper Reference

```sh
ghidra-agent-cli --help
ghidra-agent-cli ghidra discover
ghidra-agent-cli workspace init --target sample-target --binary ./sample-target
```

The wrapper also ships the bundled Ghidra scripts needed by the CLI before
launching the native binary.

## Supported Platforms

- darwin-arm64, darwin-x64
- linux-arm64, linux-x64
- win32-arm64, win32-x64
