use std::fmt;

#[derive(Debug)]
pub enum LlmError {
    NetworkError(String),
    ParseError(String),
    FileError(String),
    ConfigError(String),
    TimeoutError(String),
    SerializationError(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            LlmError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LlmError::FileError(msg) => write!(f, "File error: {}", msg),
            LlmError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            LlmError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            LlmError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for LlmError {}

impl From<LlmError> for String {
    fn from(error: LlmError) -> Self {
        error.to_string()
    }
}

pub type LlmResult<T> = Result<T, LlmError>;

// Helper traits for better error conversion
pub trait IntoLlmError<T> {
    fn map_network_err(self, context: &str) -> LlmResult<T>;
    fn map_parse_err(self, context: &str) -> LlmResult<T>;
    #[allow(dead_code)]
    fn map_file_err(self, context: &str) -> LlmResult<T>;
    #[allow(dead_code)]
    fn map_config_err(self, context: &str) -> LlmResult<T>;
    #[allow(dead_code)]
    fn map_serialization_err(self, context: &str) -> LlmResult<T>;
}

impl<T, E: std::error::Error> IntoLlmError<T> for Result<T, E> {
    fn map_network_err(self, context: &str) -> LlmResult<T> {
        self.map_err(|e| LlmError::NetworkError(format!("{}: {}", context, e)))
    }

    fn map_parse_err(self, context: &str) -> LlmResult<T> {
        self.map_err(|e| LlmError::ParseError(format!("{}: {}", context, e)))
    }

    fn map_file_err(self, context: &str) -> LlmResult<T> {
        self.map_err(|e| LlmError::FileError(format!("{}: {}", context, e)))
    }

    fn map_config_err(self, context: &str) -> LlmResult<T> {
        self.map_err(|e| LlmError::ConfigError(format!("{}: {}", context, e)))
    }

    fn map_serialization_err(self, context: &str) -> LlmResult<T> {
        self.map_err(|e| LlmError::SerializationError(format!("{}: {}", context, e)))
    }
}
