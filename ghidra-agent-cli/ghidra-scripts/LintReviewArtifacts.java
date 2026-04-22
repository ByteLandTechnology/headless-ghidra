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
            List<YamlParsers.FunctionEntry> functions = YamlParsers.loadFunctions(fnYaml);
            for (int i = 0; i < functions.size(); i++) {
                YamlParsers.FunctionEntry f = functions.get(i);
                String addr = f.getAddr();
                String name = f.getName();

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
            List<YamlParsers.TypeEntry> types = YamlParsers.loadTypes(typesYaml);
            for (int i = 0; i < types.size(); i++) {
                YamlParsers.TypeEntry t = types.get(i);
                String name = t.getName();
                String kind = t.getKind();
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
