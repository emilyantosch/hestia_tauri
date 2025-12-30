#[derive(Debug)]
pub struct ConfigError {
    pub kind: ConfigErrorKind,
}

#[derive(Debug)]
pub enum ConfigErrorKind {
    KeyringError(String),
    AndroidKeystoreError(String),
    EncryptionError(String),
    IoError(std::io::Error),
    SerializationError(String),
    NoCredentialsFound,
}

impl std::fmt::Display for ConfigErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigErrorKind::KeyringError(msg) => write!(f, "KeyringError: {}", msg),
            ConfigErrorKind::AndroidKeystoreError(msg) => {
                write!(f, "AndroidKeystoreError: {}", msg)
            }
            ConfigErrorKind::EncryptionError(msg) => write!(f, "EncryptionError: {}", msg),
            ConfigErrorKind::IoError(msg) => write!(f, "IoError: {}", msg),
            ConfigErrorKind::SerializationError(msg) => write!(f, "SerializationError: {}", msg),
            ConfigErrorKind::NoCredentialsFound => write!(f, "NoCredentialsFound"),
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for ConfigError {}
