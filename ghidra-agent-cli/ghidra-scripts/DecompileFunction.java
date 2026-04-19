// Usage: analyzeHeadless <project_dir> <target_name> -postScript DecompileFunction.java <workspace> <target> <fn_addr> <fn_id>
// Decompiles a single function and writes C output to decompilation/functions/<fn_id>.c

import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;
import ghidra.program.model.address.*;
import java.io.*;
import java.nio.file.*;

public class DecompileFunction extends GhidraScript {

    private String workspace;
    private String target;
    private String fnAddr;
    private String fnId;

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 4) {
            throw new IllegalArgumentException("Usage: DecompileFunction <workspace> <target> <fn_addr> <fn_id>");
        }
        workspace = getScriptArgs()[0];
        target = getScriptArgs()[1];
        fnAddr = getScriptArgs()[2];
        fnId = getScriptArgs()[3];

        Address addr = currentProgram.getAddressFactory().getAddress(fnAddr);
        if (addr == null) {
            throw new IOException("Invalid address: " + fnAddr);
        }

        Function func = currentProgram.getFunctionManager().getFunctionAt(addr);
        if (func == null) {
            throw new IOException("No function at address: " + fnAddr);
        }

        // Decompile
        DecompileOptions options = new DecompileOptions();
        DecompInterface decomp = new DecompInterface();
        decomp.setOptions(options);
        decomp.openProgram(currentProgram);
        DecompileResults results = decomp.decompileFunction(func, 30, monitor);

        Path outDir = Paths.get(workspace, "artifacts", target, "decompilation", "functions", fnId);
        Files.createDirectories(outDir);
        Path cPath = outDir.resolve(fnId + ".c");
        Path metaPath = outDir.resolve("decompilation-record.yaml");

        if (results.decompileCompleted()) {
            String cCode = results.getDecompiledFunction().getC();
            Files.writeString(cPath, cCode);

            // Write metadata
            String meta = "fn_id: " + fnId + "\n" +
                          "addr: " + fnAddr + "\n" +
                          "name: " + func.getName() + "\n" +
                          "prototype: " + func.getSignature().getPrototypeString() + "\n" +
                          "size: " + func.getBody().getNumAddresses() + "\n" +
                          "timestamp: " + java.time.Instant.now().toString() + "\n";
            Files.writeString(metaPath, meta);

            println("Decompiled: " + fnId + " -> " + cPath);
        } else {
            Files.writeString(cPath, "/* decompilation failed */\n");
            println("WARN: decompilation failed for " + fnId);
        }

        decomp.dispose();
    }
}
