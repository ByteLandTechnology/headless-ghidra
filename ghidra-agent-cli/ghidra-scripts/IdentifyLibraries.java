// Usage: analyzeHeadless <project_dir> <target_name> -postScript IdentifyLibraries.java <workspace> <target>
// Detects third-party libraries by version strings and known patterns.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;
import ghidra.program.model.data.*;
import ghidra.program.util.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.regex.*;

public class IdentifyLibraries extends GhidraScript {

    private String workspace;
    private String target;

    // Known library patterns: name -> list of version regexes
    private static final Map<String, String[]> LIB_PATTERNS = new LinkedHashMap<>();
    static {
        LIB_PATTERNS.put("glibc", new String[]{"2\\.\\d+", "GLIBC_2\\.\\d+"});
        LIB_PATTERNS.put("libstdc++", new String[]{"GLIBCXX_3\\.\\d+", "LIBCXX_\\d+"});
        LIB_PATTERNS.put("openssl", new String[]{"OpenSSL_1_\\d_\\d+", "OpenSSL_3_\\d_\\d+"});
        LIB_PATTERNS.put("libcurl", new String[]{"curl-7\\.\\d+\\.\\d+"});
        LIB_PATTERNS.put("zlib", new String[]{"zlib 1\\.\\d+\\.\\d+"});
        LIB_PATTERNS.put("libpng", new String[]{"libpng version 1\\.\\d+\\.\\d+"});
        LIB_PATTERNS.put("libjpeg", new String[]{"JPEG library \\d+\\.\\d+"});
        LIB_PATTERNS.put("libuv", new String[]{"libuv \\d+\\.\\d+"});
        LIB_PATTERNS.put("libelf", new String[]{"ELFUTILS_\\d+"});
        LIB_PATTERNS.put("libcrypto", new String[]{"OpenSSL_1_\\d_\\d+", "OpenSSL_3_\\d_\\d+"});
    }

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 2) {
            throw new IllegalArgumentException("Usage: IdentifyLibraries <workspace> <target>");
        }
        workspace = getScriptArgs()[0];
        target = getScriptArgs()[1];

        Path outDir = Paths.get(workspace, "artifacts", target, "third-party");
        Files.createDirectories(outDir);
        Path outPath = outDir.resolve("identified.yaml");

        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("libraries:\n");

        Set<String> foundLibs = new LinkedHashSet<>();

        // 1. Scan strings for version patterns
        for (Data data : DefinedStringIterator.forProgram(currentProgram, null)) {
            StringDataInstance sdi = StringDataInstance.getStringDataInstance(data);
            String content = sdi.getStringValue();
            if (content == null) continue;
            for (Map.Entry<String, String[]> entry : LIB_PATTERNS.entrySet()) {
                String libName = entry.getKey();
                for (String pattern : entry.getValue()) {
                    try {
                        if (Pattern.compile(pattern).matcher(content).find()) {
                            foundLibs.add(libName + ":" + extractVersion(content, pattern));
                        }
                    } catch (Exception e) {
                        // Skip bad regex
                    }
                }
            }
        }

        // 2. Check external symbols for library names
        ExternalManager extMgr = currentProgram.getExternalManager();
        String[] libNames = extMgr.getExternalLibraryNames();
        for (String libName : libNames) {
            for (String patternKey : LIB_PATTERNS.keySet()) {
                if (libName.toLowerCase().contains(patternKey.toLowerCase())) {
                    foundLibs.add(patternKey + ":<unknown>");
                }
            }
        }

        // Emit findings
        for (String lib : foundLibs) {
            String[] parts = lib.split(":", 2);
            String libName = parts[0];
            String version = parts.length > 1 ? parts[1] : "<unknown>";
            String confidence = version.equals("<unknown>") ? "low" : "high";

            yaml.append("  - name: ").append(escapeYaml(libName)).append("\n");
            yaml.append("    version: ").append(escapeYaml(version)).append("\n");
            yaml.append("    confidence: ").append(escapeYaml(confidence)).append("\n");
            yaml.append("    evidence: string_scan\n");
            yaml.append("    upstream_url: \n");
            yaml.append("    function_classifications: []\n");
        }

        Files.writeString(outPath, yaml.toString());
        println("Exported third-party/identified.yaml");
    }

    private String extractVersion(String content, String pattern) {
        try {
            Matcher m = Pattern.compile(pattern).matcher(content);
            if (m.find()) return m.group();
        } catch (Exception e) {}
        return "<unknown>";
    }

    private String escapeYaml(String s) {
        if (s == null) return "\"\"";
        if (s.contains(":") || s.contains("\"") || s.contains("'") || s.contains("\n") || s.startsWith(" ") || s.endsWith(" ") || s.contains("#") || s.equals("") || hasControlChars(s)) {
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
