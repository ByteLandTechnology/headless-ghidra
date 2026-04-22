use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use clap::{Args, Parser, Subcommand, ValueEnum};

use ghidra_agent_cli::{
    Format, StructuredError, baseline, context, execution_log, gate, ghidra, lock, ok_output,
    ok_output_with_data, paths, progress, scope, serialize_value, third_party, workspace,
    write_structured_error,
};

// ---------------------------------------------------------------------------
// Exit codes
// ---------------------------------------------------------------------------
const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;
#[allow(dead_code)]
const EXIT_USAGE: i32 = 2;
const EXIT_LOCK_TIMEOUT: i32 = 32;

// ---------------------------------------------------------------------------
// Phase enum (shared across gate / workspace-state / progress)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum Phase {
    #[value(name = "P0")]
    P0,
    #[value(name = "P0.5")]
    P0_5,
    #[value(name = "P1")]
    P1,
    #[value(name = "P2")]
    P2,
    #[value(name = "P3")]
    P3,
    #[value(name = "P4")]
    P4,
    #[value(name = "P5")]
    P5,
    #[value(name = "P6")]
    P6,
}

impl Phase {
    fn as_str(&self) -> &'static str {
        match self {
            Phase::P0 => "P0",
            Phase::P0_5 => "P0.5",
            Phase::P1 => "P1",
            Phase::P2 => "P2",
            Phase::P3 => "P3",
            Phase::P4 => "P4",
            Phase::P5 => "P5",
            Phase::P6 => "P6",
        }
    }
}

use serde::Serialize;

// ---------------------------------------------------------------------------
// Output format
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FormatCli {
    Yaml,
    Json,
    Toml,
}

impl From<FormatCli> for Format {
    fn from(f: FormatCli) -> Self {
        match f {
            FormatCli::Yaml => Format::Yaml,
            FormatCli::Json => Format::Json,
            FormatCli::Toml => Format::Toml,
        }
    }
}

// ---------------------------------------------------------------------------
// Top-level CLI
// ---------------------------------------------------------------------------
#[derive(Parser, Debug)]
#[command(
    name = "ghidra-agent-cli",
    disable_help_flag = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Output format: yaml, json, or toml
    #[arg(long, global = true, default_value = "yaml", value_enum)]
    pub format: FormatCli,

    /// Show help
    #[arg(long, global = true)]
    pub help: bool,

    /// Override config directory
    #[arg(long, global = true)]
    pub config_dir: Option<PathBuf>,

    /// Override data directory
    #[arg(long, global = true)]
    pub data_dir: Option<PathBuf>,

    /// Override state directory
    #[arg(long, global = true)]
    pub state_dir: Option<PathBuf>,

    /// Override cache directory
    #[arg(long, global = true)]
    pub cache_dir: Option<PathBuf>,

    /// Override log directory
    #[arg(long, global = true)]
    pub log_dir: Option<PathBuf>,

    /// Lock acquisition timeout in seconds
    #[arg(long, global = true, default_value_t = 30)]
    pub lock_timeout: u64,

    /// Do not wait for lock acquisition
    #[arg(long, global = true)]
    pub no_wait: bool,

    /// Target selector
    #[arg(long, global = true)]
    pub target: Option<String>,

    /// Workspace root path
    #[arg(long, global = true)]
    pub workspace: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

// ---------------------------------------------------------------------------
// Command tree
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum Commands {
    Validate(ValidateArgs),
    #[command(subcommand)]
    Workspace(WorkspaceCmd),
    #[command(subcommand)]
    Scope(ScopeCmd),
    #[command(subcommand)]
    Functions(FunctionsCmd),
    #[command(subcommand)]
    Callgraph(CallgraphCmd),
    #[command(subcommand)]
    Types(TypesCmd),
    #[command(subcommand)]
    Vtables(VtablesCmd),
    #[command(subcommand)]
    Constants(ConstantsCmd),
    #[command(subcommand)]
    Strings(StringsCmd),
    #[command(subcommand)]
    Imports(ImportsCmd),
    #[command(subcommand)]
    ThirdParty(ThirdPartyCmd),
    #[command(subcommand)]
    ExecutionLog(ExecutionLogCmd),
    #[command(subcommand)]
    Progress(ProgressCmd),
    #[command(subcommand)]
    Gate(GateCmd),
    #[command(subcommand)]
    Ghidra(GhidraCmd),
    #[command(subcommand)]
    Frida(FridaCmd),
    #[command(subcommand)]
    Inspect(InspectCmd),
    #[command(subcommand)]
    Context(ContextCmd),
    Paths(PathsArgs),
    Help(HelpArgs),
}

// ---------------------------------------------------------------------------
// Leaf: validate
// ---------------------------------------------------------------------------
#[derive(Args, Debug)]
pub struct ValidateArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub workspace: Option<PathBuf>,
    #[arg(long)]
    pub file: Option<PathBuf>,
    #[arg(long)]
    pub schema: Option<String>,
    #[arg(long)]
    pub strict: bool,
}

// ---------------------------------------------------------------------------
// Leaf: paths
// ---------------------------------------------------------------------------
#[derive(Args, Debug)]
pub struct PathsArgs {}

// ---------------------------------------------------------------------------
// Leaf: help
// ---------------------------------------------------------------------------
#[derive(Args, Debug)]
pub struct HelpArgs {
    /// Command path to get help for
    #[arg(trailing_var_arg = true)]
    pub command_path: Vec<String>,
}

// ---------------------------------------------------------------------------
// Container: workspace -> { init, state }  where state -> { show, set-phase }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum WorkspaceCmd {
    Init(WorkspaceInitArgs),
    #[command(subcommand)]
    State(WorkspaceStateCmd),
}

#[derive(Args, Debug)]
pub struct WorkspaceInitArgs {
    #[arg(long)]
    pub target: String,
    #[arg(long)]
    pub binary: PathBuf,
    #[arg(long)]
    pub force: bool,
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceStateCmd {
    Show(WorkspaceStateShowArgs),
    SetPhase(WorkspaceStateSetPhaseArgs),
}

#[derive(Args, Debug)]
pub struct WorkspaceStateShowArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct WorkspaceStateSetPhaseArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long, value_enum)]
    pub phase: Phase,
}

// ---------------------------------------------------------------------------
// Container: scope -> { show, set, add-entry, remove-entry }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum ScopeCmd {
    Show(ScopeShowArgs),
    Set(ScopeSetArgs),
    AddEntry(ScopeAddEntryArgs),
    RemoveEntry(ScopeRemoveEntryArgs),
}

#[derive(Args, Debug)]
pub struct ScopeShowArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct ScopeSetArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub mode: String,
    #[arg(long)]
    pub entries: Vec<String>,
    #[arg(long)]
    pub note: Option<String>,
}

#[derive(Args, Debug)]
pub struct ScopeAddEntryArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub entry: String,
}

#[derive(Args, Debug)]
pub struct ScopeRemoveEntryArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub entry: String,
}

// ---------------------------------------------------------------------------
// Container: functions -> { add, rename, set-prototype, list, show, remove }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum FunctionsCmd {
    Add(FunctionsAddArgs),
    Rename(FunctionsRenameArgs),
    SetPrototype(FunctionsSetPrototypeArgs),
    List(FunctionsListArgs),
    Show(FunctionsShowArgs),
    Remove(FunctionsRemoveArgs),
}

#[derive(Args, Debug)]
pub struct FunctionsAddArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub prototype: Option<String>,
    #[arg(long)]
    pub size: Option<u64>,
    #[arg(long)]
    pub section: Option<String>,
    #[arg(long)]
    pub source: Option<String>,
}

#[derive(Args, Debug)]
pub struct FunctionsRenameArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
    #[arg(long)]
    pub new_name: String,
}

#[derive(Args, Debug)]
pub struct FunctionsSetPrototypeArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
    #[arg(long)]
    pub prototype: String,
}

#[derive(Args, Debug)]
pub struct FunctionsListArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub named_only: bool,
    #[arg(long)]
    pub section: Option<String>,
    #[arg(long)]
    pub source: Option<String>,
}

#[derive(Args, Debug)]
pub struct FunctionsShowArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
}

#[derive(Args, Debug)]
pub struct FunctionsRemoveArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
}

// ---------------------------------------------------------------------------
// Container: callgraph -> { add-edge, remove-edge, list, callers, callees }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum CallgraphCmd {
    AddEdge(CallgraphAddEdgeArgs),
    RemoveEdge(CallgraphRemoveEdgeArgs),
    List(CallgraphListArgs),
    Callers(CallgraphCallersArgs),
    Callees(CallgraphCalleesArgs),
}

#[derive(Args, Debug)]
pub struct CallgraphAddEdgeArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub from: String,
    #[arg(long)]
    pub to: String,
    #[arg(long)]
    pub kind: Option<String>,
}

#[derive(Args, Debug)]
pub struct CallgraphRemoveEdgeArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub from: String,
    #[arg(long)]
    pub to: String,
}

#[derive(Args, Debug)]
pub struct CallgraphListArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct CallgraphCallersArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
    #[arg(long)]
    pub transitive: bool,
}

#[derive(Args, Debug)]
pub struct CallgraphCalleesArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
    #[arg(long)]
    pub transitive: bool,
}

// ---------------------------------------------------------------------------
// Container: types -> { add, remove, list, show }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum TypesCmd {
    Add(TypesAddArgs),
    Remove(TypesRemoveArgs),
    List(TypesListArgs),
    Show(TypesShowArgs),
}

#[derive(Args, Debug)]
pub struct TypesAddArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub kind: String,
    #[arg(long)]
    pub definition: String,
}

#[derive(Args, Debug)]
pub struct TypesRemoveArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub name: String,
}

#[derive(Args, Debug)]
pub struct TypesListArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct TypesShowArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub name: String,
}

// ---------------------------------------------------------------------------
// Container: vtables -> { add, remove, list, show }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum VtablesCmd {
    Add(VtablesAddArgs),
    Remove(VtablesRemoveArgs),
    List(VtablesListArgs),
    Show(VtablesShowArgs),
}

#[derive(Args, Debug)]
pub struct VtablesAddArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub class_name: String,
    #[arg(long)]
    pub addr: String,
    #[arg(long)]
    pub entries: Vec<String>,
}

#[derive(Args, Debug)]
pub struct VtablesRemoveArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub class_name: String,
}

#[derive(Args, Debug)]
pub struct VtablesListArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct VtablesShowArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub class_name: String,
}

// ---------------------------------------------------------------------------
// Container: constants -> { add, rename, remove, list, show }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum ConstantsCmd {
    Add(ConstantsAddArgs),
    Rename(ConstantsRenameArgs),
    Remove(ConstantsRemoveArgs),
    List(ConstantsListArgs),
    Show(ConstantsShowArgs),
}

#[derive(Args, Debug)]
pub struct ConstantsAddArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub ctype: Option<String>,
    #[arg(long)]
    pub value: Option<String>,
}

#[derive(Args, Debug)]
pub struct ConstantsRenameArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
    #[arg(long)]
    pub new_name: String,
}

#[derive(Args, Debug)]
pub struct ConstantsRemoveArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
}

#[derive(Args, Debug)]
pub struct ConstantsListArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct ConstantsShowArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
}

// ---------------------------------------------------------------------------
// Container: strings -> { add, remove, list, show }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum StringsCmd {
    Add(StringsAddArgs),
    Remove(StringsRemoveArgs),
    List(StringsListArgs),
    Show(StringsShowArgs),
}

#[derive(Args, Debug)]
pub struct StringsAddArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
    #[arg(long)]
    pub content: String,
    #[arg(long)]
    pub encoding: Option<String>,
}

#[derive(Args, Debug)]
pub struct StringsRemoveArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
}

#[derive(Args, Debug)]
pub struct StringsListArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct StringsShowArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
}

// ---------------------------------------------------------------------------
// Container: imports -> { add, remove, list }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum ImportsCmd {
    Add(ImportsAddArgs),
    Remove(ImportsRemoveArgs),
    List(ImportsListArgs),
}

#[derive(Args, Debug)]
pub struct ImportsAddArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub library: String,
    #[arg(long)]
    pub symbol: String,
    #[arg(long)]
    pub plt_addr: Option<String>,
}

#[derive(Args, Debug)]
pub struct ImportsRemoveArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub library: String,
    #[arg(long)]
    pub symbol: String,
}

#[derive(Args, Debug)]
pub struct ImportsListArgs {
    #[arg(long)]
    pub target: Option<String>,
}

// ---------------------------------------------------------------------------
// Container: third-party -> { add, set-version, list, classify-function, vendor-pristine }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum ThirdPartyCmd {
    Add(ThirdPartyAddArgs),
    SetVersion(ThirdPartySetVersionArgs),
    List(ThirdPartyListArgs),
    ClassifyFunction(ThirdPartyClassifyFunctionArgs),
    VendorPristine(ThirdPartyVendorPristineArgs),
}

#[derive(Args, Debug)]
pub struct ThirdPartyAddArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub library: String,
    #[arg(long)]
    pub version: String,
    #[arg(long)]
    pub confidence: Option<String>,
    #[arg(long)]
    pub evidence: Option<String>,
    #[arg(long)]
    pub upstream_url: Option<String>,
}

#[derive(Args, Debug)]
pub struct ThirdPartySetVersionArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub library: String,
    #[arg(long)]
    pub version: String,
}

#[derive(Args, Debug)]
pub struct ThirdPartyListArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct ThirdPartyClassifyFunctionArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub addr: String,
    #[arg(long)]
    pub classification: String,
    #[arg(long)]
    pub evidence: Option<String>,
}

#[derive(Args, Debug)]
pub struct ThirdPartyVendorPristineArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub library: String,
    /// Path to the pristine upstream source tree
    #[arg(long)]
    pub source_path: std::path::PathBuf,
    /// Commit the vendored source to git
    #[arg(long)]
    pub commit: bool,
}

// ---------------------------------------------------------------------------
// Container: execution-log -> { append, list, show }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum ExecutionLogCmd {
    Append(ExecutionLogAppendArgs),
    List(ExecutionLogListArgs),
    Show(ExecutionLogShowArgs),
}

#[derive(Args, Debug)]
pub struct ExecutionLogAppendArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub script: String,
    #[arg(long)]
    pub status: String,
    #[arg(long)]
    pub inputs_hash: Option<String>,
    #[arg(long)]
    pub outputs: Option<Vec<String>>,
    #[arg(long)]
    pub duration_ms: Option<u64>,
}

#[derive(Args, Debug)]
pub struct ExecutionLogListArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct ExecutionLogShowArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub script: String,
}

// ---------------------------------------------------------------------------
// Container: progress -> { mark-decompiled, compute-next-batch, show, list }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum ProgressCmd {
    MarkDecompiled(ProgressMarkDecompiledArgs),
    ComputeNextBatch(ProgressComputeNextBatchArgs),
    Show(ProgressShowArgs),
    List(ProgressListArgs),
}

#[derive(Args, Debug)]
pub struct ProgressMarkDecompiledArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub fn_id: String,
    #[arg(long)]
    pub addr: String,
    #[arg(long)]
    pub backend: Option<String>,
}

#[derive(Args, Debug)]
pub struct ProgressComputeNextBatchArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long, default_value_t = 8)]
    pub max: usize,
    #[arg(long)]
    pub strategy: Option<String>,
}

#[derive(Args, Debug)]
pub struct ProgressShowArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct ProgressListArgs {
    #[arg(long)]
    pub target: Option<String>,
}

// ---------------------------------------------------------------------------
// Container: gate -> { check, list, show }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum GateCmd {
    Check(GateCheckArgs),
    List(GateListArgs),
    Show(GateShowArgs),
}

#[derive(Args, Debug)]
pub struct GateCheckArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long, value_enum)]
    pub phase: Option<GatePhase>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum GatePhase {
    #[value(name = "P0")]
    P0,
    #[value(name = "P0.5")]
    P0_5,
    #[value(name = "P1")]
    P1,
    #[value(name = "P2")]
    P2,
    #[value(name = "P3")]
    P3,
    #[value(name = "P4")]
    P4,
    #[value(name = "P5")]
    P5,
    #[value(name = "P6")]
    P6,
    All,
}

impl GatePhase {
    fn as_str(&self) -> &'static str {
        match self {
            GatePhase::P0 => "P0",
            GatePhase::P0_5 => "P0.5",
            GatePhase::P1 => "P1",
            GatePhase::P2 => "P2",
            GatePhase::P3 => "P3",
            GatePhase::P4 => "P4",
            GatePhase::P5 => "P5",
            GatePhase::P6 => "P6",
            GatePhase::All => "all",
        }
    }
}

#[derive(Args, Debug)]
pub struct GateListArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct GateShowArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub phase: Option<String>,
}

// ---------------------------------------------------------------------------
// Container: ghidra -> { discover, import, auto-analyze, export-baseline,
//                        analyze-vtables, apply-renames, verify-renames,
//                        apply-signatures, verify-signatures, decompile,
//                        rebuild-project }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum GhidraCmd {
    Discover(GhidraDiscoverArgs),
    Import(GhidraImportArgs),
    AutoAnalyze(GhidraAutoAnalyzeArgs),
    ExportBaseline(GhidraExportBaselineArgs),
    AnalyzeVtables(GhidraAnalyzeVtablesArgs),
    ApplyRenames(GhidraApplyRenamesArgs),
    VerifyRenames(GhidraVerifyRenamesArgs),
    ApplySignatures(GhidraApplySignaturesArgs),
    VerifySignatures(GhidraVerifySignaturesArgs),
    Decompile(GhidraDecompileArgs),
    RebuildProject(GhidraRebuildProjectArgs),
}

#[derive(Args, Debug)]
pub struct GhidraDiscoverArgs {
    #[arg(long)]
    pub install_dir: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct GhidraImportArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct GhidraAutoAnalyzeArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct GhidraExportBaselineArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct GhidraAnalyzeVtablesArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long, default_value_t = 4)]
    pub min_entries: u32,
    #[arg(long, default_value_t = 20)]
    pub max_entries: u32,
    #[arg(long, default_value_t = 64)]
    pub scan_limit: u32,
    #[arg(long, default_value = "rodata,const,data.rel.ro,.data")]
    pub segments: String,
    #[arg(long, default_value_t = 4)]
    pub min_score: i32,
    #[arg(long)]
    pub write_baseline: bool,
    #[arg(long)]
    pub overwrite: bool,
    #[arg(long)]
    pub report_path: Option<String>,
}

#[derive(Args, Debug)]
pub struct GhidraApplyRenamesArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct GhidraVerifyRenamesArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct GhidraApplySignaturesArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct GhidraVerifySignaturesArgs {
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug)]
pub struct GhidraDecompileArgs {
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub fn_id: Option<String>,
    #[arg(long)]
    pub addr: Option<String>,
    #[arg(long)]
    pub batch: bool,
}

#[derive(Args, Debug)]
pub struct GhidraRebuildProjectArgs {
    #[arg(long)]
    pub target: Option<String>,
}

// ---------------------------------------------------------------------------
// Container: frida -> { device, io-capture, signature-analysis, ... }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum FridaCmd {
    DeviceList,
    DeviceAttach(DeviceAttachArgs),
    IoCapture(IoCaptureArgs),
    SignatureAnalysis(SignatureAnalysisArgs),
    CallTreeTrace(CallTreeTraceArgs),
    DispatchVtableTrace(DispatchVtableTraceArgs),
    HotpathCoverage(HotpathCoverageArgs),
    IoCompare(IoCompareArgs),
    DecompCompare(DecompCompareArgs),
    FuzzInputGen(FuzzInputGenArgs),
    Run(FridaRunArgs),
    Trace(FridaTraceArgs),
    Invoke(FridaInvokeArgs),
}

// ---------------------------------------------------------------------------
// Container: inspect -> { binary info }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum InspectCmd {
    Binary(InspectBinaryArgs),
}

#[derive(Args, Debug)]
pub struct IoCaptureArgs {
    #[arg(long)]
    pub target: String,
    #[arg(long, default_value = "60")]
    pub timeout: u64,
    #[arg(long)]
    pub device: Option<String>,
    #[arg(long)]
    pub spawn: Option<String>,
    /// Arguments to pass to the spawned process (after --)
    #[arg(last = true)]
    pub proc_args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct SignatureAnalysisArgs {
    /// Target executable or process
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub funcs: String,
    #[arg(long)]
    pub device: Option<String>,
    /// Arguments to pass to the spawned process (after --)
    #[arg(last = true)]
    pub proc_args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct CallTreeTraceArgs {
    /// Target executable or process
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub max_depth: Option<u32>,
    #[arg(long)]
    pub libs: Option<String>,
    #[arg(long)]
    pub device: Option<String>,
    /// Arguments to pass to the spawned process (after --)
    #[arg(last = true)]
    pub proc_args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct DispatchVtableTraceArgs {
    #[arg(long)]
    pub ranges: String,
    #[arg(long)]
    pub device: Option<String>,
    #[arg(long)]
    pub spawn: Option<String>,
}

#[derive(Args, Debug)]
pub struct HotpathCoverageArgs {
    /// Target executable or process
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub threshold: Option<u32>,
    #[arg(long)]
    pub interval: Option<u32>,
    #[arg(long)]
    pub device: Option<String>,
    /// Arguments to pass to the spawned process (after --)
    #[arg(last = true)]
    pub proc_args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct IoCompareArgs {
    #[arg(long)]
    pub original: String,
    #[arg(long)]
    pub reconstructed: String,
}

#[derive(Args, Debug)]
pub struct DecompCompareArgs {
    /// Target executable or process
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub func: String,
    #[arg(long)]
    pub log: Option<String>,
    /// Arguments to pass to the spawned process (after --)
    #[arg(last = true)]
    pub proc_args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct FuzzInputGenArgs {
    #[arg(long)]
    pub types_yaml: String,
    #[arg(long)]
    pub output: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeviceAttachArgs {
    #[arg(long)]
    pub pid: Option<u32>,
    #[arg(long)]
    pub device: Option<String>,
}

#[derive(Args, Debug)]
pub struct FridaRunArgs {
    /// Target executable
    #[arg(long)]
    pub target: String,

    /// Arguments to pass to the executable
    #[arg(long)]
    pub args: Option<String>,

    /// Standard input file
    #[arg(long)]
    pub stdin: Option<String>,

    /// Timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Device selector (local, usb, etc.)
    #[arg(long)]
    pub device: Option<String>,
}

#[derive(Args, Debug)]
pub struct FridaTraceArgs {
    /// Target executable or process
    #[arg(long)]
    pub target: String,

    /// Comma-separated list of functions to trace
    #[arg(long)]
    pub functions: Option<String>,

    /// Timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Device selector
    #[arg(long)]
    pub device: Option<String>,

    /// Arguments to pass to the spawned process (after --)
    #[arg(last = true)]
    pub proc_args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct FridaInvokeArgs {
    /// Target library or executable
    #[arg(long)]
    pub target: String,

    /// Function name to invoke
    #[arg(long)]
    pub function: String,

    /// Function signature (e.g., "int(int, int)")
    #[arg(long)]
    pub signature: Option<String>,

    /// Arguments as JSON array
    #[arg(long)]
    pub args: Option<String>,

    /// Device selector
    #[arg(long)]
    pub device: Option<String>,
}

#[derive(Args, Debug)]
pub struct InspectBinaryArgs {
    /// Binary file path (.a, .dylib, .so, executable)
    #[arg(long)]
    pub target: String,

    /// Show exports (for dynamic libraries)
    #[arg(long, default_value = "true")]
    pub exports: bool,

    /// Show imports
    #[arg(long, default_value = "true")]
    pub imports: bool,

    /// Show object files (for static libraries)
    #[arg(long, default_value = "true")]
    pub objects: bool,
}

// ---------------------------------------------------------------------------
// Container: context -> { show, use, clear }
// ---------------------------------------------------------------------------
#[derive(Subcommand, Debug)]
pub enum ContextCmd {
    Show,
    Use(ContextUseArgs),
    Clear,
}

#[derive(Args, Debug)]
pub struct ContextUseArgs {
    #[arg(long = "selector")]
    pub selectors: Vec<String>,
}

// ===========================================================================
// Plain-text man-like help
// ===========================================================================
#[allow(clippy::useless_format)]
fn render_man_help() -> String {
    format!(
        r#"GHIDRA-AGENT-CLI(1)           ghidra-agent-cli Manual          GHIDRA-AGENT-CLI(1)

NAME
    ghidra-agent-cli - Drive the headless-Ghidra decompilation pipeline

SYNOPSIS
    ghidra-agent-cli [GLOBAL FLAGS] <COMMAND> [COMMAND FLAGS]

DESCRIPTION
    ghidra-agent-cli drives the headless-Ghidra decompilation pipeline end-to-end
    through schema-validated YAML workspaces, scripted Ghidra runs, and gated
    incremental reconstruction builds.

GLOBAL FLAGS
    --format <FORMAT>          Output format: yaml (default), json, toml
    --help                     Show this help text
    --config-dir <PATH>        Override config directory
    --data-dir <PATH>          Override data directory
    --state-dir <PATH>         Override state directory
    --cache-dir <PATH>         Override cache directory
    --log-dir <PATH>           Override log directory
    --lock-timeout <SECS>      Lock acquisition timeout in seconds (default: 30)
    --no-wait                  Do not wait for lock acquisition
    --target <TARGET>          Target selector
    --workspace <PATH>         Workspace root path

COMMANDS
    validate          Validate workspace schema and gates
    workspace         Workspace management (init, state)
    scope             Scope management (show, set, add-entry, remove-entry)
    functions         Function baseline management
    callgraph         Callgraph edge management
    types             Type baseline management
    vtables           Vtable baseline management
    constants         Constant baseline management
    strings           String baseline management
    imports           Import baseline management
    third-party       Third-party library management
    execution-log     Execution log management
    progress          Decompilation progress tracking
    gate              Pipeline gate checks
    ghidra            Ghidra toolchain operations
    frida             Frida runtime operations
    context           Active context management (show, use, clear)
    paths             Show resolved runtime directory paths
    help              Show help for a command path

EXIT CODES
    0     Success
    1     Failure
    2     Usage error
    32    Lock timeout

EXAMPLES
    ghidra-agent-cli workspace init --target libfoo --binary ./libfoo.so
    ghidra-agent-cli --target libfoo gate check --phase P1
    ghidra-agent-cli --target libfoo functions list
    ghidra-agent-cli --target libfoo ghidra decompile --fn-id fn_001 --addr 0x401000
    ghidra-agent-cli paths
    ghidra-agent-cli help gate check
"#
    )
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Resolve the workspace root: subcommand flag > global flag > auto-detect.
fn resolve_workspace(
    explicit_subcmd: Option<&PathBuf>,
    global: Option<&PathBuf>,
) -> Result<PathBuf> {
    if let Some(p) = explicit_subcmd {
        return Ok(p.clone());
    }
    if let Some(p) = global {
        return Ok(p.clone());
    }
    workspace::detect_workspace(None)
}

/// Resolve the target: subcommand flag > global flag > error.
fn resolve_target(
    subcmd_target: Option<&String>,
    global_target: Option<&String>,
) -> Result<String> {
    if let Some(t) = subcmd_target {
        return Ok(t.clone());
    }
    if let Some(t) = global_target {
        return Ok(t.clone());
    }
    Err(anyhow!(
        "no target specified: use --target or set active context"
    ))
}

/// Build RuntimeOverrides from global flags.
fn make_runtime_overrides(cli: &Cli) -> context::RuntimeOverrides {
    context::RuntimeOverrides {
        config_dir: cli.config_dir.clone(),
        data_dir: cli.data_dir.clone(),
        state_dir: cli.state_dir.clone(),
        cache_dir: cli.cache_dir.clone(),
        log_dir: cli.log_dir.clone(),
    }
}

/// Check for lock timeout error and return the correct exit code.
fn exit_code_for_error(err: &anyhow::Error) -> i32 {
    let msg = format!("{err:#}");
    if msg.contains("E_LOCK_TIMEOUT") {
        EXIT_LOCK_TIMEOUT
    } else {
        EXIT_FAILURE
    }
}

// ===========================================================================
// Dispatch: top-level commands
// ===========================================================================

fn validate_baseline_doc(
    doc: &serde_yaml::Value,
    schema_name: &str,
    collection_key: &str,
    required_entry_fields: &[&str],
    checks: &mut Vec<gate::GateCheck>,
) {
    let has_target = doc.get("target").and_then(|v| v.as_str()).is_some();
    checks.push(gate::GateCheck {
        id: format!("schema_{schema_name}_target"),
        description: format!("{schema_name}.yaml has 'target' string"),
        passed: has_target,
        detail: if has_target {
            None
        } else {
            Some("missing 'target' field".into())
        },
    });

    let has_collection = doc
        .get(collection_key)
        .and_then(|v| v.as_sequence())
        .is_some();
    checks.push(gate::GateCheck {
        id: format!("schema_{schema_name}_{collection_key}"),
        description: format!("{schema_name}.yaml has '{collection_key}' sequence"),
        passed: has_collection,
        detail: if has_collection {
            None
        } else {
            Some(format!("missing '{collection_key}' sequence"))
        },
    });

    if let Some(entries) = doc.get(collection_key).and_then(|v| v.as_sequence()) {
        let missing: Vec<String> = entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                required_entry_fields
                    .iter()
                    .any(|f| entry.get(*f).is_none())
            })
            .map(|(i, _)| format!("{collection_key}[{i}]"))
            .collect();
        checks.push(gate::GateCheck {
            id: format!("schema_{schema_name}_required_fields"),
            description: format!(
                "all {schema_name} entries have required fields ({})",
                required_entry_fields.join(", ")
            ),
            passed: missing.is_empty(),
            detail: if missing.is_empty() {
                None
            } else {
                Some(format!("missing fields in: {}", missing.join(", ")))
            },
        });
    }
}

fn exec_validate(cli: &Cli, args: &ValidateArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(args.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut all_passed = true;
    let mut reports = Vec::new();

    // Phase gate checks
    for phase in gate::ALL_PHASES {
        let report = gate::check_phase(&ws, &target, phase)?;
        if !report.passed {
            all_passed = false;
        }
        reports.push(report);
    }

    // Optional schema validation of specific file
    if let Some(schema_name) = &args.schema {
        let ad = workspace::artifact_dir(&ws, &target);
        let file_path = args
            .file
            .as_ref()
            .cloned()
            .unwrap_or_else(|| match schema_name.as_str() {
                "functions" => ad.join("baseline").join("functions.yaml"),
                "callgraph" => ad.join("baseline").join("callgraph.yaml"),
                "types" => ad.join("baseline").join("types.yaml"),
                "vtables" => ad.join("baseline").join("vtables.yaml"),
                "vtable-analysis" => ad.join("baseline").join("vtable-analysis-report.yaml"),
                "constants" => ad.join("baseline").join("constants.yaml"),
                "strings" => ad.join("baseline").join("strings.yaml"),
                "imports" => ad.join("baseline").join("imports.yaml"),
                "scope" => ad.join("scope.yaml"),
                "pipeline-state" => ad.join("pipeline-state.yaml"),
                "target-selection" => ad.join("target-selection.yaml"),
                "progress" => ad.join("decompilation").join("progress.yaml"),
                "identified" => ad.join("third-party").join("identified.yaml"),
                _ => ad.join(format!("{schema_name}.yaml")),
            });

        if !file_path.exists() {
            reports.push(gate::GateReport {
                target: target.clone(),
                phase: format!("schema:{schema_name}"),
                passed: false,
                checks: vec![gate::GateCheck {
                    id: format!("schema_{schema_name}"),
                    description: format!("schema file {} exists", file_path.display()),
                    passed: false,
                    detail: Some(format!("file not found: {}", file_path.display())),
                }],
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
            all_passed = false;
        } else {
            let raw = std::fs::read_to_string(&file_path)
                .map_err(|e| anyhow!("failed to read {}: {e}", file_path.display()))?;
            let parse_result: Result<serde_yaml::Value, _> = serde_yaml::from_str(&raw);

            match parse_result {
                Ok(doc) => {
                    let mut schema_checks = Vec::new();

                    // Validate top-level structure for known schemas
                    match schema_name.as_str() {
                        "functions" => {
                            let has_functions =
                                doc.get("functions").and_then(|v| v.as_sequence()).is_some();
                            let has_target = doc.get("target").and_then(|v| v.as_str()).is_some();
                            schema_checks.push(gate::GateCheck {
                                id: "schema_functions_top_level".into(),
                                description: "functions.yaml has 'functions' sequence".into(),
                                passed: has_functions,
                                detail: if has_functions {
                                    None
                                } else {
                                    Some("missing 'functions' sequence field".into())
                                },
                            });
                            schema_checks.push(gate::GateCheck {
                                id: "schema_functions_target".into(),
                                description: "functions.yaml has 'target' string".into(),
                                passed: has_target,
                                detail: if has_target {
                                    None
                                } else {
                                    Some("missing 'target' field".into())
                                },
                            });
                            // Validate each function entry has required fields
                            if let Some(fns) = doc.get("functions").and_then(|v| v.as_sequence()) {
                                let missing: Vec<String> = fns
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, f)| f.get("addr").is_none())
                                    .map(|(i, _)| format!("functions[{i}]"))
                                    .collect();
                                schema_checks.push(gate::GateCheck {
                                    id: "schema_functions_addr".into(),
                                    description: "all function entries have 'addr'".into(),
                                    passed: missing.is_empty(),
                                    detail: if missing.is_empty() {
                                        None
                                    } else {
                                        Some(format!("missing addr in: {}", missing.join(", ")))
                                    },
                                });
                            }
                            if args.strict
                                && let Some(fns) =
                                    doc.get("functions").and_then(|v| v.as_sequence())
                            {
                                let missing_name: Vec<String> = fns
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, f)| f.get("name").is_none())
                                    .map(|(i, _)| format!("functions[{i}]"))
                                    .collect();
                                schema_checks.push(gate::GateCheck {
                                    id: "schema_functions_name_strict".into(),
                                    description: "[strict] all function entries have 'name'".into(),
                                    passed: missing_name.is_empty(),
                                    detail: if missing_name.is_empty() {
                                        None
                                    } else {
                                        Some(format!(
                                            "missing name in: {}",
                                            missing_name.join(", ")
                                        ))
                                    },
                                });
                            }
                        }
                        "scope" => {
                            let has_entries = doc
                                .get("entries")
                                .and_then(|v| v.as_sequence())
                                .map(|s| !s.is_empty())
                                .unwrap_or(false);
                            schema_checks.push(gate::GateCheck {
                                id: "schema_scope_entries".into(),
                                description: "scope.yaml has non-empty 'entries'".into(),
                                passed: has_entries,
                                detail: if has_entries {
                                    None
                                } else {
                                    Some("missing or empty 'entries'".into())
                                },
                            });
                        }
                        "pipeline-state" => {
                            for field in &["target", "phase"] {
                                let present = doc.get(*field).is_some();
                                schema_checks.push(gate::GateCheck {
                                    id: format!("schema_pipeline_state_{field}"),
                                    description: format!("pipeline-state.yaml has '{field}'"),
                                    passed: present,
                                    detail: if present {
                                        None
                                    } else {
                                        Some(format!("missing '{field}'"))
                                    },
                                });
                            }
                        }
                        "target-selection" => {
                            let has_selected = doc.get("selected_target").is_some();
                            let has_candidates = doc
                                .get("candidates")
                                .and_then(|v| v.as_sequence())
                                .is_some();
                            schema_checks.push(gate::GateCheck {
                                id: "schema_target_selection_selected".into(),
                                description: "target-selection.yaml has 'selected_target'".into(),
                                passed: has_selected,
                                detail: if has_selected {
                                    None
                                } else {
                                    Some("missing 'selected_target'".into())
                                },
                            });
                            schema_checks.push(gate::GateCheck {
                                id: "schema_target_selection_candidates".into(),
                                description: "target-selection.yaml has 'candidates' sequence"
                                    .into(),
                                passed: has_candidates,
                                detail: if has_candidates {
                                    None
                                } else {
                                    Some("missing 'candidates' sequence".into())
                                },
                            });
                        }
                        "callgraph" => {
                            validate_baseline_doc(
                                &doc,
                                "callgraph",
                                "edges",
                                &["from", "to"],
                                &mut schema_checks,
                            );
                        }
                        "types" => {
                            validate_baseline_doc(
                                &doc,
                                "types",
                                "types",
                                &["name", "kind", "definition"],
                                &mut schema_checks,
                            );
                        }
                        "vtables" => {
                            validate_baseline_doc(
                                &doc,
                                "vtables",
                                "vtables",
                                &["class", "addr", "entries"],
                                &mut schema_checks,
                            );
                        }
                        "vtable-analysis" => {
                            validate_baseline_doc(
                                &doc,
                                "vtable-analysis",
                                "candidates",
                                &["addr", "status", "score", "entries"],
                                &mut schema_checks,
                            );
                            schema_checks.push(gate::GateCheck {
                                id: "schema_vtable_analysis_pointer_size".into(),
                                description: "vtable-analysis has 'pointer_size'".into(),
                                passed: doc.get("pointer_size").is_some(),
                                detail: if doc.get("pointer_size").is_some() {
                                    None
                                } else {
                                    Some("missing 'pointer_size'".into())
                                },
                            });
                        }
                        "constants" => {
                            validate_baseline_doc(
                                &doc,
                                "constants",
                                "constants",
                                &["addr"],
                                &mut schema_checks,
                            );
                        }
                        "strings" => {
                            validate_baseline_doc(
                                &doc,
                                "strings",
                                "strings",
                                &["addr", "content"],
                                &mut schema_checks,
                            );
                        }
                        "imports" => {
                            validate_baseline_doc(
                                &doc,
                                "imports",
                                "imports",
                                &["library", "symbol"],
                                &mut schema_checks,
                            );
                        }
                        _ => {
                            // Generic: just verify it's valid YAML
                            schema_checks.push(gate::GateCheck {
                                id: format!("schema_{schema_name}_parseable"),
                                description: format!("{schema_name} is valid YAML"),
                                passed: true,
                                detail: None,
                            });
                        }
                    }

                    let schema_passed = schema_checks.iter().all(|c| c.passed);
                    if !schema_passed {
                        all_passed = false;
                    }
                    reports.push(gate::GateReport {
                        target: target.clone(),
                        phase: format!("schema:{schema_name}"),
                        passed: schema_passed,
                        checks: schema_checks,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                }
                Err(e) => {
                    all_passed = false;
                    reports.push(gate::GateReport {
                        target: target.clone(),
                        phase: format!("schema:{schema_name}"),
                        passed: false,
                        checks: vec![gate::GateCheck {
                            id: format!("schema_{schema_name}_parse"),
                            description: format!("{schema_name} is valid YAML"),
                            passed: false,
                            detail: Some(format!("YAML parse error: {e}")),
                        }],
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                }
            }
        }
    }

    let msg = if all_passed {
        "all gates passed"
    } else {
        "some gates failed"
    };
    let out = ok_output_with_data(msg, serde_yaml::to_value(&reports)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(if all_passed {
        EXIT_SUCCESS
    } else {
        EXIT_FAILURE
    })
}

fn exec_paths(cli: &Cli, fmt: Format) -> Result<i32> {
    let overrides = paths::RuntimeOverrides {
        config_dir: cli.config_dir.clone(),
        data_dir: cli.data_dir.clone(),
        state_dir: cli.state_dir.clone(),
        cache_dir: cli.cache_dir.clone(),
        log_dir: cli.log_dir.clone(),
    };
    let summary = paths::resolve_runtime_locations(&overrides, false)?;
    let out = ok_output_with_data("runtime paths", serde_yaml::to_value(&summary)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_help(args: &HelpArgs, fmt: Format) -> Result<i32> {
    if args.command_path.is_empty() {
        let text = render_man_help();
        let data = serde_yaml::Value::String(text);
        let out = ok_output_with_data("help", data);
        serialize_value(&mut std::io::stdout(), &out, fmt)?;
    } else {
        let path = args.command_path.join(" ");
        // For a specific command path, return structured help metadata
        let mut data = serde_yaml::Mapping::new();
        data.insert(
            serde_yaml::Value::String("command".into()),
            serde_yaml::Value::String(path.clone()),
        );
        data.insert(
            serde_yaml::Value::String("description".into()),
            serde_yaml::Value::String(format!("help for ghidra-agent-cli {}", path)),
        );
        let out = ok_output_with_data("help", serde_yaml::Value::Mapping(data));
        serialize_value(&mut std::io::stdout(), &out, fmt)?;
    }
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: workspace
// ===========================================================================

fn exec_workspace_init(cli: &Cli, args: &WorkspaceInitArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;

    if args.force {
        let td = workspace::target_dir(&ws, &args.target);
        if td.exists() {
            std::fs::remove_dir_all(&td)?;
        }
    }

    workspace::init_target(&ws, &args.target, &args.binary)?;
    let out = ok_output(&format!("target '{}' initialized", args.target));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_workspace_state_show(cli: &Cli, args: &WorkspaceStateShowArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let state = workspace::load_pipeline_state(&ws, &target)?;
    let out = ok_output_with_data("pipeline state", serde_yaml::to_value(&state)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_workspace_state_set_phase(
    cli: &Cli,
    args: &WorkspaceStateSetPhaseArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    workspace::set_phase(&ws, &target, args.phase.as_str())?;
    let out = ok_output(&format!("phase set to {}", args.phase.as_str()));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: scope
// ===========================================================================

fn exec_scope_show(cli: &Cli, args: &ScopeShowArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let scope_data = scope::load_scope(&ws, &target)?;
    let out = ok_output_with_data("scope", serde_yaml::to_value(&scope_data)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_scope_set(cli: &Cli, args: &ScopeSetArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    scope::set_scope(
        &ws,
        &target,
        &args.mode,
        args.entries.clone(),
        args.note.clone(),
    )?;
    let out = ok_output("scope updated");
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_scope_add_entry(cli: &Cli, args: &ScopeAddEntryArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    scope::add_entry(&ws, &target, &args.entry)?;
    let out = ok_output(&format!("entry '{}' added to scope", args.entry));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_scope_remove_entry(cli: &Cli, args: &ScopeRemoveEntryArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    scope::remove_entry(&ws, &target, &args.entry)?;
    let out = ok_output(&format!("entry '{}' removed from scope", args.entry));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: functions
// ===========================================================================

fn exec_functions_add(cli: &Cli, args: &FunctionsAddArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut functions = match baseline::load_functions(&ws, &target) {
        Ok(f) => f,
        Err(_) => baseline::FunctionsYaml {
            target: target.clone(),
            functions: vec![],
        },
    };

    let entry = baseline::FunctionEntry {
        addr: args.addr.clone(),
        name: args.name.clone(),
        prototype: args.prototype.clone(),
        size: args.size.unwrap_or(0),
        section: args.section.clone(),
        source: args.source.clone().unwrap_or_else(|| "manual".to_string()),
    };
    functions.functions.push(entry);
    baseline::save_functions(&ws, &target, &functions)?;
    let out = ok_output(&format!("function added at {}", args.addr));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_functions_rename(cli: &Cli, args: &FunctionsRenameArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut functions = baseline::load_functions(&ws, &target)?;
    let entry = functions
        .functions
        .iter_mut()
        .find(|f| f.addr == args.addr)
        .ok_or_else(|| anyhow!("function at {} not found", args.addr))?;
    entry.name = Some(args.new_name.clone());
    baseline::save_functions(&ws, &target, &functions)?;
    let out = ok_output(&format!(
        "function at {} renamed to {}",
        args.addr, args.new_name
    ));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_functions_set_prototype(
    cli: &Cli,
    args: &FunctionsSetPrototypeArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut functions = baseline::load_functions(&ws, &target)?;
    let entry = functions
        .functions
        .iter_mut()
        .find(|f| f.addr == args.addr)
        .ok_or_else(|| anyhow!("function at {} not found", args.addr))?;
    entry.prototype = Some(args.prototype.clone());
    baseline::save_functions(&ws, &target, &functions)?;
    let out = ok_output(&format!("prototype set for function at {}", args.addr));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_functions_list(cli: &Cli, args: &FunctionsListArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let functions = baseline::load_functions(&ws, &target)?;

    let filtered: Vec<&baseline::FunctionEntry> = functions
        .functions
        .iter()
        .filter(|f| {
            if args.named_only && f.name.is_none() {
                return false;
            }
            if let Some(ref sec) = args.section
                && f.section.as_deref() != Some(sec.as_str())
            {
                return false;
            }
            if let Some(ref src) = args.source
                && f.source != *src
            {
                return false;
            }
            true
        })
        .collect();

    let out = ok_output_with_data(
        &format!("{} functions", filtered.len()),
        serde_yaml::to_value(&filtered)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_functions_show(cli: &Cli, args: &FunctionsShowArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let functions = baseline::load_functions(&ws, &target)?;
    let entry = functions
        .functions
        .iter()
        .find(|f| f.addr == args.addr)
        .ok_or_else(|| anyhow!("function at {} not found", args.addr))?;
    let out = ok_output_with_data("function", serde_yaml::to_value(entry)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_functions_remove(cli: &Cli, args: &FunctionsRemoveArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut functions = baseline::load_functions(&ws, &target)?;
    let before = functions.functions.len();
    functions.functions.retain(|f| f.addr != args.addr);
    if functions.functions.len() == before {
        return Err(anyhow!("function at {} not found", args.addr));
    }
    baseline::save_functions(&ws, &target, &functions)?;
    let out = ok_output(&format!("function at {} removed", args.addr));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: callgraph
// ===========================================================================

fn exec_callgraph_add_edge(cli: &Cli, args: &CallgraphAddEdgeArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut cg = match baseline::load_callgraph(&ws, &target) {
        Ok(c) => c,
        Err(_) => baseline::CallgraphYaml {
            target: target.clone(),
            edges: vec![],
        },
    };
    cg.edges.push(baseline::CallEdge {
        from: args.from.clone(),
        to: args.to.clone(),
        kind: args.kind.clone().unwrap_or_else(|| "direct".to_string()),
    });
    baseline::save_callgraph(&ws, &target, &cg)?;
    let out = ok_output(&format!("edge {} -> {} added", args.from, args.to));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_callgraph_remove_edge(
    cli: &Cli,
    args: &CallgraphRemoveEdgeArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut cg = baseline::load_callgraph(&ws, &target)?;
    let before = cg.edges.len();
    cg.edges
        .retain(|e| !(e.from == args.from && e.to == args.to));
    if cg.edges.len() == before {
        return Err(anyhow!("edge {} -> {} not found", args.from, args.to));
    }
    baseline::save_callgraph(&ws, &target, &cg)?;
    let out = ok_output(&format!("edge {} -> {} removed", args.from, args.to));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_callgraph_list(cli: &Cli, args: &CallgraphListArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let cg = baseline::load_callgraph(&ws, &target)?;
    let out = ok_output_with_data(
        &format!("{} edges", cg.edges.len()),
        serde_yaml::to_value(&cg.edges)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_callgraph_callers(cli: &Cli, args: &CallgraphCallersArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let cg = baseline::load_callgraph(&ws, &target)?;

    let mut callers = Vec::new();
    let mut queue = vec![args.addr.clone()];
    let mut visited = std::collections::HashSet::new();

    while let Some(addr) = queue.pop() {
        if !visited.insert(addr.clone()) {
            continue;
        }
        for edge in &cg.edges {
            if edge.to == addr && !visited.contains(&edge.from) {
                callers.push(edge.from.clone());
                if args.transitive {
                    queue.push(edge.from.clone());
                }
            }
        }
        if !args.transitive {
            break;
        }
    }

    let out = ok_output_with_data(
        &format!("{} callers", callers.len()),
        serde_yaml::to_value(&callers)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_callgraph_callees(cli: &Cli, args: &CallgraphCalleesArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let cg = baseline::load_callgraph(&ws, &target)?;

    let mut callees = Vec::new();
    let mut queue = vec![args.addr.clone()];
    let mut visited = std::collections::HashSet::new();

    while let Some(addr) = queue.pop() {
        if !visited.insert(addr.clone()) {
            continue;
        }
        for edge in &cg.edges {
            if edge.from == addr && !visited.contains(&edge.to) {
                callees.push(edge.to.clone());
                if args.transitive {
                    queue.push(edge.to.clone());
                }
            }
        }
        if !args.transitive {
            break;
        }
    }

    let out = ok_output_with_data(
        &format!("{} callees", callees.len()),
        serde_yaml::to_value(&callees)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: types
// ===========================================================================

fn exec_types_add(cli: &Cli, args: &TypesAddArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut types = match baseline::load_types(&ws, &target) {
        Ok(t) => t,
        Err(_) => baseline::TypesYaml {
            target: target.clone(),
            types: vec![],
        },
    };
    types.types.push(baseline::TypeEntry {
        name: args.name.clone(),
        kind: args.kind.clone(),
        definition: args.definition.clone(),
    });
    baseline::save_types(&ws, &target, &types)?;
    let out = ok_output(&format!("type '{}' added", args.name));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_types_remove(cli: &Cli, args: &TypesRemoveArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut types = baseline::load_types(&ws, &target)?;
    let before = types.types.len();
    types.types.retain(|t| t.name != args.name);
    if types.types.len() == before {
        return Err(anyhow!("type '{}' not found", args.name));
    }
    baseline::save_types(&ws, &target, &types)?;
    let out = ok_output(&format!("type '{}' removed", args.name));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_types_list(cli: &Cli, args: &TypesListArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let types = baseline::load_types(&ws, &target)?;
    let out = ok_output_with_data(
        &format!("{} types", types.types.len()),
        serde_yaml::to_value(&types.types)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_types_show(cli: &Cli, args: &TypesShowArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let types = baseline::load_types(&ws, &target)?;
    let entry = types
        .types
        .iter()
        .find(|t| t.name == args.name)
        .ok_or_else(|| anyhow!("type '{}' not found", args.name))?;
    let out = ok_output_with_data("type", serde_yaml::to_value(entry)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: vtables
// ===========================================================================

fn exec_vtables_add(cli: &Cli, args: &VtablesAddArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut vtables = match baseline::load_vtables(&ws, &target) {
        Ok(v) => v,
        Err(_) => baseline::VtablesYaml {
            target: target.clone(),
            vtables: vec![],
        },
    };
    vtables.vtables.push(baseline::VtableEntry {
        class: args.class_name.clone(),
        addr: args.addr.clone(),
        entries: args.entries.clone(),
        entry_count: Some(args.entries.len()),
        confidence: None,
        score: None,
        source: Some("manual".to_string()),
        segment: None,
        associated_type: None,
        association_evidence: None,
        signature_summary: None,
    });
    baseline::save_vtables(&ws, &target, &vtables)?;
    let out = ok_output(&format!("vtable for '{}' added", args.class_name));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_vtables_remove(cli: &Cli, args: &VtablesRemoveArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut vtables = baseline::load_vtables(&ws, &target)?;
    let before = vtables.vtables.len();
    vtables.vtables.retain(|v| v.class != args.class_name);
    if vtables.vtables.len() == before {
        return Err(anyhow!("vtable for '{}' not found", args.class_name));
    }
    baseline::save_vtables(&ws, &target, &vtables)?;
    let out = ok_output(&format!("vtable for '{}' removed", args.class_name));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_vtables_list(cli: &Cli, args: &VtablesListArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let vtables = baseline::load_vtables(&ws, &target)?;
    let out = ok_output_with_data(
        &format!("{} vtables", vtables.vtables.len()),
        serde_yaml::to_value(&vtables.vtables)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_vtables_show(cli: &Cli, args: &VtablesShowArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let vtables = baseline::load_vtables(&ws, &target)?;
    let entry = vtables
        .vtables
        .iter()
        .find(|v| v.class == args.class_name)
        .ok_or_else(|| anyhow!("vtable for '{}' not found", args.class_name))?;
    let out = ok_output_with_data("vtable", serde_yaml::to_value(entry)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: constants
// ===========================================================================

fn exec_constants_add(cli: &Cli, args: &ConstantsAddArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut constants = match baseline::load_constants(&ws, &target) {
        Ok(c) => c,
        Err(_) => baseline::ConstantsYaml {
            target: target.clone(),
            constants: vec![],
        },
    };
    constants.constants.push(baseline::ConstantEntry {
        addr: args.addr.clone(),
        name: args.name.clone(),
        ctype: args.ctype.clone(),
        value: args.value.clone(),
    });
    baseline::save_constants(&ws, &target, &constants)?;
    let out = ok_output(&format!("constant at {} added", args.addr));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_constants_rename(cli: &Cli, args: &ConstantsRenameArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut constants = baseline::load_constants(&ws, &target)?;
    let entry = constants
        .constants
        .iter_mut()
        .find(|c| c.addr == args.addr)
        .ok_or_else(|| anyhow!("constant at {} not found", args.addr))?;
    entry.name = Some(args.new_name.clone());
    baseline::save_constants(&ws, &target, &constants)?;
    let out = ok_output(&format!(
        "constant at {} renamed to {}",
        args.addr, args.new_name
    ));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_constants_remove(cli: &Cli, args: &ConstantsRemoveArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut constants = baseline::load_constants(&ws, &target)?;
    let before = constants.constants.len();
    constants.constants.retain(|c| c.addr != args.addr);
    if constants.constants.len() == before {
        return Err(anyhow!("constant at {} not found", args.addr));
    }
    baseline::save_constants(&ws, &target, &constants)?;
    let out = ok_output(&format!("constant at {} removed", args.addr));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_constants_list(cli: &Cli, args: &ConstantsListArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let constants = baseline::load_constants(&ws, &target)?;
    let out = ok_output_with_data(
        &format!("{} constants", constants.constants.len()),
        serde_yaml::to_value(&constants.constants)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_constants_show(cli: &Cli, args: &ConstantsShowArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let constants = baseline::load_constants(&ws, &target)?;
    let entry = constants
        .constants
        .iter()
        .find(|c| c.addr == args.addr)
        .ok_or_else(|| anyhow!("constant at {} not found", args.addr))?;
    let out = ok_output_with_data("constant", serde_yaml::to_value(entry)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: strings
// ===========================================================================

fn exec_strings_add(cli: &Cli, args: &StringsAddArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut strings = match baseline::load_strings(&ws, &target) {
        Ok(s) => s,
        Err(_) => baseline::StringsYaml {
            target: target.clone(),
            strings: vec![],
        },
    };
    strings.strings.push(baseline::StringEntry {
        addr: args.addr.clone(),
        content: args.content.clone(),
        encoding: args.encoding.clone().unwrap_or_else(|| "utf8".to_string()),
    });
    baseline::save_strings(&ws, &target, &strings)?;
    let out = ok_output(&format!("string at {} added", args.addr));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_strings_remove(cli: &Cli, args: &StringsRemoveArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut strings = baseline::load_strings(&ws, &target)?;
    let before = strings.strings.len();
    strings.strings.retain(|s| s.addr != args.addr);
    if strings.strings.len() == before {
        return Err(anyhow!("string at {} not found", args.addr));
    }
    baseline::save_strings(&ws, &target, &strings)?;
    let out = ok_output(&format!("string at {} removed", args.addr));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_strings_list(cli: &Cli, args: &StringsListArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let strings = baseline::load_strings(&ws, &target)?;
    let out = ok_output_with_data(
        &format!("{} strings", strings.strings.len()),
        serde_yaml::to_value(&strings.strings)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_strings_show(cli: &Cli, args: &StringsShowArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let strings = baseline::load_strings(&ws, &target)?;
    let entry = strings
        .strings
        .iter()
        .find(|s| s.addr == args.addr)
        .ok_or_else(|| anyhow!("string at {} not found", args.addr))?;
    let out = ok_output_with_data("string", serde_yaml::to_value(entry)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: imports
// ===========================================================================

fn exec_imports_add(cli: &Cli, args: &ImportsAddArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut imports = match baseline::load_imports(&ws, &target) {
        Ok(i) => i,
        Err(_) => baseline::ImportsYaml {
            target: target.clone(),
            imports: vec![],
        },
    };
    imports.imports.push(baseline::ImportEntry {
        library: args.library.clone(),
        symbol: args.symbol.clone(),
        plt_addr: args.plt_addr.clone(),
    });
    baseline::save_imports(&ws, &target, &imports)?;
    let out = ok_output(&format!("import {}::{} added", args.library, args.symbol));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_imports_remove(cli: &Cli, args: &ImportsRemoveArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut imports = baseline::load_imports(&ws, &target)?;
    let before = imports.imports.len();
    imports
        .imports
        .retain(|i| !(i.library == args.library && i.symbol == args.symbol));
    if imports.imports.len() == before {
        return Err(anyhow!(
            "import {}::{} not found",
            args.library,
            args.symbol
        ));
    }
    baseline::save_imports(&ws, &target, &imports)?;
    let out = ok_output(&format!("import {}::{} removed", args.library, args.symbol));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_imports_list(cli: &Cli, args: &ImportsListArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let imports = baseline::load_imports(&ws, &target)?;
    let out = ok_output_with_data(
        &format!("{} imports", imports.imports.len()),
        serde_yaml::to_value(&imports.imports)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: third-party
// ===========================================================================

fn exec_third_party_add(cli: &Cli, args: &ThirdPartyAddArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut tp = match third_party::load_third_party(&ws, &target) {
        Ok(t) => t,
        Err(_) => third_party::ThirdPartyYaml {
            target: target.clone(),
            libraries: vec![],
        },
    };
    tp.libraries.push(third_party::ThirdPartyLib {
        library: args.library.clone(),
        version: args.version.clone(),
        confidence: args
            .confidence
            .clone()
            .unwrap_or_else(|| "medium".to_string()),
        evidence: args.evidence.clone(),
        upstream_url: args.upstream_url.clone(),
        vendored_path: None,
        function_classifications: vec![],
    });
    third_party::save_third_party(&ws, &target, &tp)?;
    let out = ok_output(&format!("third-party library '{}' added", args.library));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_third_party_set_version(
    cli: &Cli,
    args: &ThirdPartySetVersionArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut tp = third_party::load_third_party(&ws, &target)?;
    let lib = tp
        .libraries
        .iter_mut()
        .find(|l| l.library == args.library)
        .ok_or_else(|| anyhow!("third-party library '{}' not found", args.library))?;
    lib.version = args.version.clone();
    third_party::save_third_party(&ws, &target, &tp)?;
    let out = ok_output(&format!(
        "version for '{}' set to {}",
        args.library, args.version
    ));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_third_party_list(cli: &Cli, args: &ThirdPartyListArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let tp = third_party::load_third_party(&ws, &target)?;
    let out = ok_output_with_data(
        &format!("{} third-party libraries", tp.libraries.len()),
        serde_yaml::to_value(&tp.libraries)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_third_party_classify_function(
    cli: &Cli,
    args: &ThirdPartyClassifyFunctionArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let mut tp = third_party::load_third_party(&ws, &target)?;
    // Find any library and add classification; for simplicity, add to first library
    if let Some(lib) = tp.libraries.first_mut() {
        lib.function_classifications
            .push(third_party::FunctionClassification {
                addr: args.addr.clone(),
                classification: args.classification.clone(),
                evidence: args.evidence.clone(),
            });
        third_party::save_third_party(&ws, &target, &tp)?;
        let out = ok_output(&format!(
            "function at {} classified as {}",
            args.addr, args.classification
        ));
        serialize_value(&mut std::io::stdout(), &out, fmt)?;
        Ok(EXIT_SUCCESS)
    } else {
        Err(anyhow!(
            "no third-party libraries found; add a library first"
        ))
    }
}

fn exec_third_party_vendor_pristine(
    cli: &Cli,
    args: &ThirdPartyVendorPristineArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let vendored_dir =
        third_party::vendor_pristine(&ws, &target, &args.library, &args.source_path, args.commit)?;
    let out = ok_output(&format!(
        "vendored pristine source for '{}' at {}",
        args.library,
        vendored_dir.display()
    ));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: execution-log
// ===========================================================================

fn exec_execution_log_append(cli: &Cli, args: &ExecutionLogAppendArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let entry = execution_log::LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        script: args.script.clone(),
        status: args.status.clone(),
        inputs_hash: args.inputs_hash.clone(),
        outputs: args.outputs.clone(),
        duration_ms: args.duration_ms.unwrap_or(0),
    };
    execution_log::append_entry(&ws, &target, entry)?;
    let out = ok_output(&format!(
        "execution log entry appended for script '{}'",
        args.script
    ));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_execution_log_list(cli: &Cli, args: &ExecutionLogListArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let log = execution_log::load_execution_log(&ws, &target)?;
    let out = ok_output_with_data(
        &format!("{} entries", log.entries.len()),
        serde_yaml::to_value(&log.entries)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_execution_log_show(cli: &Cli, args: &ExecutionLogShowArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let log = execution_log::load_execution_log(&ws, &target)?;
    let entries: Vec<&execution_log::LogEntry> = log
        .entries
        .iter()
        .filter(|e| e.script == args.script)
        .collect();
    let out = ok_output_with_data(
        &format!("{} entries for script '{}'", entries.len(), args.script),
        serde_yaml::to_value(&entries)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: progress
// ===========================================================================

fn exec_progress_mark_decompiled(
    cli: &Cli,
    args: &ProgressMarkDecompiledArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    progress::mark_function_decompiled(
        &ws,
        &target,
        &args.fn_id,
        &args.addr,
        args.backend.as_deref().unwrap_or("ghidra"),
    )?;
    let out = ok_output(&format!("function {} marked as decompiled", args.fn_id));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_progress_compute_next_batch(
    cli: &Cli,
    args: &ProgressComputeNextBatchArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let strategy = args.strategy.as_deref().unwrap_or("breadth-first");
    let batch = progress::compute_next_batch(&ws, &target, args.max, strategy)?;
    progress::save_next_batch(&ws, &target, &batch)?;
    let out = ok_output_with_data(
        &format!("{} functions in next batch", batch.batch.len()),
        serde_yaml::to_value(&batch)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_progress_show(cli: &Cli, args: &ProgressShowArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let prog = progress::load_progress(&ws, &target)?;
    let out = ok_output_with_data("progress", serde_yaml::to_value(&prog)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_progress_list(cli: &Cli, args: &ProgressListArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let prog = progress::load_progress(&ws, &target)?;
    let out = ok_output_with_data(
        &format!("{} functions tracked", prog.functions.len()),
        serde_yaml::to_value(&prog.functions)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: gate
// ===========================================================================

fn exec_gate_check(cli: &Cli, args: &GateCheckArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;

    let phases: Vec<&str> = match args.phase {
        Some(ref p) if p.as_str() == "all" => {
            vec!["P0", "P0.5", "P1", "P2", "P3", "P4", "P5", "P6"]
        }
        Some(ref p) => vec![p.as_str()],
        None => vec!["P1"], // default phase
    };

    let mut reports = Vec::new();
    let mut all_passed = true;
    for phase in &phases {
        let report = gate::check_phase(&ws, &target, phase)?;
        if !report.passed {
            all_passed = false;
        }
        reports.push(report);
    }

    let out = if all_passed {
        ok_output_with_data("gate check passed", serde_yaml::to_value(&reports)?)
    } else {
        ok_output_with_data("gate check failed", serde_yaml::to_value(&reports)?)
    };
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    if all_passed {
        Ok(EXIT_SUCCESS)
    } else {
        Ok(EXIT_FAILURE)
    }
}

fn exec_gate_list(cli: &Cli, args: &GateListArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;

    let phases = gate::phase_descriptions();
    let mut results = Vec::new();

    for info in &phases {
        let report = gate::check_phase(&ws, &target, &info.phase)?;
        results.push(serde_yaml::to_value(serde_json::json!({
            "phase": info.phase,
            "name": info.name,
            "passed": report.passed,
            "check_count": report.checks.len(),
            "checks": report.checks,
        }))?);
    }

    let all_passed = results
        .iter()
        .all(|r| r.get("passed").and_then(|v| v.as_bool()).unwrap_or(false));
    let msg = if all_passed {
        "all gates passed"
    } else {
        "some gates failed"
    };
    let out = ok_output_with_data(msg, serde_yaml::Value::Sequence(results));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(if all_passed {
        EXIT_SUCCESS
    } else {
        EXIT_FAILURE
    })
}

fn exec_gate_show(cli: &Cli, args: &GateShowArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let phase = args.phase.as_deref().unwrap_or("P1");
    let report = gate::check_phase(&ws, &target, phase)?;
    let out = ok_output_with_data("gate report", serde_yaml::to_value(&report)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: ghidra
// ===========================================================================

fn exec_ghidra_discover(args: &GhidraDiscoverArgs, fmt: Format) -> Result<i32> {
    let ghidra_dir = ghidra::discover_ghidra(args.install_dir.as_deref())?;
    let mut data = serde_yaml::Mapping::new();
    data.insert(
        serde_yaml::Value::String("ghidra_install_dir".into()),
        serde_yaml::Value::String(ghidra_dir.display().to_string()),
    );
    let out = ok_output_with_data("ghidra discovered", serde_yaml::Value::Mapping(data));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_ghidra_import(cli: &Cli, args: &GhidraImportArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let ghidra_dir = ghidra::discover_ghidra(None)?;
    let project_dir = ghidra::ghidra_projects_dir(&ws, &target);
    std::fs::create_dir_all(&project_dir)?;
    let state = workspace::load_pipeline_state(&ws, &target)?;
    let binary_path = state
        .binary
        .ok_or_else(|| anyhow!("binary path not recorded for target '{}'", target))?;
    ghidra::run_headless_import(
        &ws,
        &ghidra_dir,
        &project_dir,
        &target,
        std::path::Path::new(&binary_path),
    )?;
    let out = ok_output(&format!("binary imported for target '{}'", target));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_ghidra_auto_analyze(cli: &Cli, args: &GhidraAutoAnalyzeArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let ghidra_dir = ghidra::discover_ghidra(None)?;
    let project_dir = ghidra::ghidra_projects_dir(&ws, &target);
    let state = workspace::load_pipeline_state(&ws, &target)?;
    let binary_path = state
        .binary
        .ok_or_else(|| anyhow!("binary path not recorded for target '{}'", target))?;

    // Extract binary name - native import creates program at /<binary_name>
    let binary_name = Path::new(&binary_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("invalid binary path '{}'", binary_path))?;
    // Use just the binary name without leading slash for -process
    let program_name = binary_name;

    ghidra::run_headless_with_program(
        &ws,
        &ghidra_dir,
        &project_dir,
        &target,
        "AutoAnalyze.java",
        &[],
        Some(program_name),
    )?;
    let out = ok_output(&format!("auto-analysis complete for target '{}'", target));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_ghidra_export_baseline(
    cli: &Cli,
    args: &GhidraExportBaselineArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let ghidra_dir = ghidra::discover_ghidra(None)?;
    let project_dir = ghidra::ghidra_projects_dir(&ws, &target);
    let state = workspace::load_pipeline_state(&ws, &target)?;
    let binary_path = state
        .binary
        .ok_or_else(|| anyhow!("binary path not recorded for target '{}'", target))?;

    // Extract binary name - native import creates program at /<binary_name>
    let binary_name = Path::new(&binary_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("invalid binary path '{}'", binary_path))?;
    // Use just the binary name without leading slash for -process
    let program_name = binary_name;

    ghidra::run_headless_with_program(
        &ws,
        &ghidra_dir,
        &project_dir,
        &target,
        "ExportBaseline.java",
        &[],
        Some(program_name),
    )?;
    let out = ok_output(&format!("baseline exported for target '{}'", target));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_ghidra_analyze_vtables(
    cli: &Cli,
    args: &GhidraAnalyzeVtablesArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let ghidra_dir = ghidra::discover_ghidra(None)?;
    let project_dir = ghidra::ghidra_projects_dir(&ws, &target);
    let binary_name = decompile_binary_name(&ws, &target)?;
    let report_path = args
        .report_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            workspace::artifact_dir(&ws, &target)
                .join("baseline")
                .join("vtable-analysis-report.yaml")
        });
    let min_entries = args.min_entries.to_string();
    let max_entries = args.max_entries.to_string();
    let scan_limit = args.scan_limit.to_string();
    let min_score = args.min_score.to_string();
    let write_baseline = args.write_baseline.to_string();
    let overwrite = args.overwrite.to_string();
    let report_path_string = report_path.to_string_lossy().into_owned();

    ghidra::run_headless_with_program(
        &ws,
        &ghidra_dir,
        &project_dir,
        &target,
        "AnalyzeVtables.java",
        &[
            &min_entries,
            &max_entries,
            &scan_limit,
            &args.segments,
            &min_score,
            &write_baseline,
            &overwrite,
            &report_path_string,
        ],
        Some(&binary_name),
    )?;

    let mut data = serde_yaml::Mapping::new();
    data.insert(
        serde_yaml::Value::String("report_path".into()),
        serde_yaml::Value::String(report_path.display().to_string()),
    );
    data.insert(
        serde_yaml::Value::String("write_baseline".into()),
        serde_yaml::Value::Bool(args.write_baseline),
    );
    data.insert(
        serde_yaml::Value::String("min_entries".into()),
        serde_yaml::to_value(args.min_entries)?,
    );
    data.insert(
        serde_yaml::Value::String("max_entries".into()),
        serde_yaml::to_value(args.max_entries)?,
    );
    data.insert(
        serde_yaml::Value::String("scan_limit".into()),
        serde_yaml::to_value(args.scan_limit)?,
    );
    data.insert(
        serde_yaml::Value::String("segments".into()),
        serde_yaml::Value::String(args.segments.clone()),
    );
    data.insert(
        serde_yaml::Value::String("min_score".into()),
        serde_yaml::to_value(args.min_score)?,
    );

    let out = ok_output_with_data(
        &format!("vtable analysis complete for target '{}'", target),
        serde_yaml::Value::Mapping(data),
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_ghidra_apply_renames(cli: &Cli, args: &GhidraApplyRenamesArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let ghidra_dir = ghidra::discover_ghidra(None)?;
    let project_dir = ghidra::ghidra_projects_dir(&ws, &target);
    let state = workspace::load_pipeline_state(&ws, &target)?;
    let binary_path = state
        .binary
        .ok_or_else(|| anyhow!("binary path not recorded for target '{}'", target))?;
    let binary_name = Path::new(&binary_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("invalid binary path '{}'", binary_path))?;

    ghidra::run_headless_with_program(
        &ws,
        &ghidra_dir,
        &project_dir,
        &target,
        "ApplyRenames.java",
        &[],
        Some(binary_name),
    )?;
    let out = ok_output(&format!("renames applied for target '{}'", target));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_ghidra_verify_renames(
    cli: &Cli,
    args: &GhidraVerifyRenamesArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let ghidra_dir = ghidra::discover_ghidra(None)?;
    let project_dir = ghidra::ghidra_projects_dir(&ws, &target);
    let state = workspace::load_pipeline_state(&ws, &target)?;
    let binary_path = state
        .binary
        .ok_or_else(|| anyhow!("binary path not recorded for target '{}'", target))?;
    let binary_name = Path::new(&binary_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("invalid binary path '{}'", binary_path))?;

    ghidra::run_headless_with_program(
        &ws,
        &ghidra_dir,
        &project_dir,
        &target,
        "VerifyRenames.java",
        &[],
        Some(binary_name),
    )?;
    let out = ok_output(&format!("renames verified for target '{}'", target));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_ghidra_apply_signatures(
    cli: &Cli,
    args: &GhidraApplySignaturesArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let ghidra_dir = ghidra::discover_ghidra(None)?;
    let project_dir = ghidra::ghidra_projects_dir(&ws, &target);
    let state = workspace::load_pipeline_state(&ws, &target)?;
    let binary_path = state
        .binary
        .ok_or_else(|| anyhow!("binary path not recorded for target '{}'", target))?;
    let binary_name = Path::new(&binary_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("invalid binary path '{}'", binary_path))?;

    ghidra::run_headless_with_program(
        &ws,
        &ghidra_dir,
        &project_dir,
        &target,
        "ApplySignatures.java",
        &[],
        Some(binary_name),
    )?;
    let out = ok_output(&format!("signatures applied for target '{}'", target));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_ghidra_verify_signatures(
    cli: &Cli,
    args: &GhidraVerifySignaturesArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let ghidra_dir = ghidra::discover_ghidra(None)?;
    let project_dir = ghidra::ghidra_projects_dir(&ws, &target);
    let state = workspace::load_pipeline_state(&ws, &target)?;
    let binary_path = state
        .binary
        .ok_or_else(|| anyhow!("binary path not recorded for target '{}'", target))?;
    let binary_name = Path::new(&binary_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("invalid binary path '{}'", binary_path))?;

    ghidra::run_headless_with_program(
        &ws,
        &ghidra_dir,
        &project_dir,
        &target,
        "VerifyFunctionSignatures.java",
        &[],
        Some(binary_name),
    )?;
    let out = ok_output(&format!("signatures verified for target '{}'", target));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_ghidra_decompile(cli: &Cli, args: &GhidraDecompileArgs, fmt: Format) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;

    if args.batch {
        if args.fn_id.is_some() || args.addr.is_some() {
            return Err(anyhow!("--batch cannot be combined with --fn-id or --addr"));
        }
    } else if args.fn_id.is_none() || args.addr.is_none() {
        return Err(anyhow!(
            "--fn-id and --addr are required unless --batch is set"
        ));
    }

    if args.batch {
        let next_batch = progress::load_next_batch(&ws, &target)?;
        let ghidra_dir = ghidra::discover_ghidra(None)?;
        let project_dir = ghidra::ghidra_projects_dir(&ws, &target);
        let binary_name = decompile_binary_name(&ws, &target)?;
        let mut results = Vec::with_capacity(next_batch.batch.len());

        for entry in &next_batch.batch {
            match run_ghidra_decompile_function(
                &ws,
                &target,
                &ghidra_dir,
                &project_dir,
                &binary_name,
                &entry.fn_id,
                &entry.addr,
            ) {
                Ok(()) => {
                    progress::mark_function_decompiled(
                        &ws,
                        &target,
                        &entry.fn_id,
                        &entry.addr,
                        "ghidra",
                    )?;
                    results.push(DecompileBatchResult {
                        fn_id: entry.fn_id.clone(),
                        addr: entry.addr.clone(),
                        status: "ok".to_string(),
                        error: None,
                    });
                }
                Err(err) => {
                    results.push(DecompileBatchResult {
                        fn_id: entry.fn_id.clone(),
                        addr: entry.addr.clone(),
                        status: "failed".to_string(),
                        error: Some(err.to_string()),
                    });
                }
            }
        }

        let failed = results.iter().filter(|r| r.status == "failed").count();
        let summary = DecompileBatchSummary {
            target: target.clone(),
            requested: results.len(),
            succeeded: results.len().saturating_sub(failed),
            failed,
            results,
        };
        let message = if failed == 0 {
            format!("batch decompilation complete for target '{}'", target)
        } else {
            format!(
                "batch decompilation completed with {} failure(s) for target '{}'",
                failed, target
            )
        };
        let out = ok_output_with_data(&message, serde_yaml::to_value(&summary)?);
        serialize_value(&mut std::io::stdout(), &out, fmt)?;
        return Ok(if failed == 0 {
            EXIT_SUCCESS
        } else {
            EXIT_FAILURE
        });
    }

    let ghidra_dir = ghidra::discover_ghidra(None)?;
    let project_dir = ghidra::ghidra_projects_dir(&ws, &target);
    let binary_name = decompile_binary_name(&ws, &target)?;

    let fn_id = args.fn_id.as_ref().expect("validated above");
    let addr = args.addr.as_ref().expect("validated above");

    run_ghidra_decompile_function(
        &ws,
        &target,
        &ghidra_dir,
        &project_dir,
        &binary_name,
        fn_id,
        addr,
    )?;
    let out = ok_output(&format!("decompilation complete for target '{}'", target));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

#[derive(Debug, Serialize)]
struct DecompileBatchResult {
    fn_id: String,
    addr: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct DecompileBatchSummary {
    target: String,
    requested: usize,
    succeeded: usize,
    failed: usize,
    results: Vec<DecompileBatchResult>,
}

fn decompile_binary_name(ws: &Path, target: &str) -> Result<String> {
    let state = workspace::load_pipeline_state(ws, target)?;
    let binary_path = state
        .binary
        .ok_or_else(|| anyhow!("binary path not recorded for target '{}'", target))?;
    Path::new(&binary_path)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|name| name.to_string())
        .ok_or_else(|| anyhow!("invalid binary path '{}'", binary_path))
}

fn run_ghidra_decompile_function(
    ws: &Path,
    target: &str,
    ghidra_dir: &Path,
    project_dir: &Path,
    binary_name: &str,
    fn_id: &str,
    addr: &str,
) -> Result<()> {
    ghidra::run_headless_with_program(
        ws,
        ghidra_dir,
        project_dir,
        target,
        "DecompileFunction.java",
        &[addr, fn_id],
        Some(binary_name),
    )
}

fn exec_ghidra_rebuild_project(
    cli: &Cli,
    args: &GhidraRebuildProjectArgs,
    fmt: Format,
) -> Result<i32> {
    let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
    let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
    let ghidra_dir = ghidra::discover_ghidra(None)?;
    let project_dir = ghidra::ghidra_projects_dir(&ws, &target);
    let state = workspace::load_pipeline_state(&ws, &target)?;
    let binary_path = state
        .binary
        .ok_or_else(|| anyhow!("binary path not recorded for target '{}'", target))?;
    ghidra::run_headless(
        &ws,
        &ghidra_dir,
        &project_dir,
        &target,
        "RebuildProject.java",
        &[&binary_path],
    )?;
    let out = ok_output(&format!("project rebuilt for target '{}'", target));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: frida
// ===========================================================================
use ghidra_agent_cli::frida::{self, DeviceSelector};

fn exec_frida_device_list(_cli: &Cli, fmt: Format) -> Result<i32> {
    let version = frida::check_frida_available()?;
    let devices = frida::list_devices()?;
    let out = ok_output_with_data(
        &format!("Frida {}", version),
        serde_yaml::to_value(&devices)?,
    );
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_frida_device_attach(_cli: &Cli, args: &DeviceAttachArgs, fmt: Format) -> Result<i32> {
    let selector = DeviceSelector::parse(args.device.as_deref().unwrap_or("local"));

    // Build the frida attach command
    let mut cmd = std::process::Command::new("frida");
    cmd.args(selector.to_frida_args());

    if let Some(pid) = args.pid {
        cmd.arg(pid.to_string());
    } else {
        return Err(anyhow!("--pid is required for device attach"));
    }

    // Run frida interactively — pipe stdin through
    cmd.stdin(std::process::Stdio::inherit());
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());

    let status = cmd
        .status()
        .map_err(|e| anyhow!("failed to run frida: {e}"))?;

    let out = ok_output(&format!(
        "frida attach exited with code {}",
        status.code().unwrap_or(-1)
    ));
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(status.code().unwrap_or(EXIT_FAILURE))
}

fn exec_frida_io_capture(_cli: &Cli, args: &IoCaptureArgs, _fmt: Format) -> Result<i32> {
    let selector = DeviceSelector::parse(args.device.as_deref().unwrap_or("local"));
    let proc_args = if args.proc_args.is_empty() {
        None
    } else {
        Some(args.proc_args.join(" "))
    };
    let output =
        frida::run_io_capture(&args.target, proc_args.as_deref(), &selector, args.timeout)?;
    println!("{}", output);
    Ok(EXIT_SUCCESS)
}

fn exec_frida_signature_analysis(
    _cli: &Cli,
    args: &SignatureAnalysisArgs,
    _fmt: Format,
) -> Result<i32> {
    let selector = DeviceSelector::parse(args.device.as_deref().unwrap_or("local"));
    let proc_args = if args.proc_args.is_empty() {
        None
    } else {
        Some(args.proc_args.join(" "))
    };
    let output = frida::run_signature_analysis(
        args.target.as_deref(),
        &args.funcs,
        proc_args.as_deref(),
        &selector,
    )?;
    println!("{}", output);
    Ok(EXIT_SUCCESS)
}

fn exec_frida_call_tree_trace(_cli: &Cli, args: &CallTreeTraceArgs, _fmt: Format) -> Result<i32> {
    let selector = DeviceSelector::parse(args.device.as_deref().unwrap_or("local"));
    let proc_args = if args.proc_args.is_empty() {
        None
    } else {
        Some(args.proc_args.join(" "))
    };
    let output = frida::run_call_tree_trace(
        args.target.as_deref(),
        args.max_depth,
        args.libs.as_deref(),
        proc_args.as_deref(),
        &selector,
    )?;
    println!("{}", output);
    Ok(EXIT_SUCCESS)
}

fn exec_frida_dispatch_vtable_trace(
    _cli: &Cli,
    args: &DispatchVtableTraceArgs,
    _fmt: Format,
) -> Result<i32> {
    let selector = DeviceSelector::parse(args.device.as_deref().unwrap_or("local"));
    let output = frida::run_dispatch_vtable_trace(&args.ranges, &selector)?;
    println!("{}", output);
    Ok(EXIT_SUCCESS)
}

fn exec_frida_hotpath_coverage(
    _cli: &Cli,
    args: &HotpathCoverageArgs,
    _fmt: Format,
) -> Result<i32> {
    let selector = DeviceSelector::parse(args.device.as_deref().unwrap_or("local"));
    let proc_args = if args.proc_args.is_empty() {
        None
    } else {
        Some(args.proc_args.join(" "))
    };
    let output = frida::run_hotpath_coverage(
        args.target.as_deref(),
        args.threshold,
        args.interval,
        proc_args.as_deref(),
        &selector,
    )?;
    println!("{}", output);
    Ok(EXIT_SUCCESS)
}

fn exec_frida_io_compare(_cli: &Cli, args: &IoCompareArgs, _fmt: Format) -> Result<i32> {
    let output = frida::run_io_compare(&args.original, &args.reconstructed)?;
    println!("{}", output);
    Ok(EXIT_SUCCESS)
}

fn exec_frida_decomp_compare(_cli: &Cli, args: &DecompCompareArgs, _fmt: Format) -> Result<i32> {
    let selector = DeviceSelector::parse(args.log.as_deref().unwrap_or("local"));
    let proc_args = if args.proc_args.is_empty() {
        None
    } else {
        Some(args.proc_args.join(" "))
    };
    let output = frida::run_decomp_compare(
        args.target.as_deref(),
        &args.func,
        proc_args.as_deref(),
        &selector,
    )?;
    println!("{}", output);
    Ok(EXIT_SUCCESS)
}

fn exec_frida_fuzz_input_gen(_cli: &Cli, args: &FuzzInputGenArgs, _fmt: Format) -> Result<i32> {
    let output = frida::run_fuzz_input_gen(&args.types_yaml, args.output.as_deref())?;
    println!("{}", output);
    Ok(EXIT_SUCCESS)
}

fn exec_frida_run(_cli: &Cli, args: &FridaRunArgs, _fmt: Format) -> Result<i32> {
    let selector = DeviceSelector::parse(args.device.as_deref().unwrap_or("local"));

    let output = frida::run_script(
        frida::FRIDA_RUN_JS,
        &args.target,
        args.args.as_deref(),
        args.stdin.as_deref(),
        args.timeout,
        &selector,
    )?;

    println!("{}", output);
    Ok(EXIT_SUCCESS)
}

fn exec_frida_trace(_cli: &Cli, args: &FridaTraceArgs, _fmt: Format) -> Result<i32> {
    let selector = DeviceSelector::parse(args.device.as_deref().unwrap_or("local"));

    let proc_args = if args.proc_args.is_empty() {
        None
    } else {
        Some(args.proc_args.join(" "))
    };

    let output = frida::run_trace_script(
        frida::FRIDA_TRACE_JS,
        &args.target,
        args.functions.as_deref(),
        proc_args.as_deref(),
        args.timeout,
        &selector,
    )?;

    println!("{}", output);
    Ok(EXIT_SUCCESS)
}

fn exec_frida_invoke(_cli: &Cli, args: &FridaInvokeArgs, _fmt: Format) -> Result<i32> {
    let selector = DeviceSelector::parse(args.device.as_deref().unwrap_or("local"));

    let output = frida::run_invoke_script(
        frida::FRIDA_INVOKE_JS,
        &args.target,
        &args.function,
        args.signature.as_deref(),
        args.args.as_deref(),
        &selector,
    )?;

    println!("{}", output);
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Dispatch: inspect
// ===========================================================================

// ===========================================================================
// Dispatch: inspect
// ===========================================================================

#[derive(serde::Serialize)]
struct InspectResult {
    target: String,
    file_type: String,
    exports: Vec<SymbolInfo>,
    imports: Vec<SymbolInfo>,
    objects: Vec<ObjectInfo>,
}

#[derive(serde::Serialize)]
struct SymbolInfo {
    name: String,
    address: String,
    symbol_type: String,
}

#[derive(serde::Serialize)]
struct ObjectInfo {
    name: String,
    symbols: Vec<SymbolInfo>,
}

fn exec_inspect_binary(args: &InspectBinaryArgs, fmt: Format) -> Result<i32> {
    use std::path::Path;
    use std::process::Command;

    let path = Path::new(&args.target);
    if !path.exists() {
        return Err(anyhow!("Target file does not exist: {}", args.target));
    }

    // Detect file type
    let output = Command::new("file").arg(&args.target).output()?;
    let file_type = String::from_utf8_lossy(&output.stdout).to_string();

    let mut result = InspectResult {
        target: args.target.clone(),
        file_type: file_type.trim().to_string(),
        exports: vec![],
        imports: vec![],
        objects: vec![],
    };

    // Check if it's a static library (.a)
    if args.target.ends_with(".a") {
        // List archive contents
        let ar_output = Command::new("ar").args(["-t", &args.target]).output()?;
        let ar_content = String::from_utf8_lossy(&ar_output.stdout);
        for line in ar_content.lines() {
            let obj_name = line.trim().to_string();
            if !obj_name.is_empty() && obj_name != "__.SYMDEF SORTED" && obj_name != "__.SYMDEF" {
                // Get symbols from object file using ar and nm in a temp dir
                let obj_symbols = extract_object_symbols(&args.target, &obj_name)?;
                result.objects.push(ObjectInfo {
                    name: obj_name,
                    symbols: obj_symbols,
                });
            }
        }
    } else if args.exports || args.imports {
        // Use nm for dynamic libraries and executables
        let nm_output = Command::new("nm").args(["-g", &args.target]).output()?;
        let nm_content = String::from_utf8_lossy(&nm_output.stdout);
        for line in nm_content.lines() {
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() >= 3 {
                let addr = parts[0].to_string();
                let sym_type = parts[1].to_string();
                let name = parts[2].to_string();

                if args.exports && (sym_type == "T" || sym_type == "t") {
                    result.exports.push(SymbolInfo {
                        name: name.clone(),
                        address: addr,
                        symbol_type: "function".to_string(),
                    });
                } else if args.imports && (sym_type == "U") {
                    result.imports.push(SymbolInfo {
                        name: name.clone(),
                        address: addr,
                        symbol_type: "undefined".to_string(),
                    });
                }
            }
        }
    }

    let out = ok_output_with_data("inspect", serde_yaml::to_value(&result)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn extract_object_symbols(archive_path: &str, obj_name: &str) -> Result<Vec<SymbolInfo>> {
    use std::process::Command;
    use tempfile::TempDir;

    // Create temp directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Extract object file
    let extract_output = Command::new("ar")
        .args(["-x", archive_path])
        .current_dir(temp_path)
        .output()?;
    if !extract_output.status.success() {
        return Ok(vec![]);
    }

    // Run nm on the extracted object
    let obj_path = temp_path.join(obj_name);
    let nm_output = Command::new("nm").arg(&obj_path).output()?;

    let nm_content = String::from_utf8_lossy(&nm_output.stdout);
    let mut symbols = vec![];
    for line in nm_content.lines() {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() >= 3 {
            symbols.push(SymbolInfo {
                name: parts[2].to_string(),
                address: parts[0].to_string(),
                symbol_type: parts[1].to_string(),
            });
        }
    }

    Ok(symbols)
}

// ===========================================================================
// Dispatch: context
// ===========================================================================

fn exec_context_show(cli: &Cli, fmt: Format) -> Result<i32> {
    let overrides = make_runtime_overrides(cli);
    let runtime = context::resolve_runtime_locations(&overrides, false)?;
    let inv_overrides = context::InvocationContextOverrides {
        selectors: BTreeMap::new(),
        current_directory: Some(std::env::current_dir()?),
    };
    let inspection = context::inspect_context(&runtime, &inv_overrides)?;
    let out = ok_output_with_data("context", serde_yaml::to_value(&inspection)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_context_use(cli: &Cli, args: &ContextUseArgs, fmt: Format) -> Result<i32> {
    let overrides = make_runtime_overrides(cli);
    let runtime = context::resolve_runtime_locations(&overrides, false)?;
    let selectors = context::parse_selectors(&args.selectors)?;
    let state = context::build_context_state(None, selectors, Some(std::env::current_dir()?));
    let result = context::persist_active_context(&runtime, &state)?;
    let out = ok_output_with_data(&result.message, serde_yaml::to_value(&result)?);
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

fn exec_context_clear(cli: &Cli, fmt: Format) -> Result<i32> {
    let overrides = make_runtime_overrides(cli);
    let runtime = context::resolve_runtime_locations(&overrides, false)?;
    let cf = runtime.context_file();
    if cf.exists() {
        std::fs::remove_file(&cf)?;
    }
    let out = ok_output("active context cleared");
    serialize_value(&mut std::io::stdout(), &out, fmt)?;
    Ok(EXIT_SUCCESS)
}

// ===========================================================================
// Lock guard for automatic release
// ===========================================================================

fn with_lock<F>(workspace: &std::path::Path, target: Option<&str>, scope: &str, f: F) -> Result<i32>
where
    F: FnOnce() -> Result<i32>,
{
    let lock_path = if let Some(t) = target {
        workspace
            .join(".lock")
            .join(t)
            .join(format!("{scope}.lock"))
    } else {
        workspace
            .join(".lock")
            .join(format!("_workspace_{scope}.lock"))
    };
    // acquire_lock returns a LockGuard that holds the OS file lock until dropped.
    // The guard's Drop impl releases the OS lock but does NOT delete the lock file.
    // We explicitly drop the guard first to release the OS lock, then call release_lock.
    let _guard = lock::acquire_lock(&lock_path, scope, 60)?;
    let result = f();
    // Drop the guard to release the OS lock before we try to acquire exclusive lock
    drop(_guard);
    // Release the lock and clean up the lock file
    let _ = lock::release_lock(&lock_path);
    result
}

// ===========================================================================
// Main entry point
// ===========================================================================

fn main() {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            // Non-leaf containers invoked bare should render man-like help and exit 0
            if e.kind() == clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand {
                let text = render_man_help();
                print!("{text}");
                std::process::exit(EXIT_SUCCESS);
            }
            // All other clap errors (unknown args, bad values, etc.) use default handling
            e.exit();
        }
    };

    let fmt: Format = cli.format.into();

    // --help flag renders man-like text and exits 0
    if cli.help {
        let text = render_man_help();
        print!("{text}");
        std::process::exit(EXIT_SUCCESS);
    }

    // No subcommand -> render man-like help and exit 0
    let Some(command) = &cli.command else {
        let text = render_man_help();
        print!("{text}");
        std::process::exit(EXIT_SUCCESS);
    };

    // Dispatch to subcommand handler
    let result = dispatch(&cli, command, fmt);

    match result {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            let code = exit_code_for_error(&err);
            let serr = StructuredError::new("E_ERROR", format!("{err:#}"), "main", fmt);
            let _ = write_structured_error(&mut std::io::stderr(), &serr, fmt);
            std::process::exit(code);
        }
    }
}

fn dispatch(cli: &Cli, command: &Commands, fmt: Format) -> Result<i32> {
    match command {
        // Read-only commands (no lock needed)
        Commands::Validate(args) => exec_validate(cli, args, fmt),
        Commands::Paths(_args) => exec_paths(cli, fmt),
        Commands::Help(args) => exec_help(args, fmt),

        // workspace
        Commands::Workspace(WorkspaceCmd::Init(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            with_lock(&ws, Some(&args.target), "init", || {
                exec_workspace_init(cli, args, fmt)
            })
        }
        Commands::Workspace(WorkspaceCmd::State(WorkspaceStateCmd::Show(args))) => {
            exec_workspace_state_show(cli, args, fmt)
        }
        Commands::Workspace(WorkspaceCmd::State(WorkspaceStateCmd::SetPhase(args))) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "phase", || {
                exec_workspace_state_set_phase(cli, args, fmt)
            })
        }

        // scope
        Commands::Scope(ScopeCmd::Show(args)) => exec_scope_show(cli, args, fmt),
        Commands::Scope(ScopeCmd::Set(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "scope", || {
                exec_scope_set(cli, args, fmt)
            })
        }
        Commands::Scope(ScopeCmd::AddEntry(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "scope", || {
                exec_scope_add_entry(cli, args, fmt)
            })
        }
        Commands::Scope(ScopeCmd::RemoveEntry(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "scope", || {
                exec_scope_remove_entry(cli, args, fmt)
            })
        }

        // functions
        Commands::Functions(FunctionsCmd::Add(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "functions", || {
                exec_functions_add(cli, args, fmt)
            })
        }
        Commands::Functions(FunctionsCmd::Rename(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "functions", || {
                exec_functions_rename(cli, args, fmt)
            })
        }
        Commands::Functions(FunctionsCmd::SetPrototype(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "functions", || {
                exec_functions_set_prototype(cli, args, fmt)
            })
        }
        Commands::Functions(FunctionsCmd::List(args)) => exec_functions_list(cli, args, fmt),
        Commands::Functions(FunctionsCmd::Show(args)) => exec_functions_show(cli, args, fmt),
        Commands::Functions(FunctionsCmd::Remove(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "functions", || {
                exec_functions_remove(cli, args, fmt)
            })
        }

        // callgraph
        Commands::Callgraph(CallgraphCmd::AddEdge(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "callgraph", || {
                exec_callgraph_add_edge(cli, args, fmt)
            })
        }
        Commands::Callgraph(CallgraphCmd::RemoveEdge(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "callgraph", || {
                exec_callgraph_remove_edge(cli, args, fmt)
            })
        }
        Commands::Callgraph(CallgraphCmd::List(args)) => exec_callgraph_list(cli, args, fmt),
        Commands::Callgraph(CallgraphCmd::Callers(args)) => exec_callgraph_callers(cli, args, fmt),
        Commands::Callgraph(CallgraphCmd::Callees(args)) => exec_callgraph_callees(cli, args, fmt),

        // types
        Commands::Types(TypesCmd::Add(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "types", || {
                exec_types_add(cli, args, fmt)
            })
        }
        Commands::Types(TypesCmd::Remove(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "types", || {
                exec_types_remove(cli, args, fmt)
            })
        }
        Commands::Types(TypesCmd::List(args)) => exec_types_list(cli, args, fmt),
        Commands::Types(TypesCmd::Show(args)) => exec_types_show(cli, args, fmt),

        // vtables
        Commands::Vtables(VtablesCmd::Add(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "vtables", || {
                exec_vtables_add(cli, args, fmt)
            })
        }
        Commands::Vtables(VtablesCmd::Remove(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "vtables", || {
                exec_vtables_remove(cli, args, fmt)
            })
        }
        Commands::Vtables(VtablesCmd::List(args)) => exec_vtables_list(cli, args, fmt),
        Commands::Vtables(VtablesCmd::Show(args)) => exec_vtables_show(cli, args, fmt),

        // constants
        Commands::Constants(ConstantsCmd::Add(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "constants", || {
                exec_constants_add(cli, args, fmt)
            })
        }
        Commands::Constants(ConstantsCmd::Rename(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "constants", || {
                exec_constants_rename(cli, args, fmt)
            })
        }
        Commands::Constants(ConstantsCmd::Remove(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "constants", || {
                exec_constants_remove(cli, args, fmt)
            })
        }
        Commands::Constants(ConstantsCmd::List(args)) => exec_constants_list(cli, args, fmt),
        Commands::Constants(ConstantsCmd::Show(args)) => exec_constants_show(cli, args, fmt),

        // strings
        Commands::Strings(StringsCmd::Add(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "strings", || {
                exec_strings_add(cli, args, fmt)
            })
        }
        Commands::Strings(StringsCmd::Remove(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "strings", || {
                exec_strings_remove(cli, args, fmt)
            })
        }
        Commands::Strings(StringsCmd::List(args)) => exec_strings_list(cli, args, fmt),
        Commands::Strings(StringsCmd::Show(args)) => exec_strings_show(cli, args, fmt),

        // imports
        Commands::Imports(ImportsCmd::Add(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "imports", || {
                exec_imports_add(cli, args, fmt)
            })
        }
        Commands::Imports(ImportsCmd::Remove(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "imports", || {
                exec_imports_remove(cli, args, fmt)
            })
        }
        Commands::Imports(ImportsCmd::List(args)) => exec_imports_list(cli, args, fmt),

        // third-party
        Commands::ThirdParty(ThirdPartyCmd::Add(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "third_party", || {
                exec_third_party_add(cli, args, fmt)
            })
        }
        Commands::ThirdParty(ThirdPartyCmd::SetVersion(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "third_party", || {
                exec_third_party_set_version(cli, args, fmt)
            })
        }
        Commands::ThirdParty(ThirdPartyCmd::List(args)) => exec_third_party_list(cli, args, fmt),
        Commands::ThirdParty(ThirdPartyCmd::ClassifyFunction(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "third_party", || {
                exec_third_party_classify_function(cli, args, fmt)
            })
        }
        Commands::ThirdParty(ThirdPartyCmd::VendorPristine(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "vendor", || {
                exec_third_party_vendor_pristine(cli, args, fmt)
            })
        }

        // execution-log
        Commands::ExecutionLog(ExecutionLogCmd::Append(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "exec_log", || {
                exec_execution_log_append(cli, args, fmt)
            })
        }
        Commands::ExecutionLog(ExecutionLogCmd::List(args)) => {
            exec_execution_log_list(cli, args, fmt)
        }
        Commands::ExecutionLog(ExecutionLogCmd::Show(args)) => {
            exec_execution_log_show(cli, args, fmt)
        }

        // progress
        Commands::Progress(ProgressCmd::MarkDecompiled(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "progress", || {
                exec_progress_mark_decompiled(cli, args, fmt)
            })
        }
        Commands::Progress(ProgressCmd::ComputeNextBatch(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "batch", || {
                exec_progress_compute_next_batch(cli, args, fmt)
            })
        }
        Commands::Progress(ProgressCmd::Show(args)) => exec_progress_show(cli, args, fmt),
        Commands::Progress(ProgressCmd::List(args)) => exec_progress_list(cli, args, fmt),

        // gate
        Commands::Gate(GateCmd::Check(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "gate", || {
                exec_gate_check(cli, args, fmt)
            })
        }
        Commands::Gate(GateCmd::List(args)) => exec_gate_list(cli, args, fmt),
        Commands::Gate(GateCmd::Show(args)) => exec_gate_show(cli, args, fmt),

        // ghidra
        Commands::Ghidra(GhidraCmd::Discover(args)) => exec_ghidra_discover(args, fmt),
        Commands::Ghidra(GhidraCmd::Import(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "ghidra", || {
                exec_ghidra_import(cli, args, fmt)
            })
        }
        Commands::Ghidra(GhidraCmd::AutoAnalyze(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "ghidra", || {
                exec_ghidra_auto_analyze(cli, args, fmt)
            })
        }
        Commands::Ghidra(GhidraCmd::ExportBaseline(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "ghidra", || {
                exec_ghidra_export_baseline(cli, args, fmt)
            })
        }
        Commands::Ghidra(GhidraCmd::AnalyzeVtables(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "ghidra", || {
                exec_ghidra_analyze_vtables(cli, args, fmt)
            })
        }
        Commands::Ghidra(GhidraCmd::ApplyRenames(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "ghidra", || {
                exec_ghidra_apply_renames(cli, args, fmt)
            })
        }
        Commands::Ghidra(GhidraCmd::VerifyRenames(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "ghidra", || {
                exec_ghidra_verify_renames(cli, args, fmt)
            })
        }
        Commands::Ghidra(GhidraCmd::ApplySignatures(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "ghidra", || {
                exec_ghidra_apply_signatures(cli, args, fmt)
            })
        }
        Commands::Ghidra(GhidraCmd::VerifySignatures(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "ghidra", || {
                exec_ghidra_verify_signatures(cli, args, fmt)
            })
        }
        Commands::Ghidra(GhidraCmd::Decompile(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "ghidra", || {
                exec_ghidra_decompile(cli, args, fmt)
            })
        }
        Commands::Ghidra(GhidraCmd::RebuildProject(args)) => {
            let ws = resolve_workspace(cli.workspace.as_ref(), cli.workspace.as_ref())?;
            let target = resolve_target(args.target.as_ref(), cli.target.as_ref())?;
            with_lock(&ws, Some(&target), "ghidra", || {
                exec_ghidra_rebuild_project(cli, args, fmt)
            })
        }

        // frida
        Commands::Frida(FridaCmd::DeviceList) => {
            with_lock(std::path::Path::new("."), None, "frida", || {
                exec_frida_device_list(cli, fmt)
            })
        }
        Commands::Frida(FridaCmd::DeviceAttach(args)) => exec_frida_device_attach(cli, args, fmt),
        Commands::Frida(FridaCmd::IoCapture(args)) => {
            with_lock(std::path::Path::new("."), None, "frida", || {
                exec_frida_io_capture(cli, args, fmt)
            })
        }
        Commands::Frida(FridaCmd::SignatureAnalysis(args)) => {
            with_lock(std::path::Path::new("."), None, "frida", || {
                exec_frida_signature_analysis(cli, args, fmt)
            })
        }
        Commands::Frida(FridaCmd::CallTreeTrace(args)) => {
            with_lock(std::path::Path::new("."), None, "frida", || {
                exec_frida_call_tree_trace(cli, args, fmt)
            })
        }
        Commands::Frida(FridaCmd::DispatchVtableTrace(args)) => {
            with_lock(std::path::Path::new("."), None, "frida", || {
                exec_frida_dispatch_vtable_trace(cli, args, fmt)
            })
        }
        Commands::Frida(FridaCmd::HotpathCoverage(args)) => {
            with_lock(std::path::Path::new("."), None, "frida", || {
                exec_frida_hotpath_coverage(cli, args, fmt)
            })
        }
        Commands::Frida(FridaCmd::IoCompare(args)) => {
            with_lock(std::path::Path::new("."), None, "frida", || {
                exec_frida_io_compare(cli, args, fmt)
            })
        }
        Commands::Frida(FridaCmd::DecompCompare(args)) => {
            with_lock(std::path::Path::new("."), None, "frida", || {
                exec_frida_decomp_compare(cli, args, fmt)
            })
        }
        Commands::Frida(FridaCmd::FuzzInputGen(args)) => {
            with_lock(std::path::Path::new("."), None, "frida", || {
                exec_frida_fuzz_input_gen(cli, args, fmt)
            })
        }
        Commands::Frida(FridaCmd::Run(args)) => {
            with_lock(std::path::Path::new("."), None, "frida", || {
                exec_frida_run(cli, args, fmt)
            })
        }
        Commands::Frida(FridaCmd::Trace(args)) => {
            with_lock(std::path::Path::new("."), None, "frida", || {
                exec_frida_trace(cli, args, fmt)
            })
        }
        Commands::Frida(FridaCmd::Invoke(args)) => {
            with_lock(std::path::Path::new("."), None, "frida", || {
                exec_frida_invoke(cli, args, fmt)
            })
        }

        // inspect
        Commands::Inspect(InspectCmd::Binary(args)) => exec_inspect_binary(args, fmt),

        // context
        Commands::Context(ContextCmd::Show) => exec_context_show(cli, fmt),
        Commands::Context(ContextCmd::Use(args)) => {
            with_lock(std::path::Path::new("."), None, "context", || {
                exec_context_use(cli, args, fmt)
            })
        }
        Commands::Context(ContextCmd::Clear) => {
            with_lock(std::path::Path::new("."), None, "context", || {
                exec_context_clear(cli, fmt)
            })
        }
    }
}
