// Usage:
//   analyzeHeadless <project_dir> <target_name> -process <program> \
//     -postScript ApplySignatures.java <workspace> <target> [--rename-from-signature]
//
// Applies function signatures from metadata/signatures.yaml. If that file is
// absent, falls back to legacy baseline/types.yaml entries with kind: function.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.data.BuiltInDataTypeManager;
import ghidra.program.model.data.DataType;
import ghidra.program.model.data.DataTypeManager;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.FunctionManager;
import ghidra.program.model.listing.ParameterImpl;
import ghidra.program.model.symbol.SourceType;
import ghidra.util.data.DataTypeParser;
import ghidra.util.data.DataTypeParser.AllowedDataTypes;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;

import org.yaml.snakeyaml.LoaderOptions;
import org.yaml.snakeyaml.Yaml;
import org.yaml.snakeyaml.constructor.SafeConstructor;

public class ApplySignatures extends GhidraScript {

    private String workspace;
    private String target;
    private boolean renameFromSignature = false;

    private int applied = 0;
    private int renamed = 0;
    private int skipped = 0;
    private int failed = 0;

    @Override
    protected void run() throws Exception {
        parseArgs();

        List<SignatureEntry> entries = loadSignatureEntries();
        if (entries.isEmpty()) {
            throw new IOException("No function signature entries found for target " + target);
        }

        DataTypeManager dtm = currentProgram.getDataTypeManager();
        DataTypeParser parser = new DataTypeParser(
            dtm,
            BuiltInDataTypeManager.getDataTypeManager(),
            null,
            AllowedDataTypes.ALL);

        int transactionId = currentProgram.startTransaction("Apply function signatures");
        boolean commit = false;
        try {
            for (SignatureEntry entry : entries) {
                try {
                    Function function = resolveFunction(entry);
                    if (function == null) {
                        skipped++;
                        println("WARN: function not found for signature entry: " + entry.describe());
                        continue;
                    }

                    ParsedSignature parsed = parseSignature(entry.prototype, entry.name);
                    if (parsed == null) {
                        skipped++;
                        println("WARN: unable to parse signature: " + entry.prototype);
                        continue;
                    }

                    applySignature(function, parsed, parser);
                    applied++;
                } catch (Exception error) {
                    failed++;
                    println("WARN: failed to apply signature for " + entry.describe() + ": " + error.getMessage());
                }
            }

            if (failed > 0) {
                throw new RuntimeException("ApplySignatures completed with " + failed + " failure(s)");
            }
            commit = true;
        } finally {
            currentProgram.endTransaction(transactionId, commit);
        }

        if (applied > 0 || renamed > 0) {
            currentProgram.flushEvents();
        }

        println(
            "ApplySignatures: "
                + applied
                + " applied, "
                + renamed
                + " renamed, "
                + skipped
                + " skipped, "
                + failed
                + " failed");
    }

    private void parseArgs() {
        String[] args = getScriptArgs();
        if (args.length < 2) {
            throw new IllegalArgumentException(
                "Usage: ApplySignatures <workspace> <target> [--rename-from-signature]");
        }
        workspace = args[0];
        target = args[1];

        for (int i = 2; i < args.length; i++) {
            if ("--rename-from-signature".equals(args[i])) {
                renameFromSignature = true;
            } else {
                throw new IllegalArgumentException("Unknown argument: " + args[i]);
            }
        }
    }

    private List<SignatureEntry> loadSignatureEntries() throws IOException {
        Path metadataPath = Paths.get(workspace, "artifacts", target, "metadata", "signatures.yaml");
        if (Files.exists(metadataPath)) {
            return loadMetadataSignatures(metadataPath);
        }

        Path legacyTypesPath = Paths.get(workspace, "artifacts", target, "baseline", "types.yaml");
        if (Files.exists(legacyTypesPath)) {
            return loadLegacyFunctionTypes(legacyTypesPath);
        }

        throw new IOException(
            "No signature source found: expected "
                + metadataPath
                + " or "
                + legacyTypesPath);
    }

    private List<SignatureEntry> loadMetadataSignatures(Path yamlPath) throws IOException {
        Map<?, ?> root = loadRootMap(yamlPath);
        Object signatures = root.get("signatures");
        if (!(signatures instanceof List<?>)) {
            return Collections.emptyList();
        }

        List<SignatureEntry> entries = new ArrayList<>();
        for (Object item : (List<?>) signatures) {
            if (!(item instanceof Map<?, ?>)) {
                continue;
            }
            Map<?, ?> map = (Map<?, ?>) item;
            String prototype = firstNonEmptyString(map.get("prototype"), map.get("signature"));
            if (prototype == null) {
                continue;
            }
            entries.add(new SignatureEntry(
                asString(map.get("addr")),
                firstNonEmptyString(map.get("name"), map.get("function")),
                prototype,
                "metadata/signatures.yaml"));
        }
        return entries;
    }

    private List<SignatureEntry> loadLegacyFunctionTypes(Path yamlPath) throws IOException {
        List<YamlParsers.TypeEntry> types = YamlParsers.loadTypes(yamlPath);
        List<SignatureEntry> entries = new ArrayList<>();
        for (YamlParsers.TypeEntry entry : types) {
            if (!"function".equals(entry.getKind())) {
                continue;
            }
            if (entry.getDefinition() == null) {
                continue;
            }
            entries.add(new SignatureEntry(
                null,
                entry.getName(),
                entry.getDefinition(),
                "baseline/types.yaml"));
        }
        return entries;
    }

    private Map<?, ?> loadRootMap(Path yamlPath) throws IOException {
        LoaderOptions loaderOptions = new LoaderOptions();
        loaderOptions.setAllowDuplicateKeys(false);
        Yaml yaml = new Yaml(new SafeConstructor(loaderOptions));
        try (InputStream input = Files.newInputStream(yamlPath)) {
            Object parsed = yaml.load(input);
            if (parsed == null) {
                return Collections.emptyMap();
            }
            if (!(parsed instanceof Map<?, ?>)) {
                throw new IOException("Expected YAML mapping at root: " + yamlPath);
            }
            return (Map<?, ?>) parsed;
        }
    }

    private Function resolveFunction(SignatureEntry entry) throws Exception {
        FunctionManager functionManager = currentProgram.getFunctionManager();
        if (entry.addr != null && !entry.addr.trim().isEmpty()) {
            Address address = currentProgram.getAddressFactory()
                .getDefaultAddressSpace()
                .getAddress(entry.addr);
            Function function = functionManager.getFunctionAt(address);
            if (function == null) {
                function = functionManager.getFunctionContaining(address);
            }
            return function;
        }

        if (entry.name == null || entry.name.trim().isEmpty()) {
            return null;
        }
        for (Function function : functionManager.getFunctions(true)) {
            if (entry.name.equals(function.getName())) {
                return function;
            }
        }
        return null;
    }

    private ParsedSignature parseSignature(String signature, String requestedName) {
        if (signature == null) {
            return null;
        }

        String normalized = signature.trim().replaceAll("\\s+", " ");
        int parenOpen = normalized.indexOf('(');
        if (parenOpen < 0) {
            return null;
        }

        String beforeParen = normalized.substring(0, parenOpen).trim();
        String params = normalized.substring(parenOpen + 1);
        if (params.endsWith(")")) {
            params = params.substring(0, params.length() - 1);
        }

        String functionName = requestedName;
        String returnType = beforeParen;
        int nameEnd = beforeParen.length();
        int nameStart = nameEnd;
        while (nameStart > 0) {
            char c = beforeParen.charAt(nameStart - 1);
            if (Character.isLetterOrDigit(c) || c == '_') {
                nameStart--;
            } else {
                break;
            }
        }

        if (nameStart < nameEnd) {
            String possibleName = beforeParen.substring(nameStart, nameEnd);
            String possibleReturnType = beforeParen.substring(0, nameStart).trim();
            if (!possibleReturnType.isEmpty()) {
                functionName = possibleName;
                returnType = possibleReturnType;
            }
        }

        return new ParsedSignature(returnType, functionName, params);
    }

    private void applySignature(Function function, ParsedSignature parsed, DataTypeParser parser) throws Exception {
        if (renameFromSignature
            && parsed.functionName != null
            && !parsed.functionName.trim().isEmpty()
            && !parsed.functionName.startsWith("FUN_")
            && !parsed.functionName.equals(function.getName())) {
            function.setName(parsed.functionName, SourceType.USER_DEFINED);
            renamed++;
        }

        DataType returnType = resolveDataType(parsed.returnType, parser);
        List<ParameterImpl> parameters = new ArrayList<>();
        boolean hasVarArgs = false;

        if (!parsed.params.trim().isEmpty() && !"void".equals(parsed.params.trim())) {
            List<String> paramTokens = splitParams(parsed.params);
            int index = 0;
            for (String param : paramTokens) {
                String trimmed = param.trim();
                if (trimmed.isEmpty() || "void".equals(trimmed)) {
                    continue;
                }
                if ("...".equals(trimmed)) {
                    hasVarArgs = true;
                    continue;
                }

                String[] parsedParam = parseParam(trimmed, index);
                DataType parameterType = resolveDataType(parsedParam[0], parser);
                parameters.add(
                    new ParameterImpl(
                        parsedParam[1],
                        parameterType,
                        currentProgram,
                        SourceType.USER_DEFINED));
                index++;
            }
        }

        function.setReturnType(returnType, SourceType.USER_DEFINED);
        function.replaceParameters(
            parameters,
            Function.FunctionUpdateType.DYNAMIC_STORAGE_ALL_PARAMS,
            true,
            SourceType.USER_DEFINED);
        function.setVarArgs(hasVarArgs);
    }

    private DataType resolveDataType(String typeText, DataTypeParser parser) throws Exception {
        String normalized = typeText.trim()
            .replaceAll("\\s+", " ")
            .replace("const ", "")
            .replace(" const", "")
            .replace("struct ", "")
            .replace("enum ", "")
            .trim();

        if ("int64_t".equals(normalized)) {
            normalized = "long";
        } else if ("uint64_t".equals(normalized)) {
            normalized = "unsigned long";
        } else if ("uint32_t".equals(normalized)) {
            normalized = "unsigned int";
        } else if ("int32_t".equals(normalized)) {
            normalized = "int";
        } else if ("uint16_t".equals(normalized)) {
            normalized = "short";
        } else if ("int16_t".equals(normalized)) {
            normalized = "short";
        } else if ("uint8_t".equals(normalized)) {
            normalized = "byte";
        } else if ("int8_t".equals(normalized)) {
            normalized = "char";
        } else if ("size_t".equals(normalized)) {
            normalized = "unsigned long";
        } else if ("ssize_t".equals(normalized) || "ptrdiff_t".equals(normalized)) {
            normalized = "long";
        }

        try {
            return parser.parse(normalized);
        } catch (Exception error) {
            throw new RuntimeException("failed to resolve data type `" + normalized + "`", error);
        }
    }

    private String[] parseParam(String param, int index) {
        String normalized = param
            .replace("const ", "")
            .replace("struct ", "")
            .replace("enum ", "")
            .replaceAll("\\[\\d*\\]", "")
            .trim();

        if (normalized.contains("(*)")) {
            return new String[] {"void *", "func_ptr_" + index};
        }

        String[] tokens = normalized.split("\\s+");
        if (tokens.length == 0) {
            return new String[] {"undefined8", "param_" + index};
        }
        if (tokens.length == 1) {
            return new String[] {tokens[0], "param_" + index};
        }

        String name = tokens[tokens.length - 1];
        StringBuilder type = new StringBuilder();
        for (int i = 0; i < tokens.length - 1; i++) {
            if (i > 0) {
                type.append(" ");
            }
            type.append(tokens[i]);
        }

        if (name.startsWith("*")) {
            type.append(" *");
            name = name.substring(name.lastIndexOf('*') + 1).trim();
            if (name.isEmpty()) {
                name = "param_" + index;
            }
        }

        return new String[] {type.toString(), name};
    }

    private List<String> splitParams(String params) {
        List<String> result = new ArrayList<>();
        int depth = 0;
        StringBuilder current = new StringBuilder();

        for (char c : params.toCharArray()) {
            if (c == '(' || c == '[') {
                depth++;
            } else if (c == ')' || c == ']') {
                depth--;
            }

            if (c == ',' && depth == 0) {
                result.add(current.toString().trim());
                current = new StringBuilder();
            } else {
                current.append(c);
            }
        }

        String last = current.toString().trim();
        if (!last.isEmpty()) {
            result.add(last);
        }
        return result;
    }

    private String firstNonEmptyString(Object first, Object second) {
        String firstValue = asString(first);
        if (firstValue != null && !firstValue.trim().isEmpty()) {
            return firstValue;
        }
        String secondValue = asString(second);
        if (secondValue != null && !secondValue.trim().isEmpty()) {
            return secondValue;
        }
        return null;
    }

    private String asString(Object value) {
        return value == null ? null : String.valueOf(value);
    }

    private static final class SignatureEntry {
        final String addr;
        final String name;
        final String prototype;
        final String source;

        SignatureEntry(String addr, String name, String prototype, String source) {
            this.addr = addr;
            this.name = name;
            this.prototype = prototype;
            this.source = source;
        }

        String describe() {
            if (addr != null && !addr.trim().isEmpty()) {
                return source + " addr=" + addr;
            }
            if (name != null && !name.trim().isEmpty()) {
                return source + " name=" + name;
            }
            return source;
        }
    }

    private static final class ParsedSignature {
        final String returnType;
        final String functionName;
        final String params;

        ParsedSignature(String returnType, String functionName, String params) {
            this.returnType = returnType;
            this.functionName = functionName;
            this.params = params;
        }
    }
}
