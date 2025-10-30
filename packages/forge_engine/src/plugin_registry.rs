use std::{collections::HashSet, fs, path::Path};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginRegistryError {
    #[error("failed to read registry file {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse registry yaml: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("registry is empty")]
    Empty,
    #[error("duplicate plugin id '{0}' detected")]
    DuplicateId(String),
    #[error("invalid plugin entry '{id}': {message}")]
    InvalidEntry { id: String, message: String },
    #[error("invalid signature format for plugin '{id}': {signature}")]
    InvalidSignatureFormat { id: String, signature: String },
    #[error("unsupported signature algorithm '{algorithm}' for plugin '{id}'")]
    UnsupportedSignatureAlgorithm { id: String, algorithm: String },
    #[error("failed to read entry '{path}' for plugin '{id}': {source}")]
    SignatureIo {
        id: String,
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("signature mismatch for plugin '{id}': expected {expected}, computed {actual}")]
    SignatureMismatch {
        id: String,
        expected: String,
        actual: String,
    },
}

/// Fully validated plugin registry containing descriptors for available tasks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginRegistry {
    pub plugins: Vec<PluginDescriptor>,
}

impl PluginRegistry {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, PluginRegistryError> {
        let path_ref = path.as_ref();
        let contents = fs::read_to_string(path_ref).map_err(|source| PluginRegistryError::Io {
            path: path_ref.display().to_string(),
            source,
        })?;
        Self::from_yaml_str(&contents)
    }

    pub fn from_yaml_str(yaml: &str) -> Result<Self, PluginRegistryError> {
        let raw_plugins: Vec<RawPluginDescriptor> = serde_yaml::from_str(yaml)?;
        if raw_plugins.is_empty() {
            return Err(PluginRegistryError::Empty);
        }

        let mut seen_ids = HashSet::new();
        let mut plugins = Vec::with_capacity(raw_plugins.len());

        for raw in raw_plugins {
            let descriptor = PluginDescriptor::try_from(raw)?;
            if !seen_ids.insert(descriptor.id.clone()) {
                return Err(PluginRegistryError::DuplicateId(descriptor.id));
            }
            plugins.push(descriptor);
        }

        Ok(Self { plugins })
    }

    /// Validates optional plugin signatures against the binaries stored at `base_dir`.
    pub fn validate_signatures<P: AsRef<Path>>(
        &self,
        base_dir: P,
    ) -> Result<(), PluginRegistryError> {
        let base_dir = base_dir.as_ref();
        for plugin in &self.plugins {
            let Some(signature) = plugin.signature.as_ref() else {
                continue;
            };

            let (algorithm, expected_hex) = signature.split_once(':').ok_or_else(|| {
                PluginRegistryError::InvalidSignatureFormat {
                    id: plugin.id.clone(),
                    signature: signature.clone(),
                }
            })?;

            if !algorithm.eq_ignore_ascii_case("sha256") {
                return Err(PluginRegistryError::UnsupportedSignatureAlgorithm {
                    id: plugin.id.clone(),
                    algorithm: algorithm.to_string(),
                });
            }

            let expected_hex = expected_hex.trim();
            if expected_hex.is_empty() {
                return Err(PluginRegistryError::InvalidSignatureFormat {
                    id: plugin.id.clone(),
                    signature: signature.clone(),
                });
            }

            let entry_path = Path::new(&plugin.entry);
            let resolved = if entry_path.is_absolute() {
                entry_path.to_path_buf()
            } else {
                base_dir.join(entry_path)
            };

            let bytes = fs::read(&resolved).map_err(|source| PluginRegistryError::SignatureIo {
                id: plugin.id.clone(),
                path: resolved.display().to_string(),
                source,
            })?;

            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let digest = hasher.finalize();
            let actual_hex: String = digest.iter().map(|byte| format!("{:02x}", byte)).collect();

            if !constant_time_eq(&actual_hex, expected_hex) {
                return Err(PluginRegistryError::SignatureMismatch {
                    id: plugin.id.clone(),
                    expected: expected_hex.to_ascii_lowercase(),
                    actual: actual_hex,
                });
            }
        }

        Ok(())
    }
}

/// A validated plugin entry describing how the engine should load an AI task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginDescriptor {
    pub id: String,
    pub name: String,
    pub entry: String,
    pub input_schema: String,
    pub output_schema: String,
    pub runtimes: Vec<String>,
    pub description: Option<String>,
    pub signature: Option<String>,
}

impl PluginDescriptor {
    fn try_from(raw: RawPluginDescriptor) -> Result<Self, PluginRegistryError> {
        let id = require_non_empty("id", raw.id, None)?;
        let name = require_non_empty("name", raw.name, Some(&id))?;
        let entry = require_non_empty("entry", raw.entry, Some(&id))?;
        let input_schema = require_non_empty("inputSchema", raw.input_schema, Some(&id))?;
        let output_schema = require_non_empty("outputSchema", raw.output_schema, Some(&id))?;

        let runtimes = raw
            .runtimes
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();

        if runtimes.is_empty() {
            return Err(PluginRegistryError::InvalidEntry {
                id: id.clone(),
                message: "runtimes must contain at least one entry".to_string(),
            });
        }

        Ok(Self {
            id,
            name,
            entry,
            input_schema,
            output_schema,
            runtimes,
            description: raw
                .description
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            signature: raw
                .signature
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        })
    }
}

fn require_non_empty(
    field: &str,
    value: Option<String>,
    id: Option<&str>,
) -> Result<String, PluginRegistryError> {
    match value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
    {
        Some(val) => Ok(val),
        None => Err(PluginRegistryError::InvalidEntry {
            id: id.unwrap_or("<unknown>").to_string(),
            message: format!("missing or empty field '{field}'"),
        }),
    }
}

fn constant_time_eq(actual: &str, expected: &str) -> bool {
    let expected_lower = expected.to_ascii_lowercase();
    if actual.len() != expected_lower.len() {
        return false;
    }

    let mut diff: u8 = 0;
    for (a, b) in actual.as_bytes().iter().zip(expected_lower.as_bytes()) {
        diff |= a ^ b;
    }

    diff == 0
}

#[derive(Debug, Deserialize)]
struct RawPluginDescriptor {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    entry: Option<String>,
    #[serde(rename = "inputSchema")]
    #[serde(default)]
    input_schema: Option<String>,
    #[serde(rename = "outputSchema")]
    #[serde(default)]
    output_schema: Option<String>,
    #[serde(default)]
    runtimes: Vec<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    signature: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_REGISTRY: &str = r#"
- id: inspector.v1
  name: "AI Inspector"
  entry: "./plugins/inspector/inspector.wasm"
  inputSchema: "schemas/inspector_input.json"
  outputSchema: "schemas/inspector_output.json"
  runtimes: ["local", "cloud"]
  description: "Analyzes widget tree for potential issues"
- id: layout.optimize.v1
  name: "Layout Optimizer"
  entry: "./plugins/layout/layout.wasm"
  inputSchema: "schemas/layout_input.json"
  outputSchema: "schemas/layout_output.json"
  runtimes:
    - "local"
"#;

    #[test]
    fn parses_sample_registry() {
        let registry = PluginRegistry::from_yaml_str(SAMPLE_REGISTRY).expect("registry parsed");
        assert_eq!(registry.plugins.len(), 2);
        let inspector = &registry.plugins[0];
        assert_eq!(inspector.id, "inspector.v1");
        assert_eq!(inspector.entry, "./plugins/inspector/inspector.wasm");
        assert_eq!(
            inspector.runtimes,
            vec!["local".to_string(), "cloud".to_string()]
        );
        assert_eq!(
            inspector.description.as_deref(),
            Some("Analyzes widget tree for potential issues")
        );
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let yaml = r#"
- id: duplicate
  name: A
  entry: a
  inputSchema: a.json
  outputSchema: b.json
  runtimes: ["local"]
- id: duplicate
  name: B
  entry: b
  inputSchema: c.json
  outputSchema: d.json
  runtimes: ["cloud"]
"#;
        let err = PluginRegistry::from_yaml_str(yaml).unwrap_err();
        assert!(matches!(
            err,
            PluginRegistryError::DuplicateId(ref id) if id == "duplicate"
        ));
    }

    #[test]
    fn missing_fields_are_rejected() {
        let yaml = r#"
- id: missing
  entry: only
  runtimes: ["local"]
"#;
        let err = PluginRegistry::from_yaml_str(yaml).unwrap_err();
        match err {
            PluginRegistryError::InvalidEntry { message, .. } => {
                assert!(message.contains("name"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn empty_registry_is_rejected() {
        let err = PluginRegistry::from_yaml_str("[]").unwrap_err();
        assert!(matches!(err, PluginRegistryError::Empty));
    }

    #[test]
    fn signature_is_validated() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let plugin_path = temp_dir.path().join("plugin.wasm");
        std::fs::write(&plugin_path, b"plugin-bytes").expect("write plugin");

        let mut hasher = Sha256::new();
        hasher.update(b"plugin-bytes");
        let digest = hasher.finalize();
        let expected = digest
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();

        let yaml = format!(
            "- id: sig\n  name: Sig\n  entry: {}\n  inputSchema: a.json\n  outputSchema: b.json\n  runtimes: [\"local\"]\n  signature: \"sha256:{}\"\n",
            plugin_path.to_string_lossy(),
            expected
        );

        let registry = PluginRegistry::from_yaml_str(&yaml).expect("parse");
        registry
            .validate_signatures(temp_dir.path())
            .expect("signature valid");
    }

    #[test]
    fn signature_mismatch_is_reported() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let plugin_path = temp_dir.path().join("plugin.wasm");
        std::fs::write(&plugin_path, b"plugin-bytes").expect("write plugin");

        let yaml = format!(
            "- id: sig\n  name: Sig\n  entry: {}\n  inputSchema: a.json\n  outputSchema: b.json\n  runtimes: [\"local\"]\n  signature: \"sha256:deadbeef\"\n",
            plugin_path.to_string_lossy()
        );

        let registry = PluginRegistry::from_yaml_str(&yaml).expect("parse");
        let err = registry
            .validate_signatures(temp_dir.path())
            .expect_err("should fail");
        assert!(matches!(err, PluginRegistryError::SignatureMismatch { .. }));
    }
}
