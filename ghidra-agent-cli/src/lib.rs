pub mod baseline;
pub mod context;
pub mod execution_log;
pub mod frida;
pub mod gate;
pub mod ghidra;
pub mod git_status;
pub mod help;
pub mod lock;
pub mod paths;
pub mod progress;
pub mod rebuild;
pub mod schema;
pub mod scope;
pub mod third_party;
pub mod verify;
pub mod workspace;

use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Write;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Yaml,
    Json,
    Toml,
}

impl Format {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Toml => "toml",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StructuredError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<BTreeMap<String, String>>,
    pub source: String,
    pub format: String,
}

impl StructuredError {
    pub fn new(code: &str, message: impl Into<String>, source: &str, format: Format) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            details: None,
            source: source.to_string(),
            format: format.as_str().to_string(),
        }
    }

    pub fn with_detail(mut self, key: &str, value: impl Into<String>) -> Self {
        self.details
            .get_or_insert_with(BTreeMap::new)
            .insert(key.to_string(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GhidraAgentCliOutput {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_yaml::Value>,
}

pub fn ok_output(message: &str) -> GhidraAgentCliOutput {
    GhidraAgentCliOutput {
        status: "ok".to_string(),
        message: message.to_string(),
        data: None,
    }
}

pub fn ok_output_with_data(message: &str, data: serde_yaml::Value) -> GhidraAgentCliOutput {
    GhidraAgentCliOutput {
        status: "ok".to_string(),
        message: message.to_string(),
        data: Some(data),
    }
}

pub fn serialize_value<W: Write, T: Serialize>(
    writer: &mut W,
    value: &T,
    format: Format,
) -> Result<()> {
    match format {
        Format::Yaml => {
            let serialized = serde_yaml::to_string(value).context("failed to serialize as YAML")?;
            writer.write_all(serialized.as_bytes())?;
        }
        Format::Json => {
            serde_json::to_writer_pretty(&mut *writer, value)
                .context("failed to serialize as JSON")?;
            writeln!(writer)?;
        }
        Format::Toml => {
            let serialized =
                toml::to_string_pretty(value).context("failed to serialize as TOML")?;
            writer.write_all(serialized.as_bytes())?;
            writeln!(writer)?;
        }
    }
    Ok(())
}

pub fn write_structured_error<W: Write>(
    writer: &mut W,
    error: &StructuredError,
    format: Format,
) -> Result<()> {
    serialize_value(writer, error, format)
}
