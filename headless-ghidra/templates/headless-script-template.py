# Reusable template for headless Ghidra scripts in this repository.
#
# Required metadata for every derived script:
#   - Script role: discovery | analysis | export | reconstruction-support
#   - Target outputs: tracked artifact files or generated project state
#   - Allowed side effects: read-only | export-only | analysis-metadata-update
#   - Registration: command-manifest entry + runner wiring + validation notes
#
# Script args:
#   1. output_dir
#   2. target_id (optional)

from __future__ import print_function

import os


def require_args(min_count):
    args = getScriptArgs()
    if len(args) < min_count:
        printerr("Expected at least %d script args." % min_count)
        raise SystemExit(1)
    return args


def ensure_dir(path):
    if not os.path.isdir(path):
        os.makedirs(path)


def write_markdown(path, title, body):
    handle = open(path, "w")
    try:
        handle.write("# %s\n\n%s" % (title, body))
    finally:
        handle.close()


def collect_program_summary():
    function_count = currentProgram.getFunctionManager().getFunctionCount()
    image_base = str(currentProgram.getImageBase())
    return {
        "program": currentProgram.getName(),
        "image_base": image_base,
        "function_count": str(function_count),
    }


def main():
    args = require_args(1)
    output_dir = args[0]
    target_id = args[1] if len(args) > 1 else currentProgram.getName()
    ensure_dir(output_dir)

    summary = collect_program_summary()
    body = (
        "- Target ID: `%s`\n"
        "- Program: `%s`\n"
        "- Image base: `%s`\n"
        "- Function count: `%s`\n\n"
        "## TODO\n\n"
        "- Replace this section with task-specific observations.\n"
        "- Keep writes deterministic and limited to approved output paths.\n"
        "- Record any metadata mutations in `renaming-log.md` or an equivalent tracked artifact.\n"
    ) % (
        target_id,
        summary["program"],
        summary["image_base"],
        summary["function_count"],
    )
    write_markdown(os.path.join(output_dir, "template-output.md"), "Template Output", body)


main()
