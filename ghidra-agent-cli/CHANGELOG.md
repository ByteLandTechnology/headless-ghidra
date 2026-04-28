## [1.6.5](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.6.4...v1.6.5) (2026-04-28)

## [1.6.4](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.6.3...v1.6.4) (2026-04-27)

### Bug Fixes

* **ghidra:** use correct CParserUtils overload for header parsing ([8fdc517](https://github.com/ByteLandTechnology/headless-ghidra/commit/8fdc51702cb1ced752bb30455636d4814543f012))

## [1.6.3](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.6.2...v1.6.3) (2026-04-24)

### Bug Fixes

* **ghidra:** use function address in decompiled output headings ([7c62fd1](https://github.com/ByteLandTechnology/headless-ghidra/commit/7c62fd1d5242a6529865c3e50b2a59ebe2aaf0d1))

## [1.6.2](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.6.1...v1.6.2) (2026-04-24)

### Bug Fixes

- **ghidra-agent-cli:** raise SnakeYAML code point limit ([58713d4](https://github.com/ByteLandTechnology/headless-ghidra/commit/58713d46e4cb8f79f96bd696294fde1ca995515d))

## [1.6.1](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.6.0...v1.6.1) (2026-04-24)

### Bug Fixes

- **ghidra:** disable auto-analysis for metadata commands ([41b079e](https://github.com/ByteLandTechnology/headless-ghidra/commit/41b079e9267d751bd9dbcc21c93b49b756bd5ecb))

## [1.6.0](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.5.1...v1.6.0) (2026-04-24)

### Features

- **ghidra:** import custom types and signatures ([c58b971](https://github.com/ByteLandTechnology/headless-ghidra/commit/c58b971d149d8ac41999fdfc540e6fe04201164a))

## [1.5.1](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.5.0...v1.5.1) (2026-04-24)

### Bug Fixes

- **release:** keep platform packages runtime-only ([e39d866](https://github.com/ByteLandTechnology/headless-ghidra/commit/e39d8664cb674b3c9a8f0d0f565df50c4734a8d6))
- **release:** remove native git2 dependency ([3519e18](https://github.com/ByteLandTechnology/headless-ghidra/commit/3519e1851e95e42003bf1a77895a81d61970c673))

## [1.5.0](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.4.0...v1.5.0) (2026-04-23)

### Features

- **cli:** reorganize pipeline around p0-p4 artifacts ([c953248](https://github.com/ByteLandTechnology/headless-ghidra/commit/c953248543f48de140441f721639198af9febfba))

### Bug Fixes

- **release:** serialize linux openssl builds ([b26576e](https://github.com/ByteLandTechnology/headless-ghidra/commit/b26576eb622493fbece182e72eb9e93d2bd77d3a))

## [1.4.0](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.3.0...v1.4.0) (2026-04-23)

### Features

- **ghidra-agent-cli:** add analyze-vtables command ([73d4983](https://github.com/ByteLandTechnology/headless-ghidra/commit/73d4983d2e5505d37386148ae7005e517e500a91))

## [1.3.0](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.2.0...v1.3.0) (2026-04-22)

### Features

- **ghidra-agent-cli:** use SnakeYAML for script parsing ([d4f5c05](https://github.com/ByteLandTechnology/headless-ghidra/commit/d4f5c0552f0e0e2e97f54a70b618fa46b29a41f3))

## [1.2.0](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.1.2...v1.2.0) (2026-04-22)

### Features

- **ghidra-agent-cli:** add batch decompile support ([0a6e394](https://github.com/ByteLandTechnology/headless-ghidra/commit/0a6e39428151311c31768799a7251daa46deb659))

### Bug Fixes

- **release:** prebuild ghidra bundle for npm ([4e608db](https://github.com/ByteLandTechnology/headless-ghidra/commit/4e608dbc2e00c7718b012af1b53e707c4f493f08))

## [1.1.2](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.1.1...v1.1.2) (2026-04-21)

## [1.1.1](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.1.0...v1.1.1) (2026-04-21)

### Bug Fixes

- **cli:** bundle ghidra-scripts in npm package ([71d4587](https://github.com/ByteLandTechnology/headless-ghidra/commit/71d45876abb080aa2c7ce2c9129935fe8c94cb78))
- **scripts:** fix Ghidra 12.x API compat in ExportBaseline ([5d6ad0c](https://github.com/ByteLandTechnology/headless-ghidra/commit/5d6ad0cdd6ce4b618356e5e329eee47590382f40))

## [1.1.0](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.0.1...v1.1.0) (2026-04-21)

### Features

- **cli:** implement gate list, validate schema, frida device-attach, and strengthen gate checks ([3fcbe3c](https://github.com/ByteLandTechnology/headless-ghidra/commit/3fcbe3c6446dd128d92ac8fffb9cdd04c6fbc92e))

### Bug Fixes

- **cli:** add vtables export and fix % escaping in ExportBaseline.java ([e83ae24](https://github.com/ByteLandTechnology/headless-ghidra/commit/e83ae24e01a4e9810eba72f3206191167661c5f6))
- **cli:** resolve ghidra-scripts from binary path and document npm installation ([0d34323](https://github.com/ByteLandTechnology/headless-ghidra/commit/0d34323d1f6573b61c85a671e82286ab124f806d))

## [1.0.1](https://github.com/ByteLandTechnology/headless-ghidra/compare/v1.0.0...v1.0.1) (2026-04-20)

### Bug Fixes

- **release:** simplify GitHub asset upload logic ([076b916](https://github.com/ByteLandTechnology/headless-ghidra/commit/076b9160cfa2c745615600fd69a2b46366f02c2c))
- **skills:** add YAML frontmatter to ghidra-agent-cli SKILL.md ([9328f24](https://github.com/ByteLandTechnology/headless-ghidra/commit/9328f240f702a84a7a305e208dd207fa178142f9))

## 1.0.0 (2026-04-20)

### ⚠ BREAKING CHANGES

- **skills:** Replace 7 fragmented skills with 1 orchestrator + 6 phase sub-skills.

New architecture:

- headless-ghidra: global orchestrator (zero execution, dispatch only)
- headless-ghidra-intake: P0 target intake (2 parallel agents)
- headless-ghidra-baseline: P1 baseline extraction (new)
- headless-ghidra-evidence: P2 evidence review (4-dim parallel + library ID)
- headless-ghidra-discovery: P3 batch discovery (new)
- headless-ghidra-batch-decompile: P4+P5 batch decompilation (new)
- headless-ghidra-frida-verify: P6 Frida I/O verification (new)

New scripts:

- ghidra-agent-cli gate check: programmatic gate validation (P0–P6, replaces legacy gate-check.sh)
- ghidra-queue.sh: FIFO lock for Ghidra operation serialization
- reconstruction-init.sh: CMake reconstruction project scaffolding
- io-capture.js, io-compare.js, fuzz-input-gen.js: Frida verification

Removed skills:

- headless-ghidra-frida-runtime-injection (scripts migrated)
- headless-ghidra-frida-evidence
- headless-ghidra-progressive-decompilation
- headless-ghidra-script-review
- headless-ghidra-auto-evolution

### Features

- add ghidra-agent-cli subproject and consolidate docs ([00dfdfe](https://github.com/ByteLandTechnology/headless-ghidra/commit/00dfdfe746c9935bf42639d39eef79ebc4a2caf1))
- initial headless ghidra skill suite ([309a1ee](https://github.com/ByteLandTechnology/headless-ghidra/commit/309a1ee81931973f46b18b0ae59dadcfa91b891d))

### Bug Fixes

- enforce Ghidra-only decompilation ([dc9ba54](https://github.com/ByteLandTechnology/headless-ghidra/commit/dc9ba54df27ab6f0aaa23dab878b1d317d2e52c3))
- **gates:** complete all gate check implementations per SKILL.md spec ([507ef30](https://github.com/ByteLandTechnology/headless-ghidra/commit/507ef30a5b8f4444e3763711432166a36e75fe37))
- **headless-ghidra:** stabilize verification pipeline ([64bb243](https://github.com/ByteLandTechnology/headless-ghidra/commit/64bb243c78283b7d1586a1b8bd5ff2ce63c0d7bf))
- resolve P1 index bug, discover-ghidra double-call, and remove blank agent YAMLs ([03cc054](https://github.com/ByteLandTechnology/headless-ghidra/commit/03cc054c2b40ad5cf79e9cb6c1dd387e706c7b54))
- **runner:** add analysis timeout override for large binaries ([34ceee3](https://github.com/ByteLandTechnology/headless-ghidra/commit/34ceee3c88878ad7d6a93c92b2c6d119cbad1e25))
- **runner:** add stale lock detection and PID tracking to ghidra-queue ([ac63771](https://github.com/ByteLandTechnology/headless-ghidra/commit/ac637716fda5d434b53d6ce4f5159eaaaa1ac55a))
- **runner:** align artifact format references from .md to .yaml ([33b6bd6](https://github.com/ByteLandTechnology/headless-ghidra/commit/33b6bd6cc48d2b75af7036a1a8f20f728d74eca5))
- **scripts:** use nameref instead of eval for array operations ([e9d4e42](https://github.com/ByteLandTechnology/headless-ghidra/commit/e9d4e425e4eb1cbbc3e23502c226c81aaca2180f))
- **security:** clarify source comparison and runtime evidence guardrails ([30439c1](https://github.com/ByteLandTechnology/headless-ghidra/commit/30439c12c285593b8abab67e9241e89ed6c210ed))
- **templates:** replace Python template with Java to match pipeline constraints ([b4add67](https://github.com/ByteLandTechnology/headless-ghidra/commit/b4add67563ed695a6d59e49f0339ff5b54a6e85c))

### Code Refactoring

- **skills:** restructure into P0-P6 pipeline with orchestrator ([0aaee14](https://github.com/ByteLandTechnology/headless-ghidra/commit/0aaee143235e2744f65e4c2b796d77e716453c17))

# Changelog

All notable changes to this project are recorded here by `semantic-release`.

Version numbers, Git tags, GitHub Releases, npm publication, and changelog
entries are maintained by the release workflow. Do not hand-edit release
entries or manually bump package versions for production releases.
