// Usage: analyzeHeadless <project_dir> <target_name> -postScript ExportCallGraph.java <workspace> <target>
// Exports detailed call graph with edge kinds: direct, indirect, tail, callgraph.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;
import ghidra.program.model.address.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;

public class ExportCallGraph extends GhidraScript {

    private String workspace;
    private String target;

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 2) {
            throw new IllegalArgumentException("Usage: ExportCallGraph <workspace> <target>");
        }
        workspace = getScriptArgs()[0];
        target = getScriptArgs()[1];

        Path outPath = Paths.get(workspace, "artifacts", target, "baseline", "callgraph.yaml");
        Files.createDirectories(outPath.getParent());

        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("edges:\n");

        ReferenceManager refMgr = currentProgram.getReferenceManager();
        FunctionManager funcMgr = currentProgram.getFunctionManager();

        for (Function caller : funcMgr.getFunctions(true)) {
            Address callerAddr = caller.getEntryPoint();
            String callerAddrStr = callerAddr.toString();

            ReferenceIterator iter = refMgr.getReferencesTo(callerAddr);
            while (iter.hasNext()) {
                Reference ref = iter.next();
                Address fromAddr = ref.getFromAddress();
                RefType refType = ref.getReferenceType();

                // Determine kind
                String kind = "direct";
                if (refType.isCall()) kind = "call";
                else if (refType.isJump()) kind = "jump";
                else if (refType.isConditional()) kind = "conditional";
                else if (refType.isIndirect()) kind = "indirect";
                else if (refType.isFallthrough()) kind = "fallthrough";

                yaml.append("  - from: \"").append(fromAddr.toString()).append("\"\n");
                yaml.append("    to: \"").append(callerAddrStr).append("\"\n");
                yaml.append("    kind: ").append(escapeYaml(kind)).append("\n");
            }
        }

        Files.writeString(outPath, yaml.toString());
        println("Exported callgraph.yaml");
    }

    private String escapeYaml(String s) {
        if (s == null) return "\"\"";
        if (s.contains(":") || s.contains("\"") || s.contains("'") || s.contains("\n") || s.startsWith(" ") || s.endsWith(" ") || s.contains("#") || s.contains("[") || s.contains("]") || s.contains("{") || s.contains("}") || s.contains(",") || s.equals("") || s.contains("&") || s.contains("*") || s.contains("!") || hasControlChars(s)) {
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
