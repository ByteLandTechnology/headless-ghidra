use serde::Serialize;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct HelpOption {
    pub name: String,
    pub value_name: String,
    pub default_value: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HelpSubcommand {
    pub name: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HelpExample {
    pub command: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExitCodeSpec {
    pub code: i32,
    pub meaning: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HelpDocument {
    pub command_path: Vec<String>,
    pub purpose: String,
    pub usage: String,
    pub arguments: Vec<String>,
    pub options: Vec<HelpOption>,
    pub subcommands: Vec<HelpSubcommand>,
    pub output_formats: Vec<String>,
    pub exit_behavior: Vec<ExitCodeSpec>,
    #[serde(skip_serializing)]
    pub description: Vec<String>,
    #[serde(skip_serializing)]
    pub examples: Vec<HelpExample>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const SKILL_NAME: &str = "ghidra-agent-cli";

const DESCRIPTION: &str = "Drive the headless-Ghidra decompilation pipeline \
    end-to-end through schema-validated YAML workspaces, scripted Ghidra runs, \
    and progressive function-level reconstruction.";

fn standard_exit_codes() -> Vec<ExitCodeSpec> {
    vec![
        ExitCodeSpec {
            code: 0,
            meaning: "Success".into(),
        },
        ExitCodeSpec {
            code: 1,
            meaning: "Runtime failure".into(),
        },
        ExitCodeSpec {
            code: 2,
            meaning: "Usage / validation error".into(),
        },
        ExitCodeSpec {
            code: 32,
            meaning: "Lock timeout".into(),
        },
    ]
}

fn standard_formats() -> Vec<String> {
    vec!["yaml".into(), "json".into(), "toml".into()]
}

fn opt_target() -> HelpOption {
    HelpOption {
        name: "--target".into(),
        value_name: "PATH".into(),
        default_value: String::new(),
        description: "Path to the target binary file".into(),
    }
}

fn opt_workspace() -> HelpOption {
    HelpOption {
        name: "--workspace".into(),
        value_name: "DIR".into(),
        default_value: ".".into(),
        description: "Path to the workspace directory".into(),
    }
}

fn opt_format() -> HelpOption {
    HelpOption {
        name: "--format".into(),
        value_name: "FMT".into(),
        default_value: "yaml".into(),
        description: "Output format (yaml, json, toml)".into(),
    }
}

fn opt_phase() -> HelpOption {
    HelpOption {
        name: "--phase".into(),
        value_name: "NAME".into(),
        default_value: String::new(),
        description: "Pipeline phase identifier".into(),
    }
}

fn opt_output() -> HelpOption {
    HelpOption {
        name: "--output".into(),
        value_name: "PATH".into(),
        default_value: String::new(),
        description: "Write output to PATH instead of stdout".into(),
    }
}

fn opt_fn_id() -> HelpOption {
    HelpOption {
        name: "--fn-id".into(),
        value_name: "ID".into(),
        default_value: String::new(),
        description: "Function directory identifier (for example fn_001)".into(),
    }
}

fn opt_addr() -> HelpOption {
    HelpOption {
        name: "--addr".into(),
        value_name: "ADDR".into(),
        default_value: String::new(),
        description: "Function address to decompile".into(),
    }
}

fn opt_batch() -> HelpOption {
    HelpOption {
        name: "--batch".into(),
        value_name: String::new(),
        default_value: "false".into(),
        description: "Decompile every entry from decompilation/next-batch.yaml".into(),
    }
}

// ---------------------------------------------------------------------------
// Help registry
// ---------------------------------------------------------------------------

/// Return the structured help document for the given command path, or `None`
/// if no entry exists.
pub fn structured_help(path: &[String]) -> Option<HelpDocument> {
    // Convert to &str slice for pattern matching.
    let path: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
    match path.as_slice() {
        // -----------------------------------------------------------------
        // Top-level container
        // -----------------------------------------------------------------
        [] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into()],
            purpose: DESCRIPTION.into(),
            usage: format!("{SKILL_NAME} <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "validate".into(),
                    summary: "Validate workspace schema and structure".into(),
                },
                HelpSubcommand {
                    name: "workspace".into(),
                    summary: "Workspace lifecycle management".into(),
                },
                HelpSubcommand {
                    name: "scope".into(),
                    summary: "Manage analysis scope entries".into(),
                },
                HelpSubcommand {
                    name: "functions".into(),
                    summary: "Manage function entries".into(),
                },
                HelpSubcommand {
                    name: "callgraph".into(),
                    summary: "Manage call-graph edges".into(),
                },
                HelpSubcommand {
                    name: "types".into(),
                    summary: "Manage type definitions".into(),
                },
                HelpSubcommand {
                    name: "vtables".into(),
                    summary: "Manage vtable definitions".into(),
                },
                HelpSubcommand {
                    name: "constants".into(),
                    summary: "Manage named constants".into(),
                },
                HelpSubcommand {
                    name: "strings".into(),
                    summary: "Manage string entries".into(),
                },
                HelpSubcommand {
                    name: "imports".into(),
                    summary: "Manage import entries".into(),
                },
                HelpSubcommand {
                    name: "third-party".into(),
                    summary: "Manage third-party library metadata".into(),
                },
                HelpSubcommand {
                    name: "runtime".into(),
                    summary: "Record P1 runtime manifests and run records".into(),
                },
                HelpSubcommand {
                    name: "hotpath".into(),
                    summary: "Record P1 runtime hotpath call chains".into(),
                },
                HelpSubcommand {
                    name: "metadata".into(),
                    summary: "Record P3 function metadata enrichment".into(),
                },
                HelpSubcommand {
                    name: "substitute".into(),
                    summary: "Record P4 function substitutions".into(),
                },
                HelpSubcommand {
                    name: "git-check".into(),
                    summary: "Validate artifact git tracking/staging".into(),
                },
                HelpSubcommand {
                    name: "execution-log".into(),
                    summary: "Manage execution log entries".into(),
                },
                HelpSubcommand {
                    name: "progress".into(),
                    summary: "Track decompilation progress".into(),
                },
                HelpSubcommand {
                    name: "gate".into(),
                    summary: "Quality-gate checks".into(),
                },
                HelpSubcommand {
                    name: "ghidra".into(),
                    summary: "Run Ghidra headless operations".into(),
                },
                HelpSubcommand {
                    name: "frida".into(),
                    summary: "Run Frida runtime operations".into(),
                },
                HelpSubcommand {
                    name: "context".into(),
                    summary: "Manage analysis context presets".into(),
                },
                HelpSubcommand {
                    name: "paths".into(),
                    summary: "Resolve and display workspace paths".into(),
                },
                HelpSubcommand {
                    name: "help".into(),
                    summary: "Display help for a command".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                DESCRIPTION.into(),
                String::new(),
                "Use one of the subcommands listed below to interact with a specific".into(),
                "part of the decompilation pipeline.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} workspace init --target /bin/foo"),
                    description: "Create a new workspace for /bin/foo".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} validate"),
                    description: "Validate the current workspace".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} help ghidra"),
                    description: "Show help for the ghidra subcommand group".into(),
                },
            ],
        }),

        // -----------------------------------------------------------------
        // validate
        // -----------------------------------------------------------------
        ["validate"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "validate".into()],
            purpose: "Validate workspace schema, directory structure, and cross-references".into(),
            usage: format!("{SKILL_NAME} validate [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format(), opt_output()],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Walks every YAML file inside the workspace and checks:".into(),
                "- Schema conformance against the bundled JSON schemas".into(),
                "- Cross-reference integrity (e.g. function addresses appear in scope)".into(),
                "- Directory structure matches the expected layout".into(),
                String::new(),
                "Returns a list of violations, or an empty list on success.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} validate"),
                    description: "Validate the workspace in the current directory".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} validate --workspace ./ws --format json"),
                    description: "Validate and emit JSON".into(),
                },
            ],
        }),

        // -----------------------------------------------------------------
        // paths
        // -----------------------------------------------------------------
        ["paths"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "paths".into()],
            purpose: "Resolve and display workspace-relative paths".into(),
            usage: format!("{SKILL_NAME} paths [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Resolves every well-known path inside the workspace and prints".into(),
                "a mapping of logical names to absolute filesystem paths.".into(),
                String::new(),
                "Useful for scripting and debugging workspace layout issues.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} paths"),
                    description: "Show all resolved paths".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} paths --workspace ./ws --format json"),
                    description: "Emit paths as JSON".into(),
                },
            ],
        }),

        // -----------------------------------------------------------------
        // help
        // -----------------------------------------------------------------
        ["help"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "help".into()],
            purpose: "Display help for a command or subcommand".into(),
            usage: format!("{SKILL_NAME} help [COMMAND_PATH...]"),
            arguments: vec!["COMMAND_PATH...".into()],
            options: vec![opt_format()],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Shows structured or plain-text help for the given command path.".into(),
                "When invoked with --format json or --format toml the output is".into(),
                "a machine-readable HelpDocument; otherwise a man-like page is".into(),
                "printed to stdout.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} help"),
                    description: "Show top-level help".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} help ghidra decompile"),
                    description: "Show help for ghidra decompile".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} help --format json"),
                    description: "Emit top-level help as JSON".into(),
                },
            ],
        }),

        // =================================================================
        // workspace container
        // =================================================================
        ["workspace"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "workspace".into()],
            purpose: "Workspace lifecycle management".into(),
            usage: format!("{SKILL_NAME} workspace <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "init".into(),
                    summary: "Initialise a new workspace".into(),
                },
                HelpSubcommand {
                    name: "state".into(),
                    summary: "Inspect or change workspace state".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Manages the workspace directory that holds all YAML artefacts,".into(),
                "Ghidra project files, and decompilation progress.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} workspace init --target /bin/foo"),
                description: "Create workspace for /bin/foo".into(),
            }],
        }),

        ["workspace", "init"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "workspace".into(), "init".into()],
            purpose: "Initialise a new workspace for a target binary".into(),
            usage: format!("{SKILL_NAME} workspace init --target <PATH> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_target(), opt_workspace(), opt_format(), opt_output()],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Creates the workspace directory tree, writes the initial".into(),
                "workspace.yaml manifest, and records the target binary hash.".into(),
                String::new(),
                "If the workspace directory already exists the command fails".into(),
                "with exit code 2 unless the existing workspace is empty.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} workspace init --target /usr/bin/ls"),
                    description: "Create workspace in ./<name>".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} workspace init --target ./foo --workspace ./ws"),
                    description: "Create workspace in ./ws".into(),
                },
            ],
        }),

        // -- workspace state container
        ["workspace", "state"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "workspace".into(), "state".into()],
            purpose: "Inspect or change workspace phase state".into(),
            usage: format!("{SKILL_NAME} workspace state <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "show".into(),
                    summary: "Display current workspace state".into(),
                },
                HelpSubcommand {
                    name: "set-phase".into(),
                    summary: "Advance or set the pipeline phase".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Reads or mutates the workspace.yaml phase field that".into(),
                "tracks the current position in the decompilation pipeline.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} workspace state show"),
                description: "Show current phase".into(),
            }],
        }),

        ["workspace", "state", "show"] => Some(HelpDocument {
            command_path: vec![
                SKILL_NAME.into(),
                "workspace".into(),
                "state".into(),
                "show".into(),
            ],
            purpose: "Display the current workspace phase and metadata".into(),
            usage: format!("{SKILL_NAME} workspace state show [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Prints the workspace.yaml contents including current phase,".into(),
                "target path, and creation timestamp.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} workspace state show"),
                    description: "Show state for the current workspace".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} workspace state show --format json"),
                    description: "Emit state as JSON".into(),
                },
            ],
        }),

        ["workspace", "state", "set-phase"] => Some(HelpDocument {
            command_path: vec![
                SKILL_NAME.into(),
                "workspace".into(),
                "state".into(),
                "set-phase".into(),
            ],
            purpose: "Advance or set the pipeline phase".into(),
            usage: format!("{SKILL_NAME} workspace state set-phase --phase <NAME> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_phase(), opt_format()],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Writes the given phase name into workspace.yaml.".into(),
                "The phase must be a recognised pipeline stage.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} workspace state set-phase --phase baseline"),
                description: "Set phase to baseline".into(),
            }],
        }),

        // =================================================================
        // scope container
        // =================================================================
        ["scope"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "scope".into()],
            purpose: "Manage analysis scope entries".into(),
            usage: format!("{SKILL_NAME} scope <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "show".into(),
                    summary: "Display the current scope".into(),
                },
                HelpSubcommand {
                    name: "set".into(),
                    summary: "Replace the entire scope".into(),
                },
                HelpSubcommand {
                    name: "add-entry".into(),
                    summary: "Add a scope entry".into(),
                },
                HelpSubcommand {
                    name: "remove-entry".into(),
                    summary: "Remove a scope entry".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "The scope defines which address ranges and sections are".into(),
                "included in analysis. Scope entries are stored in scope.yaml.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} scope show"),
                description: "Show the current scope".into(),
            }],
        }),

        // =================================================================
        // functions container
        // =================================================================
        ["functions"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "functions".into()],
            purpose: "Manage function entries".into(),
            usage: format!("{SKILL_NAME} functions <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "add".into(),
                    summary: "Add a function entry".into(),
                },
                HelpSubcommand {
                    name: "rename".into(),
                    summary: "Rename a function".into(),
                },
                HelpSubcommand {
                    name: "set-prototype".into(),
                    summary: "Set a function prototype".into(),
                },
                HelpSubcommand {
                    name: "list".into(),
                    summary: "List all functions".into(),
                },
                HelpSubcommand {
                    name: "show".into(),
                    summary: "Show details for one function".into(),
                },
                HelpSubcommand {
                    name: "remove".into(),
                    summary: "Remove a function entry".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Functions are stored in functions.yaml. Each entry carries".into(),
                "an address, name, prototype, and optional metadata.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} functions list"),
                description: "List all functions".into(),
            }],
        }),

        // =================================================================
        // callgraph container
        // =================================================================
        ["callgraph"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "callgraph".into()],
            purpose: "Manage call-graph edges".into(),
            usage: format!("{SKILL_NAME} callgraph <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "add-edge".into(),
                    summary: "Add a call-graph edge".into(),
                },
                HelpSubcommand {
                    name: "remove-edge".into(),
                    summary: "Remove a call-graph edge".into(),
                },
                HelpSubcommand {
                    name: "list".into(),
                    summary: "List all edges".into(),
                },
                HelpSubcommand {
                    name: "callers".into(),
                    summary: "Show callers of a function".into(),
                },
                HelpSubcommand {
                    name: "callees".into(),
                    summary: "Show callees of a function".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "The call-graph records caller/callee relationships between".into(),
                "functions. Data is stored in callgraph.yaml.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} callgraph list"),
                description: "List all call-graph edges".into(),
            }],
        }),

        // =================================================================
        // types container
        // =================================================================
        ["types"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "types".into()],
            purpose: "Manage type definitions".into(),
            usage: format!("{SKILL_NAME} types <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "add".into(),
                    summary: "Add a type definition".into(),
                },
                HelpSubcommand {
                    name: "remove".into(),
                    summary: "Remove a type definition".into(),
                },
                HelpSubcommand {
                    name: "list".into(),
                    summary: "List all type definitions".into(),
                },
                HelpSubcommand {
                    name: "show".into(),
                    summary: "Show details for one type".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Type definitions (structs, enums, typedefs) are stored in".into(),
                "types.yaml and referenced by function prototypes.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} types list"),
                description: "List all types".into(),
            }],
        }),

        // =================================================================
        // vtables container
        // =================================================================
        ["vtables"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "vtables".into()],
            purpose: "Manage vtable definitions".into(),
            usage: format!("{SKILL_NAME} vtables <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "add".into(),
                    summary: "Add a vtable definition".into(),
                },
                HelpSubcommand {
                    name: "remove".into(),
                    summary: "Remove a vtable definition".into(),
                },
                HelpSubcommand {
                    name: "list".into(),
                    summary: "List all vtables".into(),
                },
                HelpSubcommand {
                    name: "show".into(),
                    summary: "Show details for one vtable".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Vtable definitions map virtual dispatch tables to class names".into(),
                "and their method slots. Stored in vtables.yaml.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} vtables list"),
                description: "List all vtables".into(),
            }],
        }),

        // =================================================================
        // constants container
        // =================================================================
        ["constants"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "constants".into()],
            purpose: "Manage named constants".into(),
            usage: format!("{SKILL_NAME} constants <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "add".into(),
                    summary: "Add a named constant".into(),
                },
                HelpSubcommand {
                    name: "rename".into(),
                    summary: "Rename a constant".into(),
                },
                HelpSubcommand {
                    name: "remove".into(),
                    summary: "Remove a constant".into(),
                },
                HelpSubcommand {
                    name: "list".into(),
                    summary: "List all constants".into(),
                },
                HelpSubcommand {
                    name: "show".into(),
                    summary: "Show details for one constant".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Named constants provide human-readable names for numeric".into(),
                "values encountered during analysis. Stored in constants.yaml.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} constants list"),
                description: "List all constants".into(),
            }],
        }),

        // =================================================================
        // strings container
        // =================================================================
        ["strings"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "strings".into()],
            purpose: "Manage string entries".into(),
            usage: format!("{SKILL_NAME} strings <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "add".into(),
                    summary: "Add a string entry".into(),
                },
                HelpSubcommand {
                    name: "remove".into(),
                    summary: "Remove a string entry".into(),
                },
                HelpSubcommand {
                    name: "list".into(),
                    summary: "List all string entries".into(),
                },
                HelpSubcommand {
                    name: "show".into(),
                    summary: "Show details for one string".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "String entries catalogue interesting strings found in the".into(),
                "binary. Stored in strings.yaml.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} strings list"),
                description: "List all strings".into(),
            }],
        }),

        // =================================================================
        // imports container
        // =================================================================
        ["imports"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "imports".into()],
            purpose: "Manage import entries".into(),
            usage: format!("{SKILL_NAME} imports <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "add".into(),
                    summary: "Add an import entry".into(),
                },
                HelpSubcommand {
                    name: "remove".into(),
                    summary: "Remove an import entry".into(),
                },
                HelpSubcommand {
                    name: "list".into(),
                    summary: "List all imports".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Import entries record external symbols imported by the binary.".into(),
                "Stored in imports.yaml.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} imports list"),
                description: "List all imports".into(),
            }],
        }),

        // =================================================================
        // third-party container
        // =================================================================
        ["third-party"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "third-party".into()],
            purpose: "Manage third-party library metadata".into(),
            usage: format!("{SKILL_NAME} third-party <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "add".into(),
                    summary: "Add a third-party library entry".into(),
                },
                HelpSubcommand {
                    name: "none".into(),
                    summary: "Record an explicit no-third-party review".into(),
                },
                HelpSubcommand {
                    name: "set-version".into(),
                    summary: "Set the version of a library".into(),
                },
                HelpSubcommand {
                    name: "list".into(),
                    summary: "List all third-party libraries".into(),
                },
                HelpSubcommand {
                    name: "classify-function".into(),
                    summary: "Classify a function as third-party".into(),
                },
                HelpSubcommand {
                    name: "vendor-pristine".into(),
                    summary: "Mark library as vendor-pristine".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Tracks third-party libraries detected in the binary, including".into(),
                "version, pristine source status, and function classification.".into(),
                "Use 'none' to record that review found no third-party libraries.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} third-party list"),
                description: "List all third-party libraries".into(),
            }],
        }),

        // =================================================================
        // P0-P4 artifact command containers
        // =================================================================
        ["runtime"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "runtime".into()],
            purpose: "Record P1 runtime manifests and run records".into(),
            usage: format!("{SKILL_NAME} runtime <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "record".into(),
                    summary: "Record a runtime observation and run record".into(),
                },
                HelpSubcommand {
                    name: "validate".into(),
                    summary: "Validate runtime/run-manifest.yaml".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Runtime commands write artifacts/<target>/runtime/run-manifest.yaml".into(),
                "and runtime/run-records/*.yaml for the P1 gate.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} runtime record --key entrypoint --value 0x1000"),
                description: "Record a simple runtime observation".into(),
            }],
        }),
        ["hotpath"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "hotpath".into()],
            purpose: "Record P1 runtime hotpath call chains".into(),
            usage: format!("{SKILL_NAME} hotpath <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "add".into(),
                    summary: "Add a hotpath function entry".into(),
                },
                HelpSubcommand {
                    name: "validate".into(),
                    summary: "Validate runtime/hotpaths/call-chain.yaml".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Hotpath commands write artifacts/<target>/runtime/hotpaths/call-chain.yaml."
                    .into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} hotpath add --addr 0x1000 --reason runtime"),
                description: "Record a hotpath address".into(),
            }],
        }),
        ["metadata"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "metadata".into()],
            purpose: "Record P3 function metadata enrichment".into(),
            usage: format!("{SKILL_NAME} metadata <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "enrich-function".into(),
                    summary: "Record a function name and signature".into(),
                },
                HelpSubcommand {
                    name: "validate".into(),
                    summary: "Validate metadata renames and signatures".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Metadata commands write metadata/renames.yaml, metadata/signatures.yaml,".into(),
                "and metadata/apply-records/*.yaml before CLI-mediated Ghidra apply steps.".into(),
            ],
            examples: vec![HelpExample {
                command: format!(
                    "{SKILL_NAME} metadata enrich-function --addr 0x1000 --name main --prototype 'int(void)'"
                ),
                description: "Record a recovered name and prototype".into(),
            }],
        }),
        ["substitute"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "substitute".into()],
            purpose: "Record P4 function substitutions".into(),
            usage: format!("{SKILL_NAME} substitute <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "add".into(),
                    summary: "Record a function substitution".into(),
                },
                HelpSubcommand {
                    name: "validate".into(),
                    summary: "Validate substitution function records".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Substitute commands write substitution/functions/<fn_id>/substitution.yaml".into(),
                "and substitution/next-batch.yaml for the P4 gate.".into(),
            ],
            examples: vec![HelpExample {
                command: format!(
                    "{SKILL_NAME} substitute add --fn-id fn_001 --addr 0x1000 --replacement 'return 0;'"
                ),
                description: "Record a substitution skeleton".into(),
            }],
        }),
        ["git-check"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "git-check".into()],
            purpose: "Validate artifact git tracking/staging".into(),
            usage: format!("{SKILL_NAME} git-check <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![HelpSubcommand {
                name: "validate".into(),
                summary: "Validate all artifact YAML files are tracked or staged".into(),
            }],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Git-check reports artifact YAML files that are untracked or unstaged".into(),
                "in a git workspace without creating additional artifacts.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} git-check validate"),
                description: "Check artifact git state".into(),
            }],
        }),

        // =================================================================
        // execution-log container
        // =================================================================
        ["execution-log"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "execution-log".into()],
            purpose: "Manage execution log entries".into(),
            usage: format!("{SKILL_NAME} execution-log <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "append".into(),
                    summary: "Append an execution log entry".into(),
                },
                HelpSubcommand {
                    name: "list".into(),
                    summary: "List execution log entries".into(),
                },
                HelpSubcommand {
                    name: "show".into(),
                    summary: "Show details for one entry".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "The execution log records each pipeline step invocation with".into(),
                "timestamp, phase, and outcome. Stored in execution-log.yaml.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} execution-log list"),
                description: "List execution log entries".into(),
            }],
        }),

        // =================================================================
        // progress container
        // =================================================================
        ["progress"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "progress".into()],
            purpose: "Track decompilation progress".into(),
            usage: format!("{SKILL_NAME} progress <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "mark-decompiled".into(),
                    summary: "Mark a function as decompiled".into(),
                },
                HelpSubcommand {
                    name: "compute-next-batch".into(),
                    summary: "Compute the next batch of functions to decompile".into(),
                },
                HelpSubcommand {
                    name: "show".into(),
                    summary: "Show current progress summary".into(),
                },
                HelpSubcommand {
                    name: "list".into(),
                    summary: "List per-function progress entries".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Progress tracking records which functions have been decompiled".into(),
                "and computes the next batch to process. Stored in progress.yaml.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} progress show"),
                description: "Show overall progress".into(),
            }],
        }),

        ["progress", "compute-next-batch"] => Some(HelpDocument {
            command_path: vec![
                SKILL_NAME.into(),
                "progress".into(),
                "compute-next-batch".into(),
            ],
            purpose: "Compute the next batch of functions to decompile".into(),
            usage: format!("{SKILL_NAME} progress compute-next-batch [OPTIONS]"),
            arguments: vec![],
            options: vec![
                opt_workspace(),
                opt_format(),
                HelpOption {
                    name: "--batch-size".into(),
                    value_name: "N".into(),
                    default_value: "10".into(),
                    description: "Maximum number of functions in the batch".into(),
                },
            ],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Analyses dependency order and selects the next set of functions".into(),
                "that are ready for decompilation. Respects the batch-size limit.".into(),
                String::new(),
                "Output is the list of function addresses selected for the batch.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} progress compute-next-batch"),
                    description: "Compute next batch (default size)".into(),
                },
                HelpExample {
                    command: format!(
                        "{SKILL_NAME} progress compute-next-batch --batch-size 25 --format json"
                    ),
                    description: "Compute a batch of 25 as JSON".into(),
                },
            ],
        }),

        // =================================================================
        // gate container
        // =================================================================
        ["gate"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "gate".into()],
            purpose: "Quality-gate checks".into(),
            usage: format!("{SKILL_NAME} gate <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "check".into(),
                    summary: "Run a quality-gate check".into(),
                },
                HelpSubcommand {
                    name: "list".into(),
                    summary: "List all gates and their status".into(),
                },
                HelpSubcommand {
                    name: "show".into(),
                    summary: "Show details for one gate".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Quality gates enforce that decompilation artefacts meet".into(),
                "thresholds before the pipeline advances. Stored in gates.yaml.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} gate list"),
                description: "List all gates".into(),
            }],
        }),

        // =================================================================
        // ghidra container
        // =================================================================
        ["ghidra"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "ghidra".into()],
            purpose: "Run Ghidra headless operations".into(),
            usage: format!("{SKILL_NAME} ghidra <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_target(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "discover".into(),
                    summary: "Discover the Ghidra installation".into(),
                },
                HelpSubcommand {
                    name: "import".into(),
                    summary: "Import the target binary into a Ghidra project".into(),
                },
                HelpSubcommand {
                    name: "auto-analyze".into(),
                    summary: "Run Ghidra auto-analysis".into(),
                },
                HelpSubcommand {
                    name: "export-baseline".into(),
                    summary: "Export the baseline markdown report".into(),
                },
                HelpSubcommand {
                    name: "analyze-vtables".into(),
                    summary: "Analyze likely virtual tables and emit a report".into(),
                },
                HelpSubcommand {
                    name: "apply-renames".into(),
                    summary: "Apply symbol renames to Ghidra".into(),
                },
                HelpSubcommand {
                    name: "verify-renames".into(),
                    summary: "Verify applied renames".into(),
                },
                HelpSubcommand {
                    name: "apply-signatures".into(),
                    summary: "Apply function signatures to Ghidra".into(),
                },
                HelpSubcommand {
                    name: "verify-signatures".into(),
                    summary: "Verify applied signatures".into(),
                },
                HelpSubcommand {
                    name: "import-types-and-signatures".into(),
                    summary: "Import custom C types and function signatures".into(),
                },
                HelpSubcommand {
                    name: "decompile".into(),
                    summary: "Decompile functions via Ghidra".into(),
                },
                HelpSubcommand {
                    name: "rebuild-project".into(),
                    summary: "Rebuild the Ghidra project from scratch".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Wraps the Ghidra headless analyser to perform binary import,".into(),
                "analysis, and decompilation. Each subcommand maps to a single".into(),
                "Ghidra headless invocation.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} ghidra discover"),
                    description: "Locate the Ghidra installation".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} ghidra analyze-vtables --target libfoo"),
                    description: "Write a dedicated vtable analysis report".into(),
                },
            ],
        }),

        ["ghidra", "discover"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "ghidra".into(), "discover".into()],
            purpose: "Discover the Ghidra installation on this host".into(),
            usage: format!("{SKILL_NAME} ghidra discover [OPTIONS]"),
            arguments: vec![],
            options: vec![
                opt_workspace(),
                opt_format(),
                HelpOption {
                    name: "--ghidra-root".into(),
                    value_name: "DIR".into(),
                    default_value: String::new(),
                    description:
                        "Explicit path to the Ghidra install directory (overrides auto-discovery)"
                            .into(),
                },
            ],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Searches well-known locations for a Ghidra installation and".into(),
                "records the path in the workspace manifest. If --ghidra-root is".into(),
                "given the path is used directly without searching.".into(),
                String::new(),
                "The discovered path must contain analyzeHeadless and the Ghidra".into(),
                "application layout.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} ghidra discover"),
                    description: "Auto-discover Ghidra".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} ghidra discover --ghidra-root /opt/ghidra"),
                    description: "Use an explicit Ghidra path".into(),
                },
            ],
        }),

        ["ghidra", "import"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "ghidra".into(), "import".into()],
            purpose: "Import the target binary into a Ghidra project".into(),
            usage: format!("{SKILL_NAME} ghidra import [OPTIONS]"),
            arguments: vec![],
            options: vec![
                opt_target(),
                opt_workspace(),
                opt_format(),
                HelpOption {
                    name: "--project-name".into(),
                    value_name: "NAME".into(),
                    default_value: String::new(),
                    description: "Ghidra project name (defaults to target filename)".into(),
                },
            ],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Runs analyzeHeadless in import mode to create a Ghidra project".into(),
                "from the target binary. The project file is stored inside the".into(),
                "workspace's ghidra/ directory.".into(),
                String::new(),
                "Requires that ghidra discover has already been run.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} ghidra import"),
                    description: "Import target into Ghidra".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} ghidra import --format json"),
                    description: "Import and emit JSON status".into(),
                },
            ],
        }),

        ["ghidra", "apply-signatures"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "ghidra".into(), "apply-signatures".into()],
            purpose: "Apply function signatures to the current Ghidra program".into(),
            usage: format!("{SKILL_NAME} ghidra apply-signatures [OPTIONS]"),
            arguments: vec![],
            options: vec![
                opt_workspace(),
                opt_target(),
                opt_format(),
                HelpOption {
                    name: "--rename-from-signature".into(),
                    value_name: String::new(),
                    default_value: "false".into(),
                    description:
                        "Also rename functions when a signature entry contains a non-placeholder name"
                            .into(),
                },
            ],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Reads artifacts/<target>/metadata/signatures.yaml when present".into(),
                "and applies return types, parameters, and varargs by address.".into(),
                "If that file is absent, falls back to baseline/types.yaml".into(),
                "function entries for compatibility.".into(),
                "The Ghidra headless invocation uses -noanalysis so applying".into(),
                "signatures does not trigger auto-analysis.".into(),
                String::new(),
                "Function names are preserved by default. Use".into(),
                "--rename-from-signature to opt into renaming from full C".into(),
                "prototypes such as int decode(ImportedContext *ctx).".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} ghidra apply-signatures --target libfoo"),
                    description: "Apply recovered prototypes without changing function names".into(),
                },
                HelpExample {
                    command: format!(
                        "{SKILL_NAME} ghidra apply-signatures --target libfoo --rename-from-signature"
                    ),
                    description: "Apply signatures and names from full C prototypes".into(),
                },
            ],
        }),

        ["ghidra", "decompile"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "ghidra".into(), "decompile".into()],
            purpose: "Decompile one or more functions via Ghidra".into(),
            usage: format!("{SKILL_NAME} ghidra decompile [OPTIONS]"),
            arguments: vec![],
            options: vec![
                opt_workspace(),
                opt_target(),
                opt_fn_id(),
                opt_addr(),
                opt_batch(),
                opt_format(),
            ],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Runs the Ghidra headless decompiler for a single function".into(),
                "selected by --fn-id and --addr, or for the current batch when".into(),
                "--batch is given.".into(),
                "The Ghidra headless invocation uses -noanalysis so decompile".into(),
                "does not trigger auto-analysis; run ghidra auto-analyze first.".into(),
                String::new(),
                "Batch mode reads artifacts/<target>/decompilation/next-batch.yaml".into(),
                "and processes entries in order without rewriting that file.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!(
                        "{SKILL_NAME} ghidra decompile --target libfoo --fn-id fn_001 --addr 0x401000"
                    ),
                    description: "Decompile one selected function".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} ghidra decompile --target libfoo --batch"),
                    description: "Decompile the current next-batch worklist".into(),
                },
                HelpExample {
                    command: format!(
                        "{SKILL_NAME} ghidra decompile --target libfoo --fn-id fn_001 --addr 0x401000 --format json"
                    ),
                    description: "Decompile one function with JSON output".into(),
                },
            ],
        }),

        ["ghidra", "import-types-and-signatures"] => Some(HelpDocument {
            command_path: vec![
                SKILL_NAME.into(),
                "ghidra".into(),
                "import-types-and-signatures".into(),
            ],
            purpose: "Import selected C headers and apply function signatures".into(),
            usage: format!(
                "{SKILL_NAME} ghidra import-types-and-signatures --header <PATH> [--header <PATH> ...] [--include-dir <PATH> ...] [--signatures <PATH>] [--program <NAME>]"
            ),
            arguments: vec![],
            options: vec![
                opt_workspace(),
                opt_target(),
                opt_format(),
                HelpOption {
                    name: "--header".into(),
                    value_name: "PATH".into(),
                    default_value: String::new(),
                    description:
                        "Repeat to import one or more header files into the current program".into(),
                },
                HelpOption {
                    name: "--include-dir".into(),
                    value_name: "PATH".into(),
                    default_value: "parent directories of --header values".into(),
                    description: "Repeat to add C parser include search directories".into(),
                },
                HelpOption {
                    name: "--signatures".into(),
                    value_name: "PATH".into(),
                    default_value: "artifacts/<target>/metadata/signatures.yaml when present"
                        .into(),
                    description:
                        "YAML file containing signature entries with prototype or signature fields"
                            .into(),
                },
                HelpOption {
                    name: "--program".into(),
                    value_name: "NAME".into(),
                    default_value: "pipeline-state binary basename".into(),
                    description:
                        "Program name to open with -process before importing types and signatures"
                            .into(),
                },
            ],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Runs the bundled ImportTypesAndSignatures.java helper against".into(),
                "the resolved target project and current program.".into(),
                String::new(),
                "Header parent directories are added to the C parser include".into(),
                "path automatically. Use --include-dir for extra include roots.".into(),
                "If --signatures is omitted and artifacts/<target>/metadata/".into(),
                "signatures.yaml exists, that file is used automatically.".into(),
                "The Ghidra headless invocation uses -noanalysis so importing".into(),
                "types and signatures does not trigger auto-analysis.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!(
                        "{SKILL_NAME} ghidra import-types-and-signatures --target libfoo --header ./include/custom_types.h --header ./include/custom_api.h"
                    ),
                    description: "Import custom headers into the current Ghidra program".into(),
                },
                HelpExample {
                    command: format!(
                        "{SKILL_NAME} ghidra import-types-and-signatures --target libfoo --header ./include/custom_types.h --signatures ./metadata/signatures.yaml --program dummy.bin"
                    ),
                    description: "Override the signature source and program name".into(),
                },
            ],
        }),

        ["ghidra", "analyze-vtables"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "ghidra".into(), "analyze-vtables".into()],
            purpose: "Scan likely vtable regions and emit a scored report".into(),
            usage: format!("{SKILL_NAME} ghidra analyze-vtables [OPTIONS]"),
            arguments: vec![],
            options: vec![
                opt_workspace(),
                opt_target(),
                opt_format(),
                HelpOption {
                    name: "--write-baseline".into(),
                    value_name: String::new(),
                    default_value: "false".into(),
                    description: "Write accepted candidates to baseline/vtables.yaml".into(),
                },
            ],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Runs a dedicated Ghidra script that scans likely read-only".into(),
                "and relro-style sections for contiguous code pointers,".into(),
                "scores each candidate, and records type/class hints.".into(),
                "The Ghidra headless invocation uses -noanalysis so scanning".into(),
                "vtable candidates does not trigger auto-analysis.".into(),
                String::new(),
                "It always writes artifacts/<target>/baseline/".into(),
                "vtable-analysis-report.yaml. Use --write-baseline to also write".into(),
                "artifacts/<target>/baseline/vtables.yaml.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} ghidra analyze-vtables --target libfoo"),
                    description: "Write a scored vtable analysis report".into(),
                },
                HelpExample {
                    command: format!(
                        "{SKILL_NAME} ghidra analyze-vtables --target libfoo --write-baseline"
                    ),
                    description: "Refresh baseline/vtables.yaml from accepted candidates".into(),
                },
            ],
        }),

        // =================================================================
        // frida container
        // =================================================================
        ["frida"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "frida".into()],
            purpose: "Frida runtime operations".into(),
            usage: format!("{SKILL_NAME} frida <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "device-list".into(),
                    summary: "List available Frida devices".into(),
                },
                HelpSubcommand {
                    name: "device-attach".into(),
                    summary: "Prepare a device or process selector".into(),
                },
                HelpSubcommand {
                    name: "io-capture".into(),
                    summary: "Capture function I/O with Frida".into(),
                },
                HelpSubcommand {
                    name: "signature-analysis".into(),
                    summary: "Analyze runtime function signatures".into(),
                },
                HelpSubcommand {
                    name: "call-tree-trace".into(),
                    summary: "Trace call trees from a target".into(),
                },
                HelpSubcommand {
                    name: "dispatch-vtable-trace".into(),
                    summary: "Trace dispatcher or vtable ranges".into(),
                },
                HelpSubcommand {
                    name: "hotpath-coverage".into(),
                    summary: "Record hot path coverage".into(),
                },
                HelpSubcommand {
                    name: "io-compare".into(),
                    summary: "Compare original and reconstructed I/O".into(),
                },
                HelpSubcommand {
                    name: "decomp-compare".into(),
                    summary: "Compare decompilation output against runtime traces".into(),
                },
                HelpSubcommand {
                    name: "fuzz-input-gen".into(),
                    summary: "Generate fuzz inputs from type metadata".into(),
                },
                HelpSubcommand {
                    name: "run".into(),
                    summary: "Run a target under Frida control".into(),
                },
                HelpSubcommand {
                    name: "trace".into(),
                    summary: "Trace selected functions under Frida".into(),
                },
                HelpSubcommand {
                    name: "invoke".into(),
                    summary: "Invoke a function through Frida".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Frida commands manage runtime capture, tracing, invocation,".into(),
                "and comparison helpers used by verification workflows.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} frida device-list"),
                description: "List available Frida devices".into(),
            }],
        }),

        // =================================================================
        // context container
        // =================================================================
        ["context"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "context".into()],
            purpose: "Manage analysis context presets".into(),
            usage: format!("{SKILL_NAME} context <COMMAND> [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![
                HelpSubcommand {
                    name: "show".into(),
                    summary: "Show the current context".into(),
                },
                HelpSubcommand {
                    name: "use".into(),
                    summary: "Switch to a named context preset".into(),
                },
                HelpSubcommand {
                    name: "clear".into(),
                    summary: "Clear the active context".into(),
                },
            ],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Context presets control which analysis features are active.".into(),
                "They allow switching between, for example, a full-analysis mode".into(),
                "and a quick-scan mode without modifying the workspace.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} context show"),
                description: "Show active context".into(),
            }],
        }),

        ["context", "show"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "context".into(), "show".into()],
            purpose: "Show the current analysis context".into(),
            usage: format!("{SKILL_NAME} context show [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Displays the active context preset including all overridden".into(),
                "options and the effective configuration.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} context show"),
                    description: "Show current context".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} context show --format json"),
                    description: "Show context as JSON".into(),
                },
            ],
        }),

        ["context", "use"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "context".into(), "use".into()],
            purpose: "Switch to a named context preset".into(),
            usage: format!("{SKILL_NAME} context use <NAME> [OPTIONS]"),
            arguments: vec!["NAME".into()],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Activates the named context preset. The preset name must refer".into(),
                "to a built-in or user-defined context.".into(),
            ],
            examples: vec![
                HelpExample {
                    command: format!("{SKILL_NAME} context use full-analysis"),
                    description: "Switch to full-analysis context".into(),
                },
                HelpExample {
                    command: format!("{SKILL_NAME} context use quick-scan"),
                    description: "Switch to quick-scan context".into(),
                },
            ],
        }),

        ["context", "clear"] => Some(HelpDocument {
            command_path: vec![SKILL_NAME.into(), "context".into(), "clear".into()],
            purpose: "Clear the active context preset".into(),
            usage: format!("{SKILL_NAME} context clear [OPTIONS]"),
            arguments: vec![],
            options: vec![opt_workspace(), opt_format()],
            subcommands: vec![],
            output_formats: standard_formats(),
            exit_behavior: standard_exit_codes(),
            description: vec![
                "Removes any active context preset, reverting to the default".into(),
                "analysis configuration.".into(),
            ],
            examples: vec![HelpExample {
                command: format!("{SKILL_NAME} context clear"),
                description: "Clear active context".into(),
            }],
        }),

        // -----------------------------------------------------------------
        // Anything else => no entry
        // -----------------------------------------------------------------
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Plain-text (man-like) help
// ---------------------------------------------------------------------------

/// Return a man-like plain-text help string for the given command path, or
/// `None` if no entry exists.
pub fn plain_text_help(path: &[String]) -> Option<String> {
    let doc = structured_help(path)?;
    Some(render_man_page(&doc))
}

fn render_man_page(doc: &HelpDocument) -> String {
    let mut out = String::new();

    // NAME
    let cmd_name = doc.command_path.join(" ");
    out.push_str(&format!("NAME\n    {cmd_name} - {}\n\n", doc.purpose));

    // SYNOPSIS
    out.push_str(&format!("SYNOPSIS\n    {}\n\n", doc.usage));

    // DESCRIPTION
    if !doc.description.is_empty() {
        out.push_str("DESCRIPTION\n");
        for line in &doc.description {
            if line.is_empty() {
                out.push('\n');
            } else {
                out.push_str(&format!("    {line}\n"));
            }
        }
        out.push('\n');
    }

    // OPTIONS
    if !doc.options.is_empty() {
        out.push_str("OPTIONS\n");
        for opt in &doc.options {
            let mut line = format!("    {}", opt.name);
            if !opt.value_name.is_empty() {
                line.push_str(&format!(" <{}>", opt.value_name));
            }
            if !opt.default_value.is_empty() {
                line.push_str(&format!("  [default: {}]", opt.default_value));
            }
            out.push_str(&format!("{line}\n"));
            out.push_str(&format!("        {}\n", opt.description));
        }
        out.push('\n');
    }

    // SUBCOMMANDS
    if !doc.subcommands.is_empty() {
        out.push_str("SUBCOMMANDS\n");
        for sub in &doc.subcommands {
            out.push_str(&format!("    {:<28} {}\n", sub.name, sub.summary));
        }
        out.push('\n');
    }

    // ARGUMENTS (positional)
    if !doc.arguments.is_empty() {
        out.push_str("ARGUMENTS\n");
        for arg in &doc.arguments {
            out.push_str(&format!("    <{arg}>\n"));
        }
        out.push('\n');
    }

    // FORMATS
    if !doc.output_formats.is_empty() {
        out.push_str(&format!(
            "FORMATS\n    Supported output formats: {}\n\n",
            doc.output_formats.join(", ")
        ));
    }

    // EXAMPLES
    if !doc.examples.is_empty() {
        out.push_str("EXAMPLES\n");
        for ex in &doc.examples {
            out.push_str(&format!("    {}\n", ex.command));
            out.push_str(&format!("        {}\n", ex.description));
        }
        out.push('\n');
    }

    // EXIT CODES
    if !doc.exit_behavior.is_empty() {
        out.push_str("EXIT CODES\n");
        for ec in &doc.exit_behavior {
            out.push_str(&format!("    {:<4} {}\n", ec.code, ec.meaning));
        }
        out.push('\n');
    }

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_level_help_exists() {
        let doc = structured_help(&[]).expect("top-level help should exist");
        assert!(!doc.subcommands.is_empty());
        assert!(doc.subcommands.iter().any(|s| s.name == "workspace"));
        assert!(doc.subcommands.iter().any(|s| s.name == "ghidra"));
        assert!(doc.subcommands.iter().any(|s| s.name == "help"));
    }

    #[test]
    fn leaf_validate_help() {
        let doc = structured_help(&["validate".into()]).expect("validate help");
        assert!(doc.subcommands.is_empty());
        assert!(doc.options.iter().any(|o| o.name == "--workspace"));
    }

    #[test]
    fn workspace_container_help() {
        let doc = structured_help(&["workspace".into()]).expect("workspace help");
        assert!(doc.subcommands.iter().any(|s| s.name == "init"));
        assert!(doc.subcommands.iter().any(|s| s.name == "state"));
    }

    #[test]
    fn workspace_init_help() {
        let doc =
            structured_help(&["workspace".into(), "init".into()]).expect("workspace init help");
        assert!(doc.options.iter().any(|o| o.name == "--target"));
    }

    #[test]
    fn workspace_state_container() {
        let doc =
            structured_help(&["workspace".into(), "state".into()]).expect("workspace state help");
        assert!(doc.subcommands.iter().any(|s| s.name == "show"));
        assert!(doc.subcommands.iter().any(|s| s.name == "set-phase"));
    }

    #[test]
    fn workspace_state_show() {
        let doc = structured_help(&["workspace".into(), "state".into(), "show".into()])
            .expect("workspace state show help");
        assert!(doc.subcommands.is_empty());
    }

    #[test]
    fn workspace_state_set_phase() {
        let doc = structured_help(&["workspace".into(), "state".into(), "set-phase".into()])
            .expect("workspace state set-phase help");
        assert!(doc.options.iter().any(|o| o.name == "--phase"));
    }

    #[test]
    fn context_show() {
        let doc = structured_help(&["context".into(), "show".into()]).expect("context show help");
        assert!(doc.subcommands.is_empty());
    }

    #[test]
    fn context_use() {
        let doc = structured_help(&["context".into(), "use".into()]).expect("context use help");
        assert!(doc.arguments.contains(&"NAME".to_string()));
    }

    #[test]
    fn context_clear() {
        let doc = structured_help(&["context".into(), "clear".into()]).expect("context clear help");
        assert!(doc.subcommands.is_empty());
    }

    #[test]
    fn ghidra_container() {
        let doc = structured_help(&["ghidra".into()]).expect("ghidra help");
        assert!(doc.subcommands.iter().any(|s| s.name == "discover"));
        assert!(doc.subcommands.iter().any(|s| s.name == "import"));
        assert!(
            doc.subcommands
                .iter()
                .any(|s| s.name == "import-types-and-signatures")
        );
        assert!(doc.subcommands.iter().any(|s| s.name == "decompile"));
    }

    #[test]
    fn ghidra_discover() {
        let doc =
            structured_help(&["ghidra".into(), "discover".into()]).expect("ghidra discover help");
        assert!(doc.options.iter().any(|o| o.name == "--ghidra-root"));
    }

    #[test]
    fn ghidra_import() {
        let doc = structured_help(&["ghidra".into(), "import".into()]).expect("ghidra import help");
        assert!(doc.options.iter().any(|o| o.name == "--target"));
    }

    #[test]
    fn ghidra_decompile() {
        let doc =
            structured_help(&["ghidra".into(), "decompile".into()]).expect("ghidra decompile help");
        assert!(doc.options.iter().any(|o| o.name == "--fn-id"));
        assert!(doc.options.iter().any(|o| o.name == "--addr"));
        assert!(doc.options.iter().any(|o| o.name == "--batch"));
    }

    #[test]
    fn ghidra_apply_signatures() {
        let doc = structured_help(&["ghidra".into(), "apply-signatures".into()])
            .expect("ghidra apply-signatures help");
        assert!(doc.options.iter().any(|o| o.name == "--target"));
        assert!(
            doc.options
                .iter()
                .any(|o| o.name == "--rename-from-signature")
        );
    }

    #[test]
    fn ghidra_import_types_and_signatures() {
        let doc = structured_help(&["ghidra".into(), "import-types-and-signatures".into()])
            .expect("ghidra import-types-and-signatures help");
        assert!(doc.options.iter().any(|o| o.name == "--header"));
        assert!(doc.options.iter().any(|o| o.name == "--include-dir"));
        assert!(doc.options.iter().any(|o| o.name == "--signatures"));
        assert!(doc.options.iter().any(|o| o.name == "--program"));
    }

    #[test]
    fn ghidra_analyze_vtables() {
        let doc = structured_help(&["ghidra".into(), "analyze-vtables".into()])
            .expect("ghidra analyze-vtables help");
        assert!(doc.options.iter().any(|o| o.name == "--write-baseline"));
        assert!(doc.options.iter().any(|o| o.name == "--target"));
    }

    #[test]
    fn progress_compute_next_batch() {
        let doc = structured_help(&["progress".into(), "compute-next-batch".into()])
            .expect("progress compute-next-batch help");
        assert!(doc.options.iter().any(|o| o.name == "--batch-size"));
    }

    #[test]
    fn all_container_subcommands() {
        let containers = [
            ("scope", vec!["show", "set", "add-entry", "remove-entry"]),
            (
                "functions",
                vec!["add", "rename", "set-prototype", "list", "show", "remove"],
            ),
            (
                "callgraph",
                vec!["add-edge", "remove-edge", "list", "callers", "callees"],
            ),
            ("types", vec!["add", "remove", "list", "show"]),
            ("vtables", vec!["add", "remove", "list", "show"]),
            ("constants", vec!["add", "rename", "remove", "list", "show"]),
            ("strings", vec!["add", "remove", "list", "show"]),
            ("imports", vec!["add", "remove", "list"]),
            (
                "third-party",
                vec![
                    "add",
                    "none",
                    "set-version",
                    "list",
                    "classify-function",
                    "vendor-pristine",
                ],
            ),
            ("runtime", vec!["record", "validate"]),
            ("hotpath", vec!["add", "validate"]),
            ("metadata", vec!["enrich-function", "validate"]),
            ("substitute", vec!["add", "validate"]),
            ("git-check", vec!["validate"]),
            ("execution-log", vec!["append", "list", "show"]),
            (
                "progress",
                vec!["mark-decompiled", "compute-next-batch", "show", "list"],
            ),
            ("gate", vec!["check", "list", "show"]),
            (
                "ghidra",
                vec![
                    "discover",
                    "import",
                    "auto-analyze",
                    "export-baseline",
                    "analyze-vtables",
                    "apply-renames",
                    "verify-renames",
                    "apply-signatures",
                    "verify-signatures",
                    "import-types-and-signatures",
                    "decompile",
                    "rebuild-project",
                ],
            ),
            (
                "frida",
                vec![
                    "device-list",
                    "device-attach",
                    "io-capture",
                    "signature-analysis",
                    "call-tree-trace",
                    "dispatch-vtable-trace",
                    "hotpath-coverage",
                    "io-compare",
                    "decomp-compare",
                    "fuzz-input-gen",
                    "run",
                    "trace",
                    "invoke",
                ],
            ),
            ("context", vec!["show", "use", "clear"]),
        ];
        for (name, expected) in &containers {
            let path_str = name.to_string();
            let doc = structured_help(&[path_str])
                .unwrap_or_else(|| panic!("container '{name}' should have help"));
            for sub in expected {
                assert!(
                    doc.subcommands.iter().any(|s| s.name == *sub),
                    "container '{name}' missing subcommand '{sub}'"
                );
            }
        }
    }

    #[test]
    fn top_level_lists_all_subcommands() {
        let doc = structured_help(&[]).unwrap();
        let expected = [
            "validate",
            "workspace",
            "scope",
            "functions",
            "callgraph",
            "types",
            "vtables",
            "constants",
            "strings",
            "imports",
            "third-party",
            "runtime",
            "hotpath",
            "metadata",
            "substitute",
            "git-check",
            "execution-log",
            "progress",
            "gate",
            "ghidra",
            "frida",
            "context",
            "paths",
            "help",
        ];
        for name in &expected {
            assert!(
                doc.subcommands.iter().any(|s| s.name == *name),
                "top level missing subcommand '{name}'"
            );
        }
    }

    #[test]
    fn standard_exit_codes_present() {
        let doc = structured_help(&[]).unwrap();
        let codes: Vec<i32> = doc.exit_behavior.iter().map(|e| e.code).collect();
        assert!(codes.contains(&0));
        assert!(codes.contains(&1));
        assert!(codes.contains(&2));
        assert!(codes.contains(&32));
    }

    #[test]
    fn standard_output_formats() {
        let doc = structured_help(&[]).unwrap();
        assert_eq!(doc.output_formats, vec!["yaml", "json", "toml"]);
    }

    #[test]
    fn unknown_path_returns_none() {
        assert!(structured_help(&["nonexistent".into()]).is_none());
        assert!(structured_help(&["workspace".into(), "nonexistent".into()]).is_none());
    }

    #[test]
    fn plain_text_renders_sections() {
        let text = plain_text_help(&[]).expect("top-level plain text help");
        assert!(text.contains("NAME"));
        assert!(text.contains("SYNOPSIS"));
        assert!(text.contains("DESCRIPTION"));
        assert!(text.contains("OPTIONS"));
        assert!(text.contains("FORMATS"));
        assert!(text.contains("EXAMPLES"));
        assert!(text.contains("EXIT CODES"));
        assert!(text.contains("SUBCOMMANDS"));
    }

    #[test]
    fn plain_text_leaf_renders() {
        let text = plain_text_help(&["validate".into()]).expect("validate plain text");
        assert!(text.contains("NAME"));
        assert!(text.contains("SYNOPSIS"));
        assert!(!text.contains("SUBCOMMANDS")); // leaf has none
    }

    #[test]
    fn plain_text_unknown_returns_none() {
        assert!(plain_text_help(&["nope".into()]).is_none());
    }
}
