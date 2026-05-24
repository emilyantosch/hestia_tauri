use crate::SerializableError;
use thiserror::Error;

#[derive(Debug, Error, SerializableError)]
pub enum ConfigError {
    #[error("Keyring error: {0}")]
    KeyringError(String),
    #[error("Android keystore error: {0}")]
    AndroidKeystoreError(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("No credentials found")]
    NoCredentialsFound,
}
