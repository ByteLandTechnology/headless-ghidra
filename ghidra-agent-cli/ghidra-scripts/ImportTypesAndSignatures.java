// Usage:
//   analyzeHeadless <project_dir> <target_name> -process <program> \
//     -postScript ImportTypesAndSignatures.java <workspace> <target> \
//     --header <path> [--header <path> ...] [--include-dir <path> ...] [--signatures <path>]

import ghidra.app.script.GhidraScript;
import ghidra.app.util.cparser.C.CParserUtils;
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

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.yaml.snakeyaml.Yaml;
import org.yaml.snakeyaml.constructor.SafeConstructor;

public class ImportTypesAndSignatures extends GhidraScript {

    private String workspace;
    private String target;
    private final List<String> headers = new ArrayList<>();
    private final List<String> includeDirs = new ArrayList<>();
    private String signaturesPath;

    private int typeFilesImported = 0;
    private int typeFilesFailed = 0;
    private int signaturesApplied = 0;
    private int signaturesSkipped = 0;
    private int signaturesFailed = 0;

    @Override
    protected void run() throws Exception {
        parseArgs();

        int transactionId = currentProgram.startTransaction("Import custom types and function signatures");
        boolean commit = false;
        try {
            importHeaders();
            applySignatures();

            if (typeFilesFailed > 0 || signaturesFailed > 0) {
                throw new RuntimeException(
                    "ImportTypesAndSignatures completed with "
                        + typeFilesFailed
                        + " header failure(s) and "
                        + signaturesFailed
                        + " signature failure(s)");
            }
            commit = true;
        } finally {
            currentProgram.endTransaction(transactionId, commit);
        }

        if (typeFilesImported > 0 || signaturesApplied > 0) {
            currentProgram.flushEvents();
        }

        println(
            "ImportTypesAndSignatures: headers="
                + headers.size()
                + " imported="
                + typeFilesImported
                + " failed="
                + typeFilesFailed
                + " signatures_applied="
                + signaturesApplied
                + " signatures_skipped="
                + signaturesSkipped
                + " signatures_failed="
                + signaturesFailed);
    }

    private void parseArgs() {
        String[] args = getScriptArgs();
        if (args.length < 3) {
            throw new IllegalArgumentException(
                "Usage: ImportTypesAndSignatures <workspace> <target> --header <path> [--header <path> ...] [--include-dir <path> ...] [--signatures <path>]");
        }
        workspace = args[0];
        target = args[1];

        for (int i = 2; i < args.length; i++) {
            String arg = args[i];
            if ("--header".equals(arg)) {
                if (i + 1 >= args.length) {
                    throw new IllegalArgumentException("--header requires a path");
                }
                headers.add(args[++i]);
            } else if ("--signatures".equals(arg)) {
                if (i + 1 >= args.length) {
                    throw new IllegalArgumentException("--signatures requires a path");
                }
                signaturesPath = args[++i];
            } else if ("--include-dir".equals(arg)) {
                if (i + 1 >= args.length) {
                    throw new IllegalArgumentException("--include-dir requires a path");
                }
                includeDirs.add(args[++i]);
            } else {
                throw new IllegalArgumentException("Unknown argument: " + arg);
            }
        }

        if (headers.isEmpty()) {
            throw new IllegalArgumentException("At least one --header path is required");
        }
        if (signaturesPath == null || signaturesPath.trim().isEmpty()) {
            Path defaultPath = Paths.get(workspace, "artifacts", target, "metadata", "signatures.yaml");
            if (Files.exists(defaultPath)) {
                signaturesPath = defaultPath.toString();
            }
        }
    }

    private void importHeaders() {
        Set<String> normalizedHeaders = new LinkedHashSet<>();
        Set<String> normalizedIncludeDirs = new LinkedHashSet<>(includeDirs);
        for (String headerPath : headers) {
            File headerFile = new File(headerPath);
            if (!headerFile.exists()) {
                println("WARN: header not found: " + headerPath);
                typeFilesFailed++;
                continue;
            }
            normalizedHeaders.add(headerFile.getAbsolutePath());
            File parent = headerFile.getParentFile();
            if (parent != null) {
                normalizedIncludeDirs.add(parent.getAbsolutePath());
            }
        }
        if (normalizedHeaders.isEmpty()) {
            return;
        }

        String[] headerPaths = normalizedHeaders.toArray(new String[0]);
        String[] includePaths = normalizedIncludeDirs.toArray(new String[0]);
        try {
            DataTypeManager dtm = currentProgram.getDataTypeManager();
            DataTypeManager[] openTypes = new DataTypeManager[] {
                dtm,
                BuiltInDataTypeManager.getDataTypeManager()
            };
            CParserUtils.CParseResults results =
                CParserUtils.parseHeaderFiles(openTypes, headerPaths, includePaths, dtm, monitor);
            if (results != null && !results.successful()) {
                typeFilesFailed += headerPaths.length;
                println("WARN: C header parser reported failure");
                println(results.getFormattedParseMessage("ImportTypesAndSignatures"));
                return;
            }
            typeFilesImported += headerPaths.length;
            for (String headerPath : headerPaths) {
                println("Imported header: " + headerPath);
            }
        } catch (Exception e) {
            typeFilesFailed += headerPaths.length;
            println("WARN: failed to import headers: " + e.getMessage());
        }
    }

    private void applySignatures() throws Exception {
        if (signaturesPath == null || signaturesPath.trim().isEmpty()) {
            println("No signatures file supplied; skipping signature import");
            return;
        }

        Path yamlPath = Paths.get(signaturesPath);
        if (!Files.exists(yamlPath)) {
            println("WARN: signatures file not found: " + signaturesPath);
            return;
        }

        List<Map<?, ?>> entries = loadSignatureEntries(yamlPath);
        DataTypeManager dtm = currentProgram.getDataTypeManager();
        FunctionManager fm = currentProgram.getFunctionManager();
        DataTypeParser parser = new DataTypeParser(
            dtm,
            BuiltInDataTypeManager.getDataTypeManager(),
            null,
            AllowedDataTypes.ALL);

        for (Map<?, ?> entry : entries) {
            String addrStr = asString(entry.get("addr"));
            String signature = firstNonEmptyString(entry.get("prototype"), entry.get("signature"));
            String requestedName = firstNonEmptyString(entry.get("name"), entry.get("function"));
            if (addrStr == null || signature == null) {
                signaturesSkipped++;
                continue;
            }

            try {
                Address addr = currentProgram.getAddressFactory()
                    .getDefaultAddressSpace()
                    .getAddress(addrStr);
                Function function = fm.getFunctionAt(addr);
                if (function == null) {
                    function = fm.getFunctionContaining(addr);
                }
                if (function == null) {
                    signaturesSkipped++;
                    continue;
                }

                ParsedSignature parsedSig = parseSignature(signature, requestedName);
                if (parsedSig == null) {
                    signaturesSkipped++;
                    continue;
                }

                applySignature(function, parsedSig, parser);
                signaturesApplied++;
            } catch (Exception e) {
                signaturesFailed++;
                println("WARN: failed to apply signature at " + addrStr + ": " + e.getMessage());
            }
        }
    }

    private List<Map<?, ?>> loadSignatureEntries(Path yamlPath) throws IOException {
        Yaml yaml = new Yaml(new SafeConstructor(YamlParsers.createLoaderOptions()));
        try (InputStream input = Files.newInputStream(yamlPath)) {
            Object root = yaml.load(input);
            if (!(root instanceof Map<?, ?>)) {
                return Collections.emptyList();
            }
            Object signatures = ((Map<?, ?>) root).get("signatures");
            if (!(signatures instanceof List<?>)) {
                return Collections.emptyList();
            }
            List<Map<?, ?>> entries = new ArrayList<>();
            for (Object item : (List<?>) signatures) {
                if (item instanceof Map<?, ?>) {
                    entries.add((Map<?, ?>) item);
                }
            }
            return entries;
        }
    }

    private ParsedSignature parseSignature(String sig, String requestedName) {
        sig = sig.trim().replaceAll("\\s+", " ");
        int parenOpen = sig.indexOf('(');
        if (parenOpen < 0) {
            return null;
        }

        String beforeParen = sig.substring(0, parenOpen).trim();
        String paramsStr = sig.substring(parenOpen + 1);
        if (paramsStr.endsWith(")")) {
            paramsStr = paramsStr.substring(0, paramsStr.length() - 1);
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

        return new ParsedSignature(returnType, functionName, paramsStr);
    }

    private void applySignature(Function func, ParsedSignature parsedSig, DataTypeParser parser) throws Exception {
        DataType returnType = resolveDataType(parsedSig.returnType, parser);
        List<ParameterImpl> newParams = new ArrayList<>();
        boolean hasVarArgs = false;
        if (!parsedSig.params.trim().isEmpty() && !"void".equals(parsedSig.params.trim())) {
            List<String> paramTokens = splitParams(parsedSig.params);
            int idx = 0;
            for (String param : paramTokens) {
                param = param.trim();
                if (param.isEmpty() || "void".equals(param)) {
                    continue;
                }
                if ("...".equals(param)) {
                    hasVarArgs = true;
                    continue;
                }
                String[] parsedParam = parseParam(param, idx);
                DataType paramType = resolveDataType(parsedParam[0], parser);
                newParams.add(
                    new ParameterImpl(
                        parsedParam[1],
                        paramType,
                        currentProgram,
                        SourceType.USER_DEFINED));
                idx++;
            }
        }

        func.setReturnType(returnType, SourceType.USER_DEFINED);
        func.replaceParameters(
            newParams,
            Function.FunctionUpdateType.DYNAMIC_STORAGE_ALL_PARAMS,
            true,
            SourceType.USER_DEFINED);
        func.setVarArgs(hasVarArgs);
    }

    private DataType resolveDataType(String typeStr, DataTypeParser parser) throws Exception {
        String normalized = typeStr.trim()
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
        } catch (Exception e) {
            throw new RuntimeException("failed to resolve data type `" + normalized + "`", e);
        }
    }

    private String[] parseParam(String param, int index) {
        param = param.replace("const ", "").replace("struct ", "").replace("enum ", "").trim();
        if (param.contains("(*)")) {
            return new String[] {"void *", "func_ptr_" + index};
        }
        param = param.replaceAll("\\[\\d*\\]", "").trim();
        if ("...".equals(param)) {
            return new String[] {"void", "varargs"};
        }

        String[] tokens = param.split("\\s+");
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

    private List<String> splitParams(String paramsStr) {
        List<String> result = new ArrayList<>();
        int depth = 0;
        StringBuilder current = new StringBuilder();

        for (char c : paramsStr.toCharArray()) {
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
