import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;

import org.yaml.snakeyaml.LoaderOptions;
import org.yaml.snakeyaml.Yaml;
import org.yaml.snakeyaml.constructor.SafeConstructor;

public final class YamlParsers {
    private static final int YAML_CODE_POINT_LIMIT = 128 * 1024 * 1024;

    private YamlParsers() {
    }

    public static LoaderOptions createLoaderOptions() {
        LoaderOptions loaderOptions = new LoaderOptions();
        loaderOptions.setAllowDuplicateKeys(false);
        loaderOptions.setCodePointLimit(YAML_CODE_POINT_LIMIT);
        return loaderOptions;
    }

    public static List<FunctionEntry> loadFunctions(Path yamlPath) throws IOException {
        Map<?, ?> root = loadRootMap(yamlPath);
        return readFunctionEntries(root.get("functions"));
    }

    public static List<TypeEntry> loadTypes(Path yamlPath) throws IOException {
        Map<?, ?> root = loadRootMap(yamlPath);
        return readTypeEntries(root.get("types"));
    }

    private static Map<?, ?> loadRootMap(Path yamlPath) throws IOException {
        Yaml yaml = new Yaml(new SafeConstructor(createLoaderOptions()));
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

    private static List<FunctionEntry> readFunctionEntries(Object value) {
        List<FunctionEntry> result = new ArrayList<>();
        if (!(value instanceof List<?>)) {
            return result;
        }

        for (Object item : (List<?>) value) {
            if (!(item instanceof Map<?, ?>)) {
                continue;
            }
            Map<?, ?> entry = (Map<?, ?>) item;
            result.add(new FunctionEntry(
                entry.get("addr"),
                asString(entry.get("name"))
            ));
        }
        return result;
    }

    private static List<TypeEntry> readTypeEntries(Object value) {
        List<TypeEntry> result = new ArrayList<>();
        if (!(value instanceof List<?>)) {
            return result;
        }

        for (Object item : (List<?>) value) {
            if (!(item instanceof Map<?, ?>)) {
                continue;
            }
            Map<?, ?> entry = (Map<?, ?>) item;
            result.add(new TypeEntry(
                asString(entry.get("name")),
                asString(entry.get("kind")),
                asString(entry.get("definition"))
            ));
        }
        return result;
    }

    private static String asString(Object value) {
        return value == null ? null : String.valueOf(value);
    }

    public static final class FunctionEntry {
        private final Object addr;
        private final String name;

        FunctionEntry(Object addr, String name) {
            this.addr = addr;
            this.name = name;
        }

        public String getAddr() {
            return asString(addr);
        }

        public Object getAddrValue() {
            return addr;
        }

        public String getName() {
            return name;
        }
    }

    public static final class TypeEntry {
        private final String name;
        private final String kind;
        private final String definition;

        TypeEntry(String name, String kind, String definition) {
            this.name = name;
            this.kind = kind;
            this.definition = definition;
        }

        public String getName() {
            return name;
        }

        public String getKind() {
            return kind;
        }

        public String getDefinition() {
            return definition;
        }
    }
}
