// Usage: analyzeHeadless <project_dir> <target_name> -process <program> -postScript AnalyzeVtables.java <workspace> <target> [min_entries] [max_entries] [scan_limit] [segments_csv] [min_score] [write_baseline] [overwrite] [report_path]
// Scans likely vtable-bearing sections, scores contiguous function-pointer runs,
// writes a YAML analysis report, and optionally refreshes baseline/vtables.yaml.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.address.AddressOutOfBoundsException;
import ghidra.program.model.address.AddressSpace;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.Listing;
import ghidra.program.model.mem.Memory;
import ghidra.program.model.mem.MemoryAccessException;
import ghidra.program.model.mem.MemoryBlock;
import ghidra.program.model.symbol.Namespace;
import ghidra.program.model.symbol.Symbol;
import ghidra.program.model.symbol.SymbolTable;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.time.Instant;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Set;

public class AnalyzeVtables extends GhidraScript {
    private static final String DEFAULT_SEGMENTS = "rodata,const,data.rel.ro,.data";

    private String workspace;
    private String target;
    private int minEntries;
    private int maxEntries;
    private int scanLimit;
    private int minScore;
    private boolean writeBaseline;
    private boolean overwrite;
    private List<String> segments;
    private Path reportPath;

    @Override
    protected void run() throws Exception {
        parseArgs();

        Files.createDirectories(reportPath.getParent());

        AnalysisResult result = analyze();
        Files.writeString(reportPath, renderAnalysisYaml(result));
        if (writeBaseline) {
            Path baselinePath = Paths.get(workspace, "artifacts", target, "baseline", "vtables.yaml");
            if (Files.exists(baselinePath) && !overwrite) {
                throw new IOException(
                    "Refusing to overwrite existing baseline vtables without overwrite=true: "
                        + baselinePath);
            }
            Files.createDirectories(baselinePath.getParent());
            Files.writeString(baselinePath, renderBaselineYaml(result));
        }

        println(
            "Vtable analysis exported to: "
                + reportPath
                + " (candidates="
                + result.candidates.size()
                + ", accepted="
                + result.acceptedCount()
                + ")");
    }

    private void parseArgs() {
        String[] args = getScriptArgs();
        if (args.length < 2) {
            throw new IllegalArgumentException(
                "Usage: AnalyzeVtables <workspace> <target> [min_entries] [max_entries] [scan_limit] [segments_csv] [min_score] [write_baseline] [overwrite] [report_path]");
        }
        workspace = args[0];
        target = args[1];
        minEntries = parseIntArg(args, 2, 4);
        maxEntries = parseIntArg(args, 3, 20);
        scanLimit = parseIntArg(args, 4, 64);
        segments = parseSegments(args.length > 5 ? args[5] : DEFAULT_SEGMENTS);
        minScore = parseIntArg(args, 6, 4);
        writeBaseline = parseBoolArg(args, 7, false);
        overwrite = parseBoolArg(args, 8, false);
        reportPath = resolveReportPath(args.length > 9 ? args[9] : null);
    }

    private AnalysisResult analyze() throws Exception {
        Memory memory = currentProgram.getMemory();
        int pointerSize = currentProgram.getDefaultPointerSize();
        List<Candidate> candidates = new ArrayList<>();
        Set<String> scannedBlocks = new LinkedHashSet<>(segments);

        for (MemoryBlock block : memory.getBlocks()) {
            if (!shouldScanBlock(block)) {
                continue;
            }

            scannedBlocks.add(block.getName());

            Address cursor = alignAddress(block.getStart(), pointerSize);
            Address lastStart = lastPointerStart(block, pointerSize);
            if (cursor == null || lastStart == null || cursor.compareTo(lastStart) > 0) {
                continue;
            }

            while (cursor.compareTo(lastStart) <= 0) {
                Candidate candidate = analyzeCandidateAt(block, cursor, pointerSize);
                if (candidate != null) {
                    candidates.add(candidate);
                    cursor = advanceAddress(cursor, candidate.entries.size() * pointerSize);
                    if (cursor == null) {
                        break;
                    }
                    continue;
                }

                cursor = advanceAddress(cursor, pointerSize);
                if (cursor == null) {
                    break;
                }
            }
        }

        Collections.sort(
            candidates,
            Comparator
                .comparingInt((Candidate c) -> c.score)
                .reversed()
                .thenComparing(c -> c.addr.toString()));
        return new AnalysisResult(pointerSize, new ArrayList<>(scannedBlocks), candidates);
    }

    private boolean shouldScanBlock(MemoryBlock block) {
        if (!block.isInitialized() || block.isExecute()) {
            return false;
        }
        String name = block.getName().toLowerCase(Locale.ROOT);
        for (String segment : segments) {
            if (name.contains(segment)) {
                return true;
            }
        }
        return !block.isWrite() && name.contains("vtable");
    }

    private Candidate analyzeCandidateAt(MemoryBlock block, Address start, int pointerSize)
        throws MemoryAccessException {
        Listing listing = currentProgram.getListing();
        List<Entry> entries = new ArrayList<>();
        String stopReason = "block-end";

        for (int slot = 0; slot < scanLimit; slot++) {
            Address slotAddr = advanceAddress(start, slot * pointerSize);
            if (slotAddr == null || !block.contains(slotAddr)) {
                stopReason = "block-end";
                break;
            }
            Entry entry = readFunctionPointerEntry(slotAddr, slot, pointerSize);
            if (entry == null) {
                stopReason = "non-function-pointer@" + slotAddr.toString();
                break;
            }
            entries.add(entry);
        }

        if (entries.size() < minEntries) {
            return null;
        }

        String symbolHint = symbolHint(start);
        String namespaceHint = namespaceHint(entries);
        String classHint = firstNonEmpty(cleanClassHint(symbolHint), namespaceHint);
        List<String> associationEvidence = new ArrayList<>();
        if (symbolHint != null) {
            associationEvidence.add("symbol:" + symbolHint);
        }
        String typeHint = typeHint(classHint);
        int score = scoreCandidate(block, symbolHint, classHint, typeHint, entries);

        Candidate candidate = new Candidate();
        candidate.addr = start;
        candidate.section = block.getName();
        candidate.score = score;
        candidate.accepted = score >= minScore;
        candidate.classHint = classHint;
        candidate.symbolHint = emptyToNull(symbolHint);
        candidate.typeHint = emptyToNull(typeHint);
        candidate.stopReason = stopReason;
        candidate.entries = entries;
        candidate.entryCount = entries.size();
        candidate.confidence = confidenceFor(score);
        candidate.associationEvidence = associationEvidence;
        candidate.signatureSummary = summarizeSignatures(entries);
        candidate.reasons = buildReasons(block, candidate, symbolHint);

        if (candidate.classHint == null) {
            candidate.classHint = "vtable_" + start.toString();
        }
        return candidate;
    }

    private Entry readFunctionPointerEntry(Address slotAddr, int slot, int pointerSize)
        throws MemoryAccessException {
        Address targetAddr = readPointer(slotAddr, pointerSize);
        if (targetAddr == null) {
            return null;
        }

        Listing listing = currentProgram.getListing();
        Function function = listing.getFunctionAt(targetAddr);
        if (function == null) {
            function = listing.getFunctionContaining(targetAddr);
        }
        if (function == null) {
            return null;
        }

        Entry entry = new Entry();
        entry.slot = slot;
        entry.pointerAddr = slotAddr;
        entry.functionAddr = function.getEntryPoint();
        entry.functionName = function.getName();
        entry.prototype = function.getSignature().getPrototypeString();
        entry.callingConvention = function.getCallingConventionName();
        return entry;
    }

    private Address readPointer(Address addr, int pointerSize) throws MemoryAccessException {
        byte[] bytes = new byte[pointerSize];
        int read = currentProgram.getMemory().getBytes(addr, bytes);
        if (read != pointerSize) {
            return null;
        }

        long offset = 0;
        if (currentProgram.getLanguage().isBigEndian()) {
            for (byte value : bytes) {
                offset = (offset << 8) | (value & 0xffL);
            }
        } else {
            for (int i = pointerSize - 1; i >= 0; i--) {
                offset = (offset << 8) | (bytes[i] & 0xffL);
            }
        }

        AddressSpace space = currentProgram.getAddressFactory().getDefaultAddressSpace();
        try {
            return space.getAddress(offset);
        } catch (AddressOutOfBoundsException e) {
            return null;
        }
    }

    private int scoreCandidate(
        MemoryBlock block,
        String symbolHint,
        String classHint,
        String typeHint,
        List<Entry> entries) {
        int entryCount = entries.size();
        int score = 0;
        if (entryCount >= minEntries && entryCount <= maxEntries) {
            score += 3;
        } else {
            score += 1;
        }
        if (!block.isWrite()) {
            score += 2;
        }

        String blockName = block.getName().toLowerCase(Locale.ROOT);
        if (blockName.contains("vtable")) {
            score += 2;
        } else if (
            blockName.contains("rodata")
                || blockName.contains("const")
                || blockName.contains("data.rel.ro")
        ) {
            score += 1;
        }

        if (symbolHint != null) {
            String normalized = symbolHint.toLowerCase(Locale.ROOT);
            if (
                normalized.contains("vtable")
                    || normalized.contains("_ztv")
                    || normalized.contains("virtual")
            ) {
                score += 2;
            } else {
                score += 1;
            }
        }

        if (classHint != null) {
            score += 1;
        }
        if (typeHint != null) {
            score += 1;
        }
        if (!entries.isEmpty() && isDestructorLike(entries.get(0).functionName)) {
            score += 2;
        }
        if (methodLikeEntries(entries) >= Math.max(2, entryCount / 2)) {
            score += 2;
        }
        if (singleCallingConvention(entries)) {
            score += 1;
        }
        return score;
    }

    private String symbolHint(Address addr) {
        SymbolTable symbols = currentProgram.getSymbolTable();
        Symbol primary = symbols.getPrimarySymbol(addr);
        return primary != null ? primary.getName() : null;
    }

    private String namespaceHint(List<Entry> entries) {
        String best = null;
        int bestCount = 0;

        for (Entry entry : entries) {
            Function function = currentProgram.getListing().getFunctionAt(entry.functionAddr);
            if (function == null) {
                continue;
            }

            Namespace namespace = function.getParentNamespace();
            String candidate = null;
            if (namespace != null) {
                candidate = namespace.getName();
            }

            if (candidate == null || candidate.isEmpty() || "Global".equals(candidate)) {
                String name = function.getName();
                int sep = name.indexOf("::");
                if (sep > 0) {
                    candidate = name.substring(0, sep);
                }
            }

            if (candidate == null || candidate.isEmpty()) {
                continue;
            }

            int count = 0;
            for (Entry probe : entries) {
                if (probe.functionName != null && probe.functionName.startsWith(candidate + "::")) {
                    count++;
                }
            }
            if (count > bestCount) {
                best = candidate;
                bestCount = count;
            }
        }

        return best;
    }

    private String typeHint(String classHint) {
        return classHint;
    }

    private String cleanClassHint(String raw) {
        if (raw == null) {
            return null;
        }

        String hint = raw.trim();
        if (hint.startsWith("_ZTV")) {
            hint = hint.substring(4);
        }
        if (hint.startsWith("??_7")) {
            hint = hint.substring(4);
        }
        String lower = hint.toLowerCase(Locale.ROOT);
        if (lower.startsWith("vtable for ")) {
            hint = hint.substring("vtable for ".length());
        } else if (lower.startsWith("vtable_")) {
            hint = hint.substring("vtable_".length());
        }
        hint = hint.replace("`vftable'", "").replace("::`vftable'", "");
        hint = hint.replace("::`vtable'", "").replace("`vtable'", "");
        hint = hint.trim();
        return hint.isEmpty() ? null : hint;
    }

    private String renderAnalysisYaml(AnalysisResult result) throws IOException {
        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("generated_at: ").append(escapeYaml(Instant.now().toString())).append("\n");
        yaml.append("pointer_size: ").append(result.pointerSize).append("\n");
        yaml.append("scan_segments:\n");
        for (String blockName : result.scannedBlocks) {
            yaml.append("  - ").append(escapeYaml(blockName)).append("\n");
        }
        yaml.append("candidates:\n");
        for (Candidate candidate : result.candidates) {
            yaml.append("  - addr: \"").append(candidate.addr.toString()).append("\"\n");
            yaml.append("    status: ").append(candidate.accepted ? "accepted" : "rejected").append("\n");
            yaml.append("    class: ").append(escapeYaml(candidate.classHint)).append("\n");
            yaml.append("    segment: ").append(escapeYaml(candidate.section)).append("\n");
            yaml.append("    score: ").append(candidate.score).append("\n");
            yaml.append("    confidence: ").append(escapeYaml(candidate.confidence)).append("\n");
            yaml.append("    entry_count: ").append(candidate.entryCount).append("\n");
            yaml.append("    entries:\n");
            for (Entry entry : candidate.entries) {
                yaml.append("      - \"").append(entry.functionAddr.toString()).append("\"\n");
            }
            yaml.append("    reasons:\n");
            for (String reason : candidate.reasons) {
                yaml.append("      - ").append(escapeYaml(reason)).append("\n");
            }
            appendOptional(yaml, "associated_type", candidate.typeHint, 4);
            if (!candidate.associationEvidence.isEmpty()) {
                yaml.append("    association_evidence:\n");
                for (String evidence : candidate.associationEvidence) {
                    yaml.append("      - ").append(escapeYaml(evidence)).append("\n");
                }
            }
            appendOptional(yaml, "signature_summary", candidate.signatureSummary, 4);
        }
        return yaml.toString();
    }

    private String renderBaselineYaml(AnalysisResult result) {
        StringBuilder yaml = new StringBuilder();
        yaml.append("target: ").append(escapeYaml(target)).append("\n");
        yaml.append("vtables:\n");
        for (Candidate candidate : result.candidates) {
            if (!candidate.accepted) {
                continue;
            }
            yaml.append("  - class: ").append(escapeYaml(candidate.classHint)).append("\n");
            yaml.append("    addr: \"").append(candidate.addr.toString()).append("\"\n");
            yaml.append("    entries:\n");
            for (Entry entry : candidate.entries) {
                yaml.append("      - \"").append(entry.functionAddr.toString()).append("\"\n");
            }
            yaml.append("    entry_count: ").append(candidate.entryCount).append("\n");
            yaml.append("    confidence: ").append(escapeYaml(candidate.confidence)).append("\n");
            yaml.append("    score: ").append(candidate.score).append("\n");
            yaml.append("    source: \"ghidra_auto\"\n");
            yaml.append("    segment: ").append(escapeYaml(candidate.section)).append("\n");
            appendOptional(yaml, "associated_type", candidate.typeHint, 4);
            if (!candidate.associationEvidence.isEmpty()) {
                yaml.append("    association_evidence:\n");
                for (String evidence : candidate.associationEvidence) {
                    yaml.append("      - ").append(escapeYaml(evidence)).append("\n");
                }
            }
            appendOptional(yaml, "signature_summary", candidate.signatureSummary, 4);
        }
        return yaml.toString();
    }

    private int parseIntArg(String[] args, int index, int defaultValue) {
        if (args.length <= index || args[index] == null || args[index].isEmpty()) {
            return defaultValue;
        }
        return Integer.parseInt(args[index]);
    }

    private boolean parseBoolArg(String[] args, int index, boolean defaultValue) {
        if (args.length <= index || args[index] == null || args[index].isEmpty()) {
            return defaultValue;
        }
        return Boolean.parseBoolean(args[index]);
    }

    private List<String> parseSegments(String csv) {
        String raw = (csv == null || csv.isEmpty()) ? DEFAULT_SEGMENTS : csv;
        List<String> parsed = new ArrayList<>();
        for (String item : raw.split(",")) {
            String value = item.trim().toLowerCase(Locale.ROOT);
            if (!value.isEmpty()) {
                parsed.add(value);
            }
        }
        return parsed;
    }

    private Path resolveReportPath(String rawPath) {
        if (rawPath == null || rawPath.isEmpty()) {
            return Paths.get(workspace, "artifacts", target, "baseline", "vtable-analysis-report.yaml");
        }
        Path path = Paths.get(rawPath);
        if (!path.isAbsolute()) {
            path = Paths.get(workspace).resolve(path).normalize();
        }
        return path;
    }

    private void appendOptional(StringBuilder yaml, String key, String value, int indent) {
        if (value == null || value.isEmpty()) {
            return;
        }
        for (int i = 0; i < indent; i++) {
            yaml.append(' ');
        }
        yaml.append(key).append(": ").append(escapeYaml(value)).append("\n");
    }

    private Address alignAddress(Address addr, int pointerSize) {
        long offset = addr.getOffset();
        long aligned = offset;
        long remainder = offset % pointerSize;
        if (remainder != 0) {
            aligned += pointerSize - remainder;
        }
        try {
            return addr.getAddressSpace().getAddress(aligned);
        } catch (AddressOutOfBoundsException e) {
            return null;
        }
    }

    private Address lastPointerStart(MemoryBlock block, int pointerSize) {
        long endOffset = block.getEnd().getOffset();
        long startOffset = block.getStart().getOffset();
        long lastOffset = endOffset - pointerSize + 1;
        if (lastOffset < startOffset) {
            return null;
        }
        try {
            return block.getStart().getAddressSpace().getAddress(lastOffset);
        } catch (AddressOutOfBoundsException e) {
            return null;
        }
    }

    private Address advanceAddress(Address addr, int amount) {
        try {
            return addr.add(amount);
        } catch (AddressOutOfBoundsException e) {
            return null;
        }
    }

    private String firstNonEmpty(String first, String second) {
        return first != null && !first.isEmpty() ? first : second;
    }

    private String emptyToNull(String value) {
        return value == null || value.isEmpty() ? null : value;
    }

    private String confidenceFor(int score) {
        if (score >= 9) {
            return "high";
        }
        if (score >= minScore) {
            return "medium";
        }
        return "low";
    }

    private boolean isDestructorLike(String functionName) {
        if (functionName == null) {
            return false;
        }
        String lower = functionName.toLowerCase(Locale.ROOT);
        return lower.contains("dtor") || lower.contains("destruct") || lower.contains("~");
    }

    private int methodLikeEntries(List<Entry> entries) {
        int count = 0;
        for (Entry entry : entries) {
            String lower = entry.prototype == null ? "" : entry.prototype.toLowerCase(Locale.ROOT);
            if (lower.contains("(void *") || lower.contains("(void*") || lower.contains("this")) {
                count++;
            }
        }
        return count;
    }

    private boolean singleCallingConvention(List<Entry> entries) {
        Set<String> conventions = new LinkedHashSet<>();
        for (Entry entry : entries) {
            if (entry.callingConvention != null && !entry.callingConvention.isEmpty()) {
                conventions.add(entry.callingConvention);
            }
        }
        return !conventions.isEmpty() && conventions.size() == 1;
    }

    private String summarizeSignatures(List<Entry> entries) {
        List<String> parts = new ArrayList<>();
        for (Entry entry : entries) {
            if (entry.prototype != null && !entry.prototype.isEmpty()) {
                parts.add(entry.prototype);
            }
            if (parts.size() == 3) {
                break;
            }
        }
        return parts.isEmpty() ? null : String.join(" | ", parts);
    }

    private List<String> buildReasons(MemoryBlock block, Candidate candidate, String symbolHint) {
        List<String> reasons = new ArrayList<>();
        reasons.add("entry_count=" + candidate.entryCount);
        if (candidate.entryCount >= minEntries && candidate.entryCount <= maxEntries) {
            reasons.add("entry_count_in_expected_range=true");
        }
        if (!block.isWrite()) {
            reasons.add("readonly_block=true");
        }
        if (symbolHint != null) {
            reasons.add("symbol_hint_present=true");
        }
        if (!candidate.entries.isEmpty() && isDestructorLike(candidate.entries.get(0).functionName)) {
            reasons.add("first_entry_destructor_like=true");
        }
        if (candidate.signatureSummary != null) {
            reasons.add("signature_summary_present=true");
        }
        if (!candidate.accepted) {
            reasons.add("score_below_threshold=" + minScore);
        }
        return reasons;
    }

    private String escapeYaml(String value) {
        if (value == null) {
            return "\"\"";
        }
        return "\"" + value
            .replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", "\\n")
            + "\"";
    }

    private static final class AnalysisResult {
        final int pointerSize;
        final List<String> scannedBlocks;
        final List<Candidate> candidates;

        AnalysisResult(int pointerSize, List<String> scannedBlocks, List<Candidate> candidates) {
            this.pointerSize = pointerSize;
            this.scannedBlocks = scannedBlocks;
            this.candidates = candidates;
        }

        int acceptedCount() {
            int count = 0;
            for (Candidate candidate : candidates) {
                if (candidate.accepted) {
                    count++;
                }
            }
            return count;
        }
    }

    private static final class Candidate {
        Address addr;
        String section;
        int score;
        int entryCount;
        boolean accepted;
        String confidence;
        String classHint;
        String symbolHint;
        String typeHint;
        String signatureSummary;
        String stopReason;
        List<String> reasons;
        List<String> associationEvidence;
        List<Entry> entries;
    }

    private static final class Entry {
        int slot;
        Address pointerAddr;
        Address functionAddr;
        String functionName;
        String prototype;
        String callingConvention;
    }
}
