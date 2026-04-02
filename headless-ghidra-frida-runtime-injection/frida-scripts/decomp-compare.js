/*
script_id: decomp-compare
scenario: decompilation-to-original comparison
invocation_shape: frida -f <target> -l decomp-compare.js --runtime=v8 --no-pause
expected_outputs: call-order notes, branch markers, return summaries
coverage_notes: Supports bounded compare targets and configured branch markers.
*/

"use strict";

const CONFIG = globalThis.__GHIDRA_FRIDA_CONFIG__ || {
  targets: [],
};

function emit(kind, payload) {
  send({ script_id: "decomp-compare", kind, payload });
}

for (const targetConfig of CONFIG.targets) {
  const target = targetConfig.address
    ? ptr(targetConfig.address)
    : Module.getExportByName(targetConfig.module || null, targetConfig.name);
  Interceptor.attach(target, {
    onEnter() {
      emit("enter", {
        name: targetConfig.name || targetConfig.address,
        compare_notes: targetConfig.compareNotes || [],
      });
    },
    onLeave(retval) {
      emit("leave", {
        name: targetConfig.name || targetConfig.address,
        retval: retval.toString(),
      });
    },
  });
}
