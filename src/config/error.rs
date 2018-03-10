use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::num::ParseIntError;

pub type ConfigurationResult<T> = Result<T, ConfigurationError>;

#[derive(Debug)]
pub enum ConfigurationError {
    EmptyHostname,
    EmptyPort,
    WrongPort,
    EmptyDatabase,
    EmptyUsername,
    EmptyPassword,
    EmptySuites,
}

impl ConfigurationError {
    pub fn wrong_port(_: ParseIntError) -> ConfigurationError {
        ConfigurationError::WrongPort
    }
}

impl Display for ConfigurationError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            ConfigurationError::EmptyHostname => write!(f, "Empty host name"),
            ConfigurationError::EmptyPort => write!(f, "Empty port"),
            ConfigurationError::WrongPort => write!(f, "Wrong port"),
            ConfigurationError::EmptyDatabase => write!(f, "Empty database"),
            ConfigurationError::EmptyUsername => write!(f, "Empty user name"),
            ConfigurationError::EmptyPassword => write!(f, "Empty password"),
            ConfigurationError::EmptySuites => write!(f, "Empty suites"),
        }
    }
}
