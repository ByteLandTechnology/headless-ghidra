// fuzz-input-gen.js - Generate fuzz inputs from function signatures
// Usage: frida -f <binary> -l fuzz-input-gen.js -- <args> > fuzz_inputs.txt
// Reads function prototypes from BASELINE_TYPES_YAML env var path, generates inputs

{
  const yamlPath = env.BASELINE_TYPES_YAML;
  if (!yamlPath) {
    console.error(
      "Usage: BASELINE_TYPES_YAML=<path> frida -f <bin> -l fuzz-input-gen.js",
    );
    process.exit(1);
  }

  const yaml = JSON.parse(new TextDecoder().decode(readFileSync(yamlPath)));

  const inputs = [];
  const seenSigs = new Set();

  for (const entry of yaml.types || []) {
    if (entry.kind !== "function") continue;
    const sig = entry.definition;
    if (seenSigs.has(sig)) continue;
    seenSigs.add(sig);

    // Parse prototype and generate inputs
    const inputs_for_fn = generateInputs(sig);
    for (const inp of inputs_for_fn) {
      inputs.push({ prototype: sig, input: inp });
    }
  }

  for (const inp of inputs) {
    console.log(JSON.stringify(inp));
  }

  function generateInputs(prototype) {
    // Very simple generator: infer from type names
    const results = [];
    // null input
    results.push("null");
    // zeroed memory
    results.push("zeroed");
    // walk prototype string for type hints
    if (prototype.includes("char") || prototype.includes("str")) {
      results.push('"A"');
      results.push('""');
      results.push('"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"');
    }
    if (prototype.includes("int")) {
      results.push("0");
      results.push("1");
      results.push("-1");
      results.push("0x7fffffff");
    }
    if (prototype.includes("size_t")) {
      results.push("0");
      results.push("4096");
      results.push("0xffffffff");
    }
    if (prototype.includes("void*") || prototype.includes("ptr")) {
      results.push("NULL");
      results.push("0x1");
    }
    return results.length > 0 ? results : ["NULL"];
  }
}
