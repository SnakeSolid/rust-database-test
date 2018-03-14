mod config;
mod error;

pub use self::config::Configuration;
pub use self::config::DATABASE;
pub use self::config::EXTENSIONS;
pub use self::config::HOSTNAME;
pub use self::config::NWORKERS;
pub use self::config::PASSWORD;
pub use self::config::PORT;
pub use self::config::RECURSIVE;
pub use self::config::SUITES;
pub use self::config::TEXTMODE;
pub use self::config::USERNAME;
pub use self::error::ConfigurationError;
pub use self::error::ConfigurationResult;
