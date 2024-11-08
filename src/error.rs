use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrbitError {
    #[error("Docker error: {0}")]
    Docker(#[from] gadget_sdk::docker::bollard::errors::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Deployment error: {0}")]
    Deployment(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Command execution error: {0}")]
    Command(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Contract deployment error: {0}")]
    ContractDeployment(String),
}

pub type Result<T> = std::result::Result<T, OrbitError>;

impl From<OrbitError> for gadget_sdk::Error {
    fn from(val: OrbitError) -> Self {
        match val {
            OrbitError::Docker(e) => gadget_sdk::Error::Docker(e),
            OrbitError::Config(e) => gadget_sdk::Error::Other(format!("Config error: {}", e)),
            OrbitError::Deployment(e) => {
                gadget_sdk::Error::Other(format!("Deployment error: {}", e))
            }
            OrbitError::Io(e) => gadget_sdk::Error::IoError(e),
            OrbitError::Json(e) => gadget_sdk::Error::Json(e),
            OrbitError::Command(e) => gadget_sdk::Error::Other(format!("Command error: {}", e)),
            OrbitError::FileSystem(e) => {
                gadget_sdk::Error::Other(format!("FileSystem error: {}", e))
            }
            OrbitError::ContractDeployment(e) => {
                gadget_sdk::Error::Other(format!("Contract deployment error: {}", e))
            }
        }
    }
}
