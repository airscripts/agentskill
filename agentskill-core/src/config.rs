use std::fmt::Display;
use std::path::Path;

use serde::Deserialize;

const CONFIG_FILE: &str = "agentskill.toml";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigSource {
    Default,
    File,
}

impl ConfigSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::File => CONFIG_FILE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatureMode {
    Auto,
    On,
    Off,
}

impl SignatureMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::On => "on",
            Self::Off => "off",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryConfig {
    pub signature: bool,
    pub source: ConfigSource,
    pub valid: bool,
    pub error: Option<String>,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            signature: true,
            source: ConfigSource::Default,
            valid: true,
            error: None,
        }
    }
}

impl RepositoryConfig {
    pub fn resolved_signature(&self, mode: SignatureMode) -> bool {
        match mode {
            SignatureMode::Auto => self.signature,
            SignatureMode::On => true,
            SignatureMode::Off => false,
        }
    }

    fn invalid(error: impl Display) -> Self {
        Self {
            valid: false,
            error: Some(error.to_string()),
            ..Self::default()
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    #[serde(default = "default_signature")]
    signature: bool,
}

const fn default_signature() -> bool {
    true
}

pub fn load(root: &Path) -> RepositoryConfig {
    let path = root.join(CONFIG_FILE);
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Default::default(),
        Err(error) => return RepositoryConfig::invalid(error),
    };

    match toml::from_str::<FileConfig>(&content) {
        Ok(config) => RepositoryConfig {
            signature: config.signature,
            source: ConfigSource::File,
            valid: true,
            error: None,
        },
        Err(error) => RepositoryConfig::invalid(error),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{ConfigSource, SignatureMode, load};

    #[test]
    fn absent_configuration_uses_enabled_default() {
        let directory = tempdir().unwrap();
        let config = load(directory.path());

        assert!(config.signature);
        assert_eq!(config.source, ConfigSource::Default);
        assert!(config.valid);
        assert!(!config.resolved_signature(SignatureMode::Off));
    }

    #[test]
    fn reads_signature_and_rejects_unknown_keys() {
        let directory = tempdir().unwrap();
        fs::write(
            directory.path().join("agentskill.toml"),
            "signature = false\n",
        )
        .unwrap();

        let config = load(directory.path());
        assert!(!config.signature);
        assert_eq!(config.source, ConfigSource::File);
        assert!(config.valid);

        fs::write(
            directory.path().join("agentskill.toml"),
            "signature = true\n",
        )
        .unwrap();

        let config = load(directory.path());
        assert!(config.signature);
        assert_eq!(config.source, ConfigSource::File);
        assert!(config.valid);

        fs::write(
            directory.path().join("agentskill.toml"),
            "signature = false\nextra = true\n",
        )
        .unwrap();

        let config = load(directory.path());
        assert!(config.signature);
        assert_eq!(config.source, ConfigSource::Default);
        assert!(!config.valid);
        assert!(config.error.is_some());

        fs::write(directory.path().join("agentskill.toml"), "signature = ").unwrap();

        let config = load(directory.path());
        assert!(config.signature);
        assert_eq!(config.source, ConfigSource::Default);
        assert!(!config.valid);
        assert!(config.error.is_some());
    }
}
