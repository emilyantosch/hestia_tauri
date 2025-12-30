use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};

use argon2::{password_hash::SaltString, Argon2};

use secrecy::{ExposeSecret, SecretString};

use serde::{Deserialize, Serialize};

use sea_orm::sqlx::sqlite::{SqliteJournalMode, SqliteSynchronous};

use crate::errors::{ConfigError, ConfigErrorKind, DbError};

pub struct DatabaseSettings {
    pub db_type: DatabaseType,
    pub sqlite_config: Option<SqliteConfig>,
    pub postgres_config: Option<PostgresConfig>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DatabaseType {
    Sqlite,
    Postgres,
    None,
}

#[derive(Clone, Debug)]
pub struct SqliteConfig {
    pub con_string: String,
    pub create_if_missing: bool,
    pub connection_timeout_ms: u32,
    pub journal_mode: SqliteJournalMode,
    pub synchronous: SqliteSynchronous,
}

#[derive(Serialize, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    #[serde(skip)]
    pub password: secrecy::SecretString,
    pub ssl_mode: PostgresSslMode,
    pub connection_timeout_ms: u32,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_ms: u32,
    pub idle_timeout_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostgresSslMode {
    ///All of the data is sent as plain text, no ssl encryption
    Disable,
    ///Try no SSL, switch to it, if the server demands it
    Allow,
    ///Try SSL first, fallback to plain text if the server doesn't support it
    Prefer,
    ///SSL is mandatory, break connection if not supported by server
    Require,
    ///SSL required and also check CA certificates are from reputable source
    VerifyCa,
    ///SSL required, CA certificates and hostnames are checked
    VerifyFull,
}

#[derive(Serialize, Deserialize)]
struct EncryptedConfig {
    encrypted_data: Vec<u8>,
    nonce: Vec<u8>,
    salt: Vec<u8>,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            db_type: DatabaseType::None,
            sqlite_config: None,
            postgres_config: None,
        }
    }
}

// Custom Debug implementation to hide sensitive data
impl std::fmt::Debug for DatabaseSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseSettings")
            .field("db_type", &self.db_type)
            .field("sqlite_config", &self.sqlite_config)
            .field(
                "postgres_config",
                &self
                    .postgres_config
                    .as_ref()
                    .map(|_| "PostgresConfig { [REDACTED] }"),
            )
            .finish()
    }
}

impl std::fmt::Debug for PostgresConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("ssl_mode", &self.ssl_mode)
            .field("connection_timeout_ms", &self.connection_timeout_ms)
            .field("max_connections", &self.max_connections)
            .field("min_connections", &self.min_connections)
            .field("acquire_timeout_ms", &self.acquire_timeout_ms)
            .field("idle_timeout_ms", &self.idle_timeout_ms)
            .finish()
    }
}

impl PostgresConfig {
    ///Layered loading of all data for the connection to postgres database
    pub async fn load_layered(service: &str, username: &str) -> Result<Self, ConfigError> {
        //Layer 1: Fetch all data from secure storage
        if let Ok(config) = Self::from_secure_storage(service, username).await {
            return Ok(config);
        }
        //Layer 2: Get all data from a secure config file with secure password
        if let Ok(config) = Self::from_config_file(service, username).await {
            return Ok(config);
        }
        //Layer 3: Retrieve data from environment variables (dev mode only)
        if let Ok(config) = Self::from_env().await {
            return Ok(config);
        }
        Err(ConfigError {
            kind: ConfigErrorKind::NoCredentialsFound,
        })
    }

    ///Load credentials using the most secure method available, depending on the operating system
    async fn from_secure_storage(service: &str, username: &str) -> Result<Self, ConfigError> {
        cfg_if::cfg_if! {
            if #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux", target_os = "ios"))] {
                Self::from_keyring(service, username).await
            } else if #[cfg(target_os = "android")] {
                Self::from_android_keystore(service, username)
                .or_else(|_| Self::from_encrypted_file_with_prompt(service))
            } else {
                Self::from_encrypted_file_with_prompt(service)
            }
        }
    }

    async fn from_config_file(service: &str, master_password: &str) -> Result<Self, ConfigError> {
        let config_path = Self::get_config_file_path(service).await?;
        if !config_path.exists() {
            return Err(ConfigError {
                kind: ConfigErrorKind::NoCredentialsFound,
            });
        }
        Self::from_encrypted_file(&config_path, master_password).await
    }

    /// Load from environment variables (fallback method)
    async fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            host: std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            database: std::env::var("DB_NAME")?,
            username: std::env::var("DB_USER")?,
            password: secrecy::SecretString::new(std::env::var("DB_PASSWORD")?.into()),
            ssl_mode: match std::env::var("DB_SSL_MODE").as_deref() {
                Ok("disable") => PostgresSslMode::Disable,
                Ok("require") => PostgresSslMode::Require,
                Ok("verify-full") => PostgresSslMode::VerifyFull,
                _ => PostgresSslMode::Prefer,
            },
            ..Self::default()
        })
    }

    async fn save_layered(&self, service: &str, master_password: &str) -> Result<(), ConfigError> {
        //Save the current config to the secured config file
        self.save_to_encrypted_file(service, master_password)
            .await?;

        self.save_secure_storage(service, &self.username).await?;

        Ok(())
    }

    async fn get_config_file_path(service: &str) -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));

        Ok(config_dir.join(format!("{}_hestia_db_config.toml", service)))
    }

    pub async fn save_secure_storage(
        &self,
        service: &str,
        username: &str,
    ) -> Result<(), ConfigError> {
        cfg_if::cfg_if! {
            if #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux", target_os = "ios"))] {
                self.save_to_keyring(service, username).await?
            } else if #[cfg(target_os = "android")] {
                self.save_to_android_keystore(service, username).await?
            }
        }
        Err(ConfigError {
            kind: ConfigErrorKind::KeyringError(format!(
                "Could not store config to secure storage"
            )),
        })
    }

    #[cfg(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "ios"
    ))]
    pub async fn from_keyring(service: &str, username: &str) -> Result<Self, ConfigError> {
        use keyring::Entry;

        let entry = Entry::new(service, username).map_err(|e| ConfigError {
            kind: ConfigErrorKind::KeyringError(format!("Failed to create keyring entry: {}", e)),
        })?;

        let stored_data = entry.get_password().map_err(|e| ConfigError {
            kind: ConfigErrorKind::KeyringError(format!(
                "Failed to get stored data: {}",
                e.to_string()
            )),
        })?;

        if let Ok(config) = serde_json::from_str::<Self>(&stored_data) {
            return Ok(config);
        }

        Err(ConfigError {
            kind: ConfigErrorKind::KeyringError(format!("Could not extract values from keyring")),
        })
    }

    #[cfg(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "ios"
    ))]
    async fn save_to_keyring(&self, service: &str, username: &str) -> Result<(), ConfigError> {
        use keyring::Entry;

        let entry = Entry::new(service, username).map_err(|e| ConfigError {
            kind: ConfigErrorKind::KeyringError(format!("Failed to create keyring entry: {}", e)),
        })?;

        // Store full configuration as JSON
        let config_json = serde_json::to_string(self).map_err(|e| ConfigError {
            kind: ConfigErrorKind::SerializationError(e.to_string()),
        })?;

        entry.set_password(&config_json).map_err(|e| ConfigError {
            kind: ConfigErrorKind::KeyringError(format!("Failed to set configuration: {}", e)),
        })?;

        Ok(())
    }

    #[cfg(target_os = "android")]
    async fn save_to_android_keystore(
        &self,
        service: &str,
        username: &str,
    ) -> Result<(), ConfigError> {
        Self::set_android_keystore_password(service, username, self.password.expose_secret())
    }

    #[cfg(target_os = "android")]
    async fn from_android_keystore(service: &str, username: &str) -> Result<Self, ConfigError> {
        // This would require JNI setup and Android Keystore integration
        // For now, we'll provide a skeleton implementation

        // In a real implementation, you would:
        // 1. Get the JNI environment
        // 2. Call Android Keystore APIs to retrieve the password
        // 3. Decrypt the stored credential

        let password = Self::get_android_keystore_password(service, username)?;

        Ok(Self {
            host: std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            database: std::env::var("DB_NAME").unwrap_or_else(|_| "app".to_string()),
            username: username.to_string(),
            password: Secret::new(password),
            ssl_mode: PostgresSslMode::Prefer,
            ..Self::default()
        })
    }

    #[cfg(target_os = "android")]
    async fn get_android_keystore_password(
        service: &str,
        username: &str,
    ) -> Result<String, ConfigError> {
        // This is a simplified example. In a real implementation, you would:
        // 1. Use JNI to call into Android Keystore
        // 2. Retrieve and decrypt the stored password

        // Example JNI call structure (requires proper JNI setup):
        /*
        let vm = JavaVM::from_raw(java_vm_ptr)?;
        let env = vm.attach_current_thread()?;

        let keystore_class = env.find_class("java/security/KeyStore")?;
        let keystore = env.call_static_method(
            keystore_class,
            "getInstance",
            "(Ljava/lang/String;)Ljava/security/KeyStore;",
            &[JValue::Object(env.new_string("AndroidKeyStore")?.into())]
        )?;

        // Load keystore, get key, decrypt password...
        */

        Err(ConfigError::AndroidKeystoreError(
            "Android Keystore integration requires JNI setup".to_string(),
        ))
    }

    #[cfg(target_os = "android")]
    async fn set_android_keystore_password(
        service: &str,
        username: &str,
        password: &str,
    ) -> Result<(), ConfigError> {
        // Similar to get_android_keystore_password, but for storing
        Err(ConfigError::AndroidKeystoreError(
            "Android Keystore integration requires JNI setup".to_string(),
        ))
    }

    async fn from_encrypted_file(
        path: &PathBuf,
        master_password: &str,
    ) -> Result<Self, ConfigError> {
        let encrypted_data = tokio::fs::read(path).await.map_err(|e| ConfigError {
            kind: ConfigErrorKind::IoError(e),
        })?;

        let encrypted_config: EncryptedConfig =
            serde_json::from_slice(&encrypted_data).map_err(|e| ConfigError {
                kind: ConfigErrorKind::SerializationError(e.to_string()),
            })?;

        let argon2 = Argon2::default();

        let salt = SaltString::from_b64(&String::from_utf8_lossy(&encrypted_config.salt)).map_err(
            |e| ConfigError {
                kind: ConfigErrorKind::EncryptionError(format!("Invalid Salt: {}", e.to_string())),
            },
        )?;

        let mut key_bytes = [0u8; 32];
        let mut salt_buffer = Vec::new();
        let salt_bytes = salt.decode_b64(&mut salt_buffer).map_err(|e| ConfigError {
            kind: ConfigErrorKind::EncryptionError(e.to_string()),
        })?;
        argon2
            .hash_password_into(master_password.as_bytes(), salt_bytes, &mut key_bytes)
            .map_err(|e| ConfigError {
                kind: ConfigErrorKind::EncryptionError(e.to_string()),
            })?;

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let nonce = Nonce::from_slice(&encrypted_config.nonce);
        let decrypted_data = cipher
            .decrypt(nonce, encrypted_config.encrypted_data.as_ref())
            .map_err(|e| ConfigError {
                kind: ConfigErrorKind::SerializationError(e.to_string()),
            })?;

        let mut config: PostgresConfig =
            serde_json::from_slice(&decrypted_data).map_err(|e| ConfigError {
                kind: ConfigErrorKind::SerializationError(format!(
                    "Deserialization into JSON failed: {}",
                    e.to_string()
                )),
            })?;

        if let Ok(password_str) = serde_json::from_slice::<String>(&decrypted_data) {
            config.password = SecretString::new(password_str.into());
        }

        Ok(config)
    }

    /// Save configuration to encrypted file with provided password
    async fn save_to_encrypted_file(
        &self,
        service: &str,
        master_password: &str,
    ) -> Result<(), ConfigError> {
        let config_path = Self::get_config_file_path(service).await?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| ConfigError {
                    kind: ConfigErrorKind::IoError(e),
                })?;
        }

        // Generate random salt and nonce
        let salt = SaltString::try_from_rng(&mut rand::rngs::OsRng).map_err(|e| ConfigError {
            kind: ConfigErrorKind::EncryptionError(e.to_string()),
        })?;

        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Derive encryption key from master password
        let argon2 = Argon2::default();
        let mut key_bytes = [0u8; 32];
        let mut salt_buffer = Vec::new();
        let salt_bytes = salt.decode_b64(&mut salt_buffer).map_err(|e| ConfigError {
            kind: ConfigErrorKind::EncryptionError(e.to_string()),
        })?;
        argon2
            .hash_password_into(master_password.as_bytes(), salt_bytes, &mut key_bytes)
            .map_err(|e| ConfigError {
                kind: ConfigErrorKind::EncryptionError(format!("Key derivation failed: {}", e)),
            })?;

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        // Serialize config (excluding password for now, we'll handle it separately)
        let config_data = serde_json::to_vec(&self).map_err(|e| ConfigError {
            kind: ConfigErrorKind::SerializationError(e.to_string()),
        })?;

        // Encrypt the data
        let encrypted_data =
            cipher
                .encrypt(nonce, config_data.as_ref())
                .map_err(|e| ConfigError {
                    kind: ConfigErrorKind::EncryptionError(format!("Encryption failed: {}", e)),
                })?;

        // Create encrypted config structure
        let encrypted_config = EncryptedConfig {
            encrypted_data,
            nonce: nonce_bytes.to_vec(),
            salt: salt.as_str().as_bytes().to_vec(),
        };

        // Save to file
        let encrypted_json =
            serde_json::to_vec_pretty(&encrypted_config).map_err(|e| ConfigError {
                kind: ConfigErrorKind::SerializationError(e.to_string()),
            })?;

        tokio::fs::write(&config_path, encrypted_json)
            .await
            .map_err(|e| ConfigError {
                kind: ConfigErrorKind::IoError(e),
            })?;

        Ok(())
    }
}

impl Default for SqliteConfig {
    fn default() -> Self {
        // Use a default path relative to the app directory
        let preamble = String::from("sqlite://");
        let app_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .to_string();
        let suffix = String::from("/main.sqlite?mode=rw");

        let con_string = format!("{}{}{}", preamble, app_dir, suffix);
        println!("{}", con_string);

        Self {
            con_string,
            create_if_missing: true,
            connection_timeout_ms: 30000,
            journal_mode: SqliteJournalMode::Wal,
            synchronous: SqliteSynchronous::Normal,
        }
    }
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "app".to_string(),
            username: "postgres".to_string(),
            password: SecretString::new("".to_string().into()),
            ssl_mode: PostgresSslMode::Prefer,
            connection_timeout_ms: 10000,
            max_connections: 10,
            min_connections: 1,
            acquire_timeout_ms: 5000,
            idle_timeout_ms: Some(600000), // 10 minutes
        }
    }
}

impl DatabaseSettings {
    pub fn new_sqlite(config: SqliteConfig) -> Self {
        Self {
            db_type: DatabaseType::Sqlite,
            sqlite_config: Some(config),
            postgres_config: None,
        }
    }

    pub fn new_postgres(config: PostgresConfig) -> Self {
        Self {
            db_type: DatabaseType::Postgres,
            sqlite_config: None,
            postgres_config: Some(config),
        }
    }

    pub fn get_connection_string(&self) -> Result<String> {
        match self.db_type {
            DatabaseType::Sqlite => {
                if let Some(config) = &self.sqlite_config {
                    Ok(config.con_string.clone())
                } else {
                    Err(DbError::ConfigurationError)?
                }
            }
            DatabaseType::Postgres => {
                if let Some(config) = &self.postgres_config {
                    let ssl_param = match config.ssl_mode {
                        PostgresSslMode::Disable => "sslmode=disable",
                        PostgresSslMode::Allow => "sslmode=allow",
                        PostgresSslMode::Prefer => "sslmode=prefer",
                        PostgresSslMode::Require => "sslmode=require",
                        PostgresSslMode::VerifyCa => "sslmode=verify-ca",
                        PostgresSslMode::VerifyFull => "sslmode=verify-full",
                    };

                    Ok(format!(
                        "postgresql://{}:{}@{}:{}/{}?{}",
                        config.username,
                        config.password.expose_secret(),
                        config.host,
                        config.port,
                        config.database,
                        ssl_param
                    ))
                } else {
                    Err(DbError::ConfigurationError)?
                }
            }
            DatabaseType::None => {
                Err(DbError::ConfigurationError).context("The is no database type selected!")
            }
        }
    }

    pub fn is_configured(&self) -> bool {
        match self.db_type {
            DatabaseType::Sqlite => self.sqlite_config.is_some(),
            DatabaseType::Postgres => self.postgres_config.is_some(),
            DatabaseType::None => false,
        }
    }
}
