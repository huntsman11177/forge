use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::PluginDescriptor;

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("sandbox base directory not found: {path}")]
    BaseDirectoryMissing { path: String },
    #[error("sandbox base directory is not a directory: {path}")]
    BaseDirectoryNotDirectory { path: String },
    #[error("failed to access sandbox base directory {path}: {source}")]
    BaseDirectoryIo {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("plugin '{plugin_id}' entry path uses absolute path but sandbox forbids it: {path}")]
    AbsolutePathDisallowed { plugin_id: String, path: String },
    #[error("plugin '{plugin_id}' entry not found at {path}")]
    EntryMissing { plugin_id: String, path: String },
    #[error("plugin '{plugin_id}' entry is not a file: {path}")]
    EntryNotFile { plugin_id: String, path: String },
    #[error("plugin '{plugin_id}' entry escapes sandbox (base: {base}): {path}")]
    EntryOutsideSandbox {
        plugin_id: String,
        base: String,
        path: String,
    },
    #[error("failed to access plugin '{plugin_id}' entry {path}: {source}")]
    EntryIo {
        plugin_id: String,
        path: String,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone)]
pub struct PluginSandbox {
    base_dir: PathBuf,
    allow_absolute_paths: bool,
}

impl PluginSandbox {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self, SandboxError> {
        Self::with_options(base_dir, false)
    }

    pub fn with_options<P: AsRef<Path>>(
        base_dir: P,
        allow_absolute_paths: bool,
    ) -> Result<Self, SandboxError> {
        let base_dir_ref = base_dir.as_ref();
        let metadata = fs::metadata(base_dir_ref).map_err(|err| match err.kind() {
            ErrorKind::NotFound => SandboxError::BaseDirectoryMissing {
                path: base_dir_ref.display().to_string(),
            },
            _ => SandboxError::BaseDirectoryIo {
                path: base_dir_ref.display().to_string(),
                source: err,
            },
        })?;

        if !metadata.is_dir() {
            return Err(SandboxError::BaseDirectoryNotDirectory {
                path: base_dir_ref.display().to_string(),
            });
        }

        let canonical =
            base_dir_ref
                .canonicalize()
                .map_err(|source| SandboxError::BaseDirectoryIo {
                    path: base_dir_ref.display().to_string(),
                    source,
                })?;

        Ok(Self {
            base_dir: canonical,
            allow_absolute_paths,
        })
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Resolves the plugin entry path within the sandbox, ensuring it exists and is a file.
    pub fn resolve_entry(&self, plugin: &PluginDescriptor) -> Result<PathBuf, SandboxError> {
        let entry_path = Path::new(&plugin.entry);
        let candidate = if entry_path.is_absolute() {
            if !self.allow_absolute_paths {
                return Err(SandboxError::AbsolutePathDisallowed {
                    plugin_id: plugin.id.clone(),
                    path: plugin.entry.clone(),
                });
            }
            entry_path.to_path_buf()
        } else {
            self.base_dir.join(entry_path)
        };

        let metadata = fs::metadata(&candidate).map_err(|err| match err.kind() {
            ErrorKind::NotFound => SandboxError::EntryMissing {
                plugin_id: plugin.id.clone(),
                path: candidate.display().to_string(),
            },
            _ => SandboxError::EntryIo {
                plugin_id: plugin.id.clone(),
                path: candidate.display().to_string(),
                source: err,
            },
        })?;

        if !metadata.is_file() {
            return Err(SandboxError::EntryNotFile {
                plugin_id: plugin.id.clone(),
                path: candidate.display().to_string(),
            });
        }

        let canonical = candidate
            .canonicalize()
            .map_err(|source| SandboxError::EntryIo {
                plugin_id: plugin.id.clone(),
                path: candidate.display().to_string(),
                source,
            })?;

        if !self.allow_absolute_paths && !canonical.starts_with(&self.base_dir) {
            return Err(SandboxError::EntryOutsideSandbox {
                plugin_id: plugin.id.clone(),
                base: self.base_dir.display().to_string(),
                path: canonical.display().to_string(),
            });
        }

        Ok(canonical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_descriptor(entry: &str) -> PluginDescriptor {
        PluginDescriptor {
            id: "plugin".to_string(),
            name: "Plugin".to_string(),
            entry: entry.to_string(),
            input_schema: "in.json".to_string(),
            output_schema: "out.json".to_string(),
            runtimes: vec!["local".to_string()],
            description: None,
            signature: None,
        }
    }

    #[test]
    fn resolves_relative_file_inside_sandbox() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let base = temp_dir.path().join("sandbox");
        fs::create_dir_all(&base).expect("create base");
        let plugin_file = base.join("plugins").join("plugin.wasm");
        fs::create_dir_all(plugin_file.parent().unwrap()).expect("create plugin dir");
        fs::write(&plugin_file, b"wasm").expect("write plugin file");

        let sandbox = PluginSandbox::new(&base).expect("sandbox");
        let descriptor = make_descriptor("plugins/plugin.wasm");
        let resolved = sandbox.resolve_entry(&descriptor).expect("resolve");
        assert_eq!(resolved, plugin_file.canonicalize().unwrap());
    }

    #[test]
    fn rejects_missing_files() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let base = temp_dir.path().join("sandbox");
        fs::create_dir_all(&base).expect("create base");

        let sandbox = PluginSandbox::new(&base).expect("sandbox");
        let descriptor = make_descriptor("plugins/missing.wasm");
        let err = sandbox.resolve_entry(&descriptor).unwrap_err();
        assert!(matches!(err, SandboxError::EntryMissing { .. }));
    }

    #[test]
    fn rejects_escape_outside_sandbox() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let base = temp_dir.path().join("sandbox");
        let outside = temp_dir.path().join("outside");
        fs::create_dir_all(&base).expect("create base");
        fs::create_dir_all(&outside).expect("create outside");
        let outside_file = outside.join("plugin.wasm");
        fs::write(&outside_file, b"wasm").expect("write outside plugin");

        let sandbox = PluginSandbox::new(&base).expect("sandbox");
        let descriptor = make_descriptor("../outside/plugin.wasm");
        let err = sandbox.resolve_entry(&descriptor).unwrap_err();
        assert!(matches!(err, SandboxError::EntryOutsideSandbox { .. }));
    }

    #[test]
    fn absolute_paths_require_explicit_opt_in() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let base = temp_dir.path().join("sandbox");
        fs::create_dir_all(&base).expect("create base");
        let absolute_plugin = temp_dir.path().join("plugin.wasm");
        fs::write(&absolute_plugin, b"wasm").expect("write plugin");

        let sandbox = PluginSandbox::new(&base).expect("sandbox");
        let descriptor = make_descriptor(absolute_plugin.to_str().unwrap());
        let err = sandbox.resolve_entry(&descriptor).unwrap_err();
        assert!(matches!(err, SandboxError::AbsolutePathDisallowed { .. }));

        let sandbox = PluginSandbox::with_options(&base, true).expect("sandbox");
        let resolved = sandbox.resolve_entry(&descriptor).expect("resolve");
        assert_eq!(resolved, absolute_plugin.canonicalize().unwrap());
    }

    #[test]
    fn base_directory_must_exist() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let base = temp_dir.path().join("missing");
        let err = PluginSandbox::new(&base).unwrap_err();
        assert!(matches!(err, SandboxError::BaseDirectoryMissing { .. }));
    }
}
