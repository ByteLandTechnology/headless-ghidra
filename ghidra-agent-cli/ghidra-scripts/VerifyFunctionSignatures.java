// Usage: analyzeHeadless <project_dir> <target_name> -postScript VerifyFunctionSignatures.java <workspace> <target>
// Verifies that function signatures in types.yaml match the current Ghidra program state.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;

public class VerifyFunctionSignatures extends GhidraScript {

    private String workspace;
    private String target;

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 2) {
            throw new IllegalArgumentException("Usage: VerifyFunctionSignatures <workspace> <target>");
        }
        workspace = getScriptArgs()[0];
        target = getScriptArgs()[1];

        Path yamlPath = Paths.get(workspace, "artifacts", target, "baseline", "types.yaml");
        Path outPath = Paths.get(workspace, "artifacts", target, "gates", "p5-verify-sigs.yaml");
        Files.createDirectories(outPath.getParent());

        List<Map<String, String>> types = parseTypesYaml(yamlPath);
        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("phase: P5\n");
        yaml.append("passed: true\n");
        yaml.append("checks:\n");

        int matched = 0;
        int mismatched = 0;

        FunctionManager funcMgr = currentProgram.getFunctionManager();

        for (Map<String, String> entry : types) {
            String kind = entry.get("kind");
            if (!"function".equals(kind)) continue;

            String name = entry.get("name");
            String expectedDef = entry.get("definition");

            if (name == null || expectedDef == null) continue;

            // Find function by name
            Function func = null;
            for (Function f : funcMgr.getFunctions(true)) {
                if (name.equals(f.getName())) {
                    func = f;
                    break;
                }
            }
            if (func == null) continue;

            String actualSig = func.getSignature().getPrototypeString();
            boolean ok = expectedDef.equals(actualSig);

            if (!ok) {
                mismatched++;
                yaml.append("  - id: sig_").append(name.replace("-", "_").replace(":", "_")).append("\n");
                yaml.append("    description: Function signature for ").append(name).append("\n");
                yaml.append("    passed: false\n");
                yaml.append("    detail: expected \"").append(expectedDef).append("\", got \"").append(actualSig).append("\"\n");
            } else {
                matched++;
            }
        }

        if (mismatched > 0) {
            yaml = new StringBuilder(yaml.toString().replace("passed: true", "passed: false"));
        }
        yaml.append("timestamp: ").append(java.time.Instant.now().toString()).append("\n");

        Files.writeString(outPath, yaml.toString());
        println("VerifyFunctionSignatures: " + matched + " matched, " + mismatched + " mismatched");
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

    private String escapeYaml(String s) {
        if (s == null) return "\"\"";
        if (s.contains(":") || s.contains("\"") || s.contains("\n") || s.startsWith(" ") || s.endsWith(" ") || s.contains("#") || s.equals("") || hasControlChars(s)) {
            return "\"" + escapeControlChars(s) + "\"";
        }
        return s;
    }

    private boolean hasControlChars(String s) {
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (c < 0x20 || c == 0x7F) {
                return true;
            }
        }
        return false;
    }

    private String escapeControlChars(String s) {
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (c < 0x20 || c == 0x7F) {
                sb.append(String.format("\\x%02X", (int)c));
            } else if (c == '\\') {
                sb.append("\\\\");
            } else if (c == '"') {
                sb.append("\\\"");
            } else {
                sb.append(c);
            }
        }
        return sb.toString();
    }
}
