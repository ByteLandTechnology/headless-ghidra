// Usage: analyzeHeadless <project_dir> <target_name> -postScript LintReviewArtifacts.java <workspace> <target>
// Lints rename and signature YAML files, checking for common issues.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.regex.*;

public class LintReviewArtifacts extends GhidraScript {

    private String workspace;
    private String target;

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 2) {
            throw new IllegalArgumentException("Usage: LintReviewArtifacts <workspace> <target>");
        }
        workspace = getScriptArgs()[0];
        target = getScriptArgs()[1];

        Path outPath = Paths.get(workspace, "artifacts", target, "gates", "lint-report.yaml");
        Files.createDirectories(outPath.getParent());

        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("phase: P3\n");
        yaml.append("passed: true\n");
        yaml.append("issues:\n");

        boolean hasIssues = false;

        // Lint functions.yaml
        Path fnYaml = Paths.get(workspace, "artifacts", target, "baseline", "functions.yaml");
        if (Files.exists(fnYaml)) {
            List<Map<String, String>> functions = parseFunctionsYaml(fnYaml);
            for (int i = 0; i < functions.size(); i++) {
                Map<String, String> f = functions.get(i);
                String addr = f.get("addr");
                String name = f.get("name");

                if (addr == null || addr.isEmpty()) {
                    yaml.append("  - severity: error\n");
                    yaml.append("    file: functions.yaml\n");
                    yaml.append("    entry: ").append(i).append("\n");
                    yaml.append("    message: missing address\n");
                    hasIssues = true;
                }
                if (name != null && !name.isEmpty() && !isValidIdent(name)) {
                    yaml.append("  - severity: warning\n");
                    yaml.append("    file: functions.yaml\n");
                    yaml.append("    entry: ").append(i).append("\n");
                    yaml.append("    message: name \"").append(name).append("\" may not be a valid identifier\n");
                    hasIssues = true;
                }
            }
        }

        // Lint types.yaml
        Path typesYaml = Paths.get(workspace, "artifacts", target, "baseline", "types.yaml");
        if (Files.exists(typesYaml)) {
            List<Map<String, String>> types = parseTypesYaml(typesYaml);
            for (int i = 0; i < types.size(); i++) {
                Map<String, String> t = types.get(i);
                String name = t.get("name");
                String kind = t.get("kind");
                if (name == null || name.isEmpty()) {
                    yaml.append("  - severity: error\n");
                    yaml.append("    file: types.yaml\n");
                    yaml.append("    entry: ").append(i).append("\n");
                    yaml.append("    message: missing type name\n");
                    hasIssues = true;
                }
                if (kind != null && !kind.matches("^(struct|enum|union|function|typedef|pointer|array|builtin)$")) {
                    yaml.append("  - severity: warning\n");
                    yaml.append("    file: types.yaml\n");
                    yaml.append("    entry: ").append(i).append("\n");
                    yaml.append("    message: unknown kind \"").append(kind).append("\"\n");
                    hasIssues = true;
                }
            }
        }

        if (hasIssues) {
            yaml = new StringBuilder(yaml.toString().replace("passed: true", "passed: false"));
        }
        yaml.append("timestamp: ").append(java.time.Instant.now().toString()).append("\n");

        Files.writeString(outPath, yaml.toString());
        println("LintReviewArtifacts: " + (hasIssues ? "issues found" : "clean"));
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

    private boolean isValidIdent(String name) {
        return name.matches("^[a-zA-Z_][a-zA-Z0-9_]*$");
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
