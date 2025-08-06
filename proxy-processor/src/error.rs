use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessorError {
    #[error("Unsupported protocol: {0}")]
    UnsupportedProtocol(String),
    #[error("Invalid link format: {0}")]
    InvalidLinkFormat(String),
    #[error("Failed to decode Base64 for link: {0}")]
    Base64DecodeFailed(String),
    #[error("Failed to parse JSON from Base64 for link: {0}")]
    JsonParseFailed(String),
    #[error("GeoIP database error: {0}")]
    GeoIpDbError(String),
    #[error("Failed to parse decoded proxy link: {0}")]
    UrlParse(String),
    #[error("Missing component in url: {0}")]
    MissingComponent(&'static str),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("Protocol not supported by this exporter: {0:?}")]
    UnsupportedProtocol(String),
    #[error("Failed to serialize to JSON: {0}")]
    JsonSerializationFailed(#[from] serde_json::Error),
}
