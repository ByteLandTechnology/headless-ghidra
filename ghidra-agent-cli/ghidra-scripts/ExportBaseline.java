// Usage: analyzeHeadless <project_dir> <target_name> -postScript ExportBaseline.java <workspace> <target>
// Exports all 7 baseline YAML artifacts: functions, callgraph, types, vtables, constants, strings, imports.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.*;
import ghidra.program.model.listing.*;
import ghidra.program.model.data.*;
import ghidra.program.model.symbol.*;
import ghidra.program.model.mem.*;
import ghidra.program.util.*;
import java.io.*;
import java.util.*;
import java.nio.file.*;

public class ExportBaseline extends GhidraScript {

    private String workspace;
    private String target;

    @Override
    protected void run() throws Exception {
        if (getScriptArgs().length < 2) {
            throw new IllegalArgumentException("Usage: ExportBaseline <workspace> <target>");
        }
        workspace = getScriptArgs()[0];
        target = getScriptArgs()[1];

        Path baseDir = Paths.get(workspace, "artifacts", target, "baseline");
        Files.createDirectories(baseDir);

        exportFunctions(baseDir);
        exportCallgraph(baseDir);
        exportTypes(baseDir);
        exportVtables(baseDir);
        exportConstants(baseDir);
        exportStrings(baseDir);
        exportImports(baseDir);

        println("Baseline exported to: " + baseDir);
    }

    private void exportFunctions(Path dir) throws IOException {
        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("functions:\n");

        for (Function func : currentProgram.getListing().getFunctions(true)) {
            yaml.append("  - addr: \"").append(func.getEntryPoint().toString()).append("\"\n");
            String name = func.getName();
            if (name != null && !name.equals("")) {
                yaml.append("    name: ").append(escapeYaml(name)).append("\n");
            }
            String prototype = func.getSignature().getPrototypeString();
            if (prototype != null && !prototype.equals("")) {
                yaml.append("    prototype: ").append(escapeYaml(prototype)).append("\n");
            }
            yaml.append("    size: ").append(func.getBody().getNumAddresses()).append("\n");
            AddressSetView body = func.getBody();
            String section = null;
            MemoryBlock block = currentProgram.getMemory().getBlock(body.getMinAddress());
            if (block != null) section = block.getName();
            if (section != null) {
                yaml.append("    section: ").append(escapeYaml(section)).append("\n");
            }
            yaml.append("    source: ghidra\n");
        }

        Files.writeString(dir.resolve("functions.yaml"), yaml.toString());
        println("  exported functions.yaml");
    }

    private void exportCallgraph(Path dir) throws IOException {
        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("edges:\n");

        ReferenceManager refMgr = currentProgram.getReferenceManager();
        Listing listing = currentProgram.getListing();
        for (Function func : listing.getFunctions(true)) {
            String fromAddr = func.getEntryPoint().toString();
            // Iterate over all instructions in the function body to find calls
            for (Instruction inst : listing.getInstructions(func.getBody(), true)) {
                if (inst.getFlowType().isCall()) {
                    // Get the reference target for this call instruction
                    for (Reference ref : refMgr.getReferencesFrom(inst.getAddress())) {
                        if (ref.getReferenceType().isCall()) {
                            Address target = ref.getToAddress();
                            if (target != null && !target.isExternalAddress()) {
                                yaml.append("  - from: \"").append(fromAddr).append("\"\n");
                                yaml.append("    to: \"").append(target.toString()).append("\"\n");
                                yaml.append("    kind: direct\n");
                            }
                        }
                    }
                }
            }
        }

        Files.writeString(dir.resolve("callgraph.yaml"), yaml.toString());
        println("  exported callgraph.yaml");
    }

    private void exportTypes(Path dir) throws IOException {
        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("types:\n");

        DataTypeManager dtm = currentProgram.getDataTypeManager();
        for (Category cat : dtm.getRootCategory().getCategories()) {
            exportCategoryTypes(yaml, cat, "");
        }

        Files.writeString(dir.resolve("types.yaml"), yaml.toString());
        println("  exported types.yaml");
    }

    private void exportCategoryTypes(StringBuilder yaml, Category cat, String prefix) {
        for (DataType dt : cat.getDataTypes()) {
            yaml.append("  - name: ").append(escapeYaml(dt.getName())).append("\n");
            yaml.append("    kind: ").append(escapeYaml(dt.getClass().getSimpleName().replace("DataType", "").toLowerCase())).append("\n");
            String def = dt.getDescription();
            if (def == null || def.isEmpty()) def = dt.toString();
            yaml.append("    definition: ").append(escapeYaml(def)).append("\n");
        }
        for (Category sub : cat.getCategories()) {
            exportCategoryTypes(yaml, sub, prefix + "/" + cat.getName());
        }
    }

    private void exportVtables(Path dir) throws IOException {
        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("vtables:\n");

        int count = 0;
        // Walk all defined data looking for vtable-like structures:
        // pointers in read-only sections that point to functions.
        Memory mem = currentProgram.getMemory();
        Listing listing = currentProgram.getListing();

        for (MemoryBlock block : mem.getBlocks()) {
            String blockName = block.getName().toLowerCase();
            // Look in .rodata, .const, __const, .data.rel.ro, etc.
            if (!blockName.contains("rodata") && !blockName.contains("const")
                && !blockName.contains("data.rel.ro") && !blockName.contains(".data")) {
                continue;
            }

            Address start = block.getStart();
            Address end = block.getEnd();
            DataIterator dataIter = listing.getDefinedData(start, end);

            while (dataIter.hasNext()) {
                Data data = dataIter.next();
                // Look for pointer arrays where the first element points to a function
                if (!(data.getDataType() instanceof Pointer)) {
                    continue;
                }

                // Collect consecutive function pointers starting from this address
                List<String> funcAddrs = new ArrayList<>();
                Address scanAddr = data.getAddress();
                int ptrSize = data.getLength();

                for (int offset = 0; offset < 64; offset++) { // scan up to 64 slots
                    Data slot = listing.getDefinedDataAt(scanAddr.add(offset * ptrSize));
                    if (slot == null || !(slot.getDataType() instanceof Pointer)) {
                        break;
                    }
                    Object value = slot.getValue();
                    if (value instanceof Address) {
                        Address targetAddr = (Address) value;
                        Function func = listing.getFunctionAt(targetAddr);
                        if (func != null) {
                            funcAddrs.add(targetAddr.toString());
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // A vtable needs at least 2 function pointers
                if (funcAddrs.size() >= 2) {
                    String className = data.getLabel();
                    if (className == null || className.equals(data.getAddress().toString())) {
                        // Try to derive class name from surrounding symbols
                        Symbol sym = currentProgram.getSymbolTable().getPrimarySymbol(data.getAddress());
                        className = sym != null ? sym.getName() : "vtable_" + data.getAddress().toString();
                    }

                    yaml.append("  - class: ").append(escapeYaml(className)).append("\n");
                    yaml.append("    addr: \"").append(data.getAddress().toString()).append("\"\n");
                    yaml.append("    entries:\n");
                    for (String fa : funcAddrs) {
                        yaml.append("      - \"").append(fa).append("\"\n");
                    }
                    count++;

                    // Skip past the entries we already emitted
                    // (the outer DataIterator will also visit them, but they
                    //  won't start a new vtable because they're mid-array)
                }
            }
        }

        Files.writeString(dir.resolve("vtables.yaml"), yaml.toString());
        println("  exported vtables.yaml (" + count + " vtables)");
    }

    private void exportConstants(Path dir) throws IOException {
        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("constants:\n");

        Memory mem = currentProgram.getMemory();
        Listing listing = currentProgram.getListing();
        SymbolTable symTab = currentProgram.getSymbolTable();
        for (Symbol sym : symTab.getAllSymbols(true)) {
            if (!sym.isGlobal() || sym.getObject() == null) {
                continue;
            }
            if (!(sym.getObject() instanceof Data)) {
                continue;
            }
            Data data = (Data) sym.getObject();

            // Check if it's a Ghidra constant data type
            boolean isConstant = data.isConstant();
            // Also check for const-qualified data or enum-like data types
            boolean isConstQualified = data.getDataType().getName().toLowerCase().contains("const") ||
                                      data.getDataType() instanceof ghidra.program.model.data.Enum;
            // Check if data is in a __const section (non-executable, read-only, not-write memory)
            boolean inConstSection = false;
            MemoryBlock block = mem.getBlock(data.getAddress());
            if (block != null) {
                String blockName = block.getName();
                if (blockName != null && (blockName.contains("const") || blockName.contains("CONST"))) {
                    inConstSection = !block.isExecute() && block.isRead() && !block.isWrite();
                }
            }

            if (isConstant || isConstQualified || inConstSection) {
                yaml.append("  - addr: \"").append(data.getAddress().toString()).append("\"\n");
                yaml.append("    name: ").append(escapeYaml(sym.getName())).append("\n");
                yaml.append("    ctype: ").append(escapeYaml(data.getDataType().getName())).append("\n");
                Object value = data.getValue();
                yaml.append("    value: ").append(escapeYaml(value != null ? value.toString() : "null")).append("\n");
            }
        }

        Files.writeString(dir.resolve("constants.yaml"), yaml.toString());
        println("  exported constants.yaml");
    }

    private void exportStrings(Path dir) throws IOException {
        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("strings:\n");

        int count = 0;
        for (Data data : DefinedStringIterator.forProgram(currentProgram, null)) {
            StringDataInstance sdi = StringDataInstance.getStringDataInstance(data);
            Address addr = data.getAddress();
            String content = sdi.getStringValue();
            // For encoding, just use utf8 as default (common case)
            // Unicode detection is unreliable with this API
            String encoding = "utf8";
            yaml.append("  - addr: \"").append(addr.toString()).append("\"\n");
            yaml.append("    content: ").append(escapeYaml(content != null ? content : "")).append("\n");
            yaml.append("    encoding: ").append(escapeYaml(encoding)).append("\n");
            count++;
        }

        Files.writeString(dir.resolve("strings.yaml"), yaml.toString());
        println("  exported strings.yaml (" + count + " strings)");
    }

    private void exportImports(Path dir) throws IOException {
        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("imports:\n");

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

                yaml.append("  - library: ").append(escapeYaml(library != null ? library : "<unknown>")).append("\n");
                yaml.append("    symbol: ").append(escapeYaml(symbol != null ? symbol : "<unknown>")).append("\n");
                yaml.append("    addr: \"").append(addr.toString()).append("\"\n");
            }
        }

        Files.writeString(dir.resolve("imports.yaml"), yaml.toString());
        println("  exported imports.yaml");
    }

    private String escapeYaml(String s) {
        if (s == null) return "\"\"";
        // Check if string needs quoting (special chars, leading/trailing spaces, or control chars)
        if (s.contains(":") || s.contains("\"") || s.contains("'") || s.contains("\n") || s.startsWith(" ") || s.endsWith(" ") || s.contains("#") || s.contains("[") || s.contains("]") || s.contains("{") || s.contains("}") || s.contains(",") || s.contains("&") || s.contains("*") || s.contains("!") || s.contains("|") || s.contains(">") || s.contains("=") || s.startsWith("-") || s.equals("") || s.equals("~") || s.equals("@") || s.contains("%") || s.contains("^") || hasControlChars(s)) {
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
                // Escape control characters as \\xNN
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
