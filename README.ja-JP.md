# Headless Ghidra Skill Family

Headless Ghidra は、agent 向けの Ghidra reverse-engineering skill family です。skill family をインストールした後、agent に `headless-ghidra` を使うよう依頼します。同梱の `ghidra-agent-cli` は skill が必要に応じて呼び出す helper であり、通常の利用者が手動で install、build、run するものではありません。

言語: [English](./README.md) | [简体中文](./README.zh-CN.md)

## Install

Codex では `$skill-installer` の利用を推奨します。

```text
$skill-installer install all skills from https://github.com/ByteLandTechnology/headless-ghidra
```

これにより 7 個の sibling skill が入ります: `headless-ghidra`、P0-P4 の 5 phase skills、同梱 helper skill の `ghidra-agent-cli`。インストール後に Codex を restart してください。

`skills` CLI では、この skill family のすべての skill を対応するすべての agent に一度でインストールできます。

```sh
npx --yes skills add https://github.com/ByteLandTechnology/headless-ghidra --all
```

ここでの `--all` は `--skill '*' --agent '*' --yes` の shorthand です。

1 つの agent だけにすべての skill を入れる場合:

```sh
npx --yes skills add https://github.com/ByteLandTechnology/headless-ghidra --agent codex --skill '*' --yes
npx --yes skills add https://github.com/ByteLandTechnology/headless-ghidra --agent claude-code --skill '*' --yes
```

## Use

前提条件:

- local Ghidra がインストール済み。
- target binary が agent から読める workspace path にある。
- Frida は任意で、runtime observation が必要な場合だけ使います。

新しい analysis を始める:

```text
Use the headless-ghidra skill to analyze ./sample-target. Start at P0 intake,
choose a stable target id, and stop after each phase gate so I can review the
artifacts.
```

既存 target の続き:

```text
Resume the same target and continue through P1 baseline.
Show me the current pipeline state and the artifacts I should review.
Continue with P2 evidence, but do not classify uncertain third-party code
without showing me the evidence first.
Run the next P3/P4 iteration for the selected hotpath functions.
```

runtime output は active workspace の `targets/<target-id>/` と `artifacts/<target-id>/` に置き、installed skill directory には書き込みません。

## Documentation Map

| やりたいこと                                  | 読むもの                                                                                                                                                                           |
| --------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| skill family をインストールし、run を始める   | この README                                                                                                                                                                        |
| entry skill が作業をどう route するか理解する | [Orchestrator Skill Guide](./headless-ghidra/README.ja-JP.md)                                                                                                                      |
| 特定 phase を実行またはレビューする           | 下の P0-P4 phase README                                                                                                                                                            |
| walkthrough、playbook、script guidance を選ぶ | [Examples And Guides](./headless-ghidra/examples/README.ja-JP.md)                                                                                                                  |
| 何を先に分析するか選ぶ                        | [Analysis Selection Playbook](./headless-ghidra/examples/analysis-selection-playbook.ja-JP.md)                                                                                     |
| 完全な analysis narrative を見る              | [Reverse Engineering Walkthrough](./headless-ghidra/examples/reverse-engineering-walkthrough.ja-JP.md)                                                                             |
| custom Ghidra script を作成またはレビューする | [Ghidra Script Authoring](./headless-ghidra/examples/ghidra-script-authoring.md) と [Ghidra Script Review Checklist](./headless-ghidra/examples/ghidra-script-review-checklist.md) |
| agent command syntax や output を debug する  | [CLI Tool Reference](./ghidra-agent-cli/README.ja-JP.md)                                                                                                                           |

## Phases

| Phase | README                                                               | Purpose                                                          |
| ----- | -------------------------------------------------------------------- | ---------------------------------------------------------------- |
| P0    | [Intake](./headless-ghidra-intake/README.ja-JP.md)                   | target を確認し、workspace を初期化し、scope を設定する。        |
| P1    | [Baseline](./headless-ghidra-baseline/README.ja-JP.md)               | Ghidra import、baseline artifact export、runtime evidence 記録。 |
| P2    | [Evidence](./headless-ghidra-evidence/README.ja-JP.md)               | third-party code と evidence source を識別する。                 |
| P3    | [Discovery](./headless-ghidra-discovery/README.ja-JP.md)             | names、signatures、types、constants、strings を補強する。        |
| P4    | [Batch Decompile](./headless-ghidra-batch-decompile/README.ja-JP.md) | metadata を apply し、選択関数を decompile する。                |
