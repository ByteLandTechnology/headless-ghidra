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

        List<YamlParsers.FunctionEntry> functions = YamlParsers.loadFunctions(yamlPath);
        int matched = 0;
        int mismatched = 0;
        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("phase: P2\n");
        yaml.append("passed: true\n");
        yaml.append("checks:\n");

        FunctionManager funcMgr = currentProgram.getFunctionManager();

        for (YamlParsers.FunctionEntry funcEntry : functions) {
            String addrStr = AddressFormats.normalizeAddress(funcEntry.getAddrValue());
            String expectedName = funcEntry.getName();

            if (addrStr == null) continue;

            Address addr = AddressFormats.resolveAddress(currentProgram.getAddressFactory(), funcEntry.getAddrValue());
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
