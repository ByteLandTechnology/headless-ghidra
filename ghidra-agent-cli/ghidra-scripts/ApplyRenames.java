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

        List<YamlParsers.FunctionEntry> functions = YamlParsers.loadFunctions(yamlPath);
        int renamed = 0;
        int failed = 0;

        FunctionManager funcMgr = currentProgram.getFunctionManager();

        for (YamlParsers.FunctionEntry funcEntry : functions) {
            String addrStr = AddressFormats.normalizeAddress(funcEntry.getAddrValue());
            String name = funcEntry.getName();

            if (addrStr == null || name == null) continue;

            Address addr = AddressFormats.resolveAddress(currentProgram.getAddressFactory(), funcEntry.getAddrValue());
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
}
