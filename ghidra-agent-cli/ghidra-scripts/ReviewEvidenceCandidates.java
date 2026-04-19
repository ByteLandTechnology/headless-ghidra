// Usage: analyzeHeadless <project_dir> <target_name> -postScript ReviewEvidenceCandidates.java <workspace> <target>
// Exports P2 evidence candidates for human review: external refs, library attachments, suspicious code patterns.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;
import ghidra.program.model.mem.*;
import ghidra.program.model.address.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;

public class ReviewEvidenceCandidates extends GhidraScript {

    private String workspace;
    private String target;

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 2) {
            throw new IllegalArgumentException("Usage: ReviewEvidenceCandidates <workspace> <target>");
        }
        workspace = getScriptArgs()[0];
        target = getScriptArgs()[1];

        Path outPath = Paths.get(workspace, "artifacts", target, "evidence-candidates.yaml");
        Files.createDirectories(outPath.getParent());

        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("candidates:\n");

        // 1. External references (library calls)
        ReferenceManager refMgr = currentProgram.getReferenceManager();
        ReferenceIterator externalRefs = refMgr.getExternalReferences();
        while (externalRefs.hasNext()) {
            Reference ref = externalRefs.next();
            if (ref instanceof ExternalReference) {
                ExternalReference extRef = (ExternalReference) ref;
                ExternalLocation extLoc = extRef.getExternalLocation();
                String library = extLoc.getLibraryName();
                String symbol = extLoc.getLabel();
                Address addr = extRef.getFromAddress();

                yaml.append("  - kind: external_ref\n");
                yaml.append("    library: ").append(escapeYaml(library != null ? library : "<unknown>")).append("\n");
                yaml.append("    symbol: ").append(escapeYaml(symbol != null ? symbol : "<unknown>")).append("\n");
                yaml.append("    addr: \"").append(addr.toString()).append("\"\n");
                yaml.append("    confidence: medium\n");
                yaml.append("    note: external library call reference\n");
            }
        }

        // 2. Functions with no name (likely auto-analyzed)
        FunctionManager funcMgr = currentProgram.getFunctionManager();
        for (Function func : funcMgr.getFunctions(true)) {
            String name = func.getName();
            if (name == null || name.startsWith("FUN_") || name.startsWith("thunk_")) {
                yaml.append("  - kind: unnamed_function\n");
                yaml.append("    addr: \"").append(func.getEntryPoint().toString()).append("\"\n");
                yaml.append("    current_name: ").append(escapeYaml(name != null ? name : "<none>")).append("\n");
                yaml.append("    confidence: low\n");
                yaml.append("    note: auto-generated name may need review\n");
            }
        }

        // 3. Memory blocks with execute access for potential code regions
        Memory memory = currentProgram.getMemory();
        for (MemoryBlock block : memory.getBlocks()) {
            if (block.isExecute()) {
                yaml.append("  - kind: executable_block\n");
                yaml.append("    name: ").append(escapeYaml(block.getName())).append("\n");
                yaml.append("    start: \"").append(block.getStart().toString()).append("\"\n");
                yaml.append("    end: \"").append(block.getEnd().toString()).append("\"\n");
                yaml.append("    size: ").append(block.getSize()).append("\n");
                yaml.append("    confidence: low\n");
                yaml.append("    note: executable memory block\n");
            }
        }

        Files.writeString(outPath, yaml.toString());
        println("Exported evidence-candidates.yaml");
    }

    private String escapeYaml(String s) {
        if (s == null) return "\"\"";
        if (s.contains(":") || s.contains("\"") || s.contains("'") || s.contains("\n") || s.startsWith(" ") || s.endsWith(" ") || s.contains("#") || s.contains("[") || s.contains("]") || s.equals("") || hasControlChars(s)) {
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
