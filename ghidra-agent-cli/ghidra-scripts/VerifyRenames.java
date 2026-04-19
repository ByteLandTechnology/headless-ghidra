// Usage: analyzeHeadless <project_dir> <target_name> -postScript VerifyRenames.java <workspace> <target>
// Verifies that function names in functions.yaml match the current Ghidra program state.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.address.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;

public class VerifyRenames extends GhidraScript {

    private String workspace;
    private String target;

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 2) {
            throw new IllegalArgumentException("Usage: VerifyRenames <workspace> <target>");
        }
        workspace = getScriptArgs()[0];
        target = getScriptArgs()[1];

        Path yamlPath = Paths.get(workspace, "artifacts", target, "baseline", "functions.yaml");
        Path outPath = Paths.get(workspace, "artifacts", target, "gates", "p2-verify-renames.yaml");
        Files.createDirectories(outPath.getParent());

        List<Map<String, String>> functions = parseFunctionsYaml(yamlPath);
        int matched = 0;
        int mismatched = 0;
        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("phase: P2\n");
        yaml.append("passed: true\n");
        yaml.append("checks:\n");

        FunctionManager funcMgr = currentProgram.getFunctionManager();

        for (Map<String, String> funcEntry : functions) {
            String addrStr = funcEntry.get("addr");
            String expectedName = funcEntry.get("name");

            if (addrStr == null) continue;

            Address addr = currentProgram.getAddressFactory().getAddress(addrStr);
            if (addr == null) continue;

            Function func = funcMgr.getFunctionAt(addr);
            if (func == null) continue;

            String actualName = func.getName();
            boolean ok = expectedName == null || expectedName.equals(actualName);

            if (ok) {
                matched++;
            } else {
                mismatched++;
                yaml.append("  - id: verify_").append(addrStr.replace(":", "_")).append("\n");
                yaml.append("    description: Function rename at ").append(addrStr).append("\n");
                yaml.append("    passed: false\n");
                yaml.append("    detail: expected \"").append(expectedName).append("\", got \"").append(actualName).append("\"\n");
            }
        }

        if (mismatched > 0) {
            yaml = new StringBuilder(yaml.toString().replace("passed: true", "passed: false"));
        }
        yaml.append("timestamp: ").append(java.time.Instant.now().toString()).append("\n");

        Files.writeString(outPath, yaml.toString());
        println("VerifyRenames: " + matched + " matched, " + mismatched + " mismatched");
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
