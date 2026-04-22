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

        List<YamlParsers.TypeEntry> types = YamlParsers.loadTypes(yamlPath);
        int applied = 0;
        int failed = 0;

        FunctionManager funcMgr = currentProgram.getFunctionManager();

        for (YamlParsers.TypeEntry entry : types) {
            String kind = entry.getKind();
            if (!"function".equals(kind)) continue;

            String name = entry.getName();
            String definition = entry.getDefinition();

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
}
