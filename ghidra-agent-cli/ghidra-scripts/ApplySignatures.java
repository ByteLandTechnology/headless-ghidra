// Usage: analyzeHeadless <project_dir> <target_name> -postScript ApplySignatures.java <workspace> <target>
// Applies function signatures (prototypes) from types.yaml to the Ghidra program.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;

public class ApplySignatures extends GhidraScript {

    private String workspace;
    private String target;

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 2) {
            throw new IllegalArgumentException("Usage: ApplySignatures <workspace> <target>");
        }
        workspace = getScriptArgs()[0];
        target = getScriptArgs()[1];

        Path yamlPath = Paths.get(workspace, "artifacts", target, "baseline", "types.yaml");
        if (!Files.exists(yamlPath)) {
            throw new IOException("types.yaml not found: " + yamlPath);
        }

        List<Map<String, String>> types = parseTypesYaml(yamlPath);
        int applied = 0;
        int failed = 0;

        FunctionManager funcMgr = currentProgram.getFunctionManager();

        for (Map<String, String> entry : types) {
            String kind = entry.get("kind");
            if (!"function".equals(kind)) continue;

            String name = entry.get("name");
            String definition = entry.get("definition");

            if (name == null || definition == null) continue;

            // Find function by name
            Function func = null;
            for (Function f : funcMgr.getFunctions(true)) {
                if (name.equals(f.getName())) {
                    func = f;
                    break;
                }
            }
            if (func == null) {
                println("WARN: function not found: " + name);
                failed++;
                continue;
            }

            // Apply signature directly on the function's symbol
            try {
                func.getSymbol().setName(definition, SourceType.USER_DEFINED);
                applied++;
                println("Applied signature: " + name);
            } catch (Exception e) {
                println("WARN: failed to apply signature for " + name + ": " + e.getMessage());
                failed++;
            }
        }

        println("ApplySignatures: " + applied + " applied, " + failed + " failed");
    }

    private List<Map<String, String>> parseTypesYaml(Path yamlPath) throws IOException {
        List<Map<String, String>> result = new ArrayList<>();
        List<String> lines = Files.readAllLines(yamlPath);
        Map<String, String> current = null;
        for (String line : lines) {
            if (line.startsWith("  - name:")) {
                if (current != null) result.add(current);
                current = new HashMap<>();
                current.put("name", extractYamlValue(line));
            } else if (line.startsWith("    kind:") && current != null) {
                current.put("kind", extractYamlValue(line));
            } else if (line.startsWith("    definition:") && current != null) {
                current.put("definition", extractYamlValue(line));
            }
        }
        if (current != null) result.add(current);
        return result;
    }

    private String extractYamlValue(String line) {
        int colon = line.indexOf(':');
        if (colon < 0) return "";
        String val = line.substring(colon + 1).trim();
        // Handle double-quoted strings: "value"
        if (val.startsWith("\"") && val.endsWith("\"")) {
            val = val.substring(1, val.length() - 1);
        }
        // Handle single-quoted strings: 'value' (serde_yaml may output this)
        else if (val.startsWith("'") && val.endsWith("'")) {
            val = val.substring(1, val.length() - 1);
        }
        return val;
    }
}
