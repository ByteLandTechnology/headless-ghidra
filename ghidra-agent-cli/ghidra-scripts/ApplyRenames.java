// Usage: analyzeHeadless <project_dir> <target_name> -postScript ApplyRenames.java <workspace> <target>
// Applies function renames from functions.yaml to the Ghidra program.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;
import ghidra.program.model.address.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;

public class ApplyRenames extends GhidraScript {

    private String workspace;
    private String target;

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 2) {
            throw new IllegalArgumentException("Usage: ApplyRenames <workspace> <target>");
        }
        workspace = getScriptArgs()[0];
        target = getScriptArgs()[1];

        Path yamlPath = Paths.get(workspace, "artifacts", target, "baseline", "functions.yaml");
        if (!Files.exists(yamlPath)) {
            throw new IOException("functions.yaml not found: " + yamlPath);
        }

        List<Map<String, String>> functions = parseFunctionsYaml(yamlPath);
        int renamed = 0;
        int failed = 0;

        FunctionManager funcMgr = currentProgram.getFunctionManager();

        for (Map<String, String> funcEntry : functions) {
            String addrStr = funcEntry.get("addr");
            String name = funcEntry.get("name");

            if (addrStr == null || name == null) continue;

            Address addr = currentProgram.getAddressFactory().getAddress(addrStr);
            if (addr == null) {
                println("WARN: could not resolve address: " + addrStr);
                failed++;
                continue;
            }

            Function func = funcMgr.getFunctionAt(addr);
            if (func == null) {
                println("WARN: no function at address: " + addrStr);
                failed++;
                continue;
            }

            // Rename using symbol's setName method
            try {
                func.getSymbol().setName(name, SourceType.USER_DEFINED);
                renamed++;
                println("Renamed " + addrStr + " -> " + name);
            } catch (Exception e) {
                println("WARN: failed to rename " + addrStr + " to " + name + ": " + e.getMessage());
                failed++;
            }
        }

        println("ApplyRenames: " + renamed + " renamed, " + failed + " failed");
    }

    private List<Map<String, String>> parseFunctionsYaml(Path yamlPath) throws IOException {
        List<Map<String, String>> result = new ArrayList<>();
        List<String> lines = Files.readAllLines(yamlPath);
        Map<String, String> current = null;
        for (String line : lines) {
            if (line.startsWith("  - addr:")) {
                if (current != null) result.add(current);
                current = new HashMap<>();
                current.put("addr", extractYamlValue(line));
            } else if (line.startsWith("    name:") && current != null) {
                current.put("name", extractYamlValue(line));
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
