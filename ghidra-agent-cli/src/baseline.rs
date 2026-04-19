use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::schema::{load_yaml, save_yaml};
use crate::workspace::artifact_dir;

// --- functions.yaml ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionEntry {
    pub addr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prototype: Option<String>,
    #[serde(default)]
    pub size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "ghidra".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionsYaml {
    pub target: String,
    pub functions: Vec<FunctionEntry>,
}

pub fn load_functions(workspace: &Path, target: &str) -> Result<FunctionsYaml> {
    load_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("functions.yaml"),
    )
}

pub fn save_functions(workspace: &Path, target: &str, data: &FunctionsYaml) -> Result<()> {
    save_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("functions.yaml"),
        data,
    )
}

// --- callgraph.yaml ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    pub from: String,
    pub to: String,
    #[serde(default = "default_direct")]
    pub kind: String,
}

fn default_direct() -> String {
    "direct".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallgraphYaml {
    pub target: String,
    pub edges: Vec<CallEdge>,
}

pub fn load_callgraph(workspace: &Path, target: &str) -> Result<CallgraphYaml> {
    load_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("callgraph.yaml"),
    )
}

pub fn save_callgraph(workspace: &Path, target: &str, data: &CallgraphYaml) -> Result<()> {
    save_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("callgraph.yaml"),
        data,
    )
}

// --- types.yaml ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeEntry {
    pub name: String,
    pub kind: String,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypesYaml {
    pub target: String,
    pub types: Vec<TypeEntry>,
}

pub fn load_types(workspace: &Path, target: &str) -> Result<TypesYaml> {
    load_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("types.yaml"),
    )
}

pub fn save_types(workspace: &Path, target: &str, data: &TypesYaml) -> Result<()> {
    save_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("types.yaml"),
        data,
    )
}

// --- vtables.yaml ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VtableEntry {
    pub class: String,
    pub addr: String,
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VtablesYaml {
    pub target: String,
    pub vtables: Vec<VtableEntry>,
}

pub fn load_vtables(workspace: &Path, target: &str) -> Result<VtablesYaml> {
    load_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("vtables.yaml"),
    )
}

pub fn save_vtables(workspace: &Path, target: &str, data: &VtablesYaml) -> Result<()> {
    save_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("vtables.yaml"),
        data,
    )
}

// --- constants.yaml ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantEntry {
    pub addr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctype: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantsYaml {
    pub target: String,
    pub constants: Vec<ConstantEntry>,
}

pub fn load_constants(workspace: &Path, target: &str) -> Result<ConstantsYaml> {
    load_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("constants.yaml"),
    )
}

pub fn save_constants(workspace: &Path, target: &str, data: &ConstantsYaml) -> Result<()> {
    save_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("constants.yaml"),
        data,
    )
}

// --- strings.yaml ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringEntry {
    pub addr: String,
    pub content: String,
    #[serde(default = "default_encoding")]
    pub encoding: String,
}

fn default_encoding() -> String {
    "utf8".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringsYaml {
    pub target: String,
    pub strings: Vec<StringEntry>,
}

pub fn load_strings(workspace: &Path, target: &str) -> Result<StringsYaml> {
    load_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("strings.yaml"),
    )
}

pub fn save_strings(workspace: &Path, target: &str, data: &StringsYaml) -> Result<()> {
    save_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("strings.yaml"),
        data,
    )
}

// --- imports.yaml ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEntry {
    pub library: String,
    pub symbol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plt_addr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportsYaml {
    pub target: String,
    pub imports: Vec<ImportEntry>,
}

pub fn load_imports(workspace: &Path, target: &str) -> Result<ImportsYaml> {
    load_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("imports.yaml"),
    )
}

pub fn save_imports(workspace: &Path, target: &str, data: &ImportsYaml) -> Result<()> {
    save_yaml(
        &artifact_dir(workspace, target)
            .join("baseline")
            .join("imports.yaml"),
        data,
    )
}
