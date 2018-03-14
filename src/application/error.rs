use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;
use std::path::PathBuf;
use std::sync::mpsc::SendError;

use serde_yaml::Error as YamlError;

use super::worker::WorkerError;

pub type ApplicationResult<T> = Result<T, ApplicationError>;

#[derive(Debug)]
pub enum ApplicationError {
    SuiteIoError { message: String },
    SuiteYamlError { message: String },
    SuiteIsDirectory { path: PathBuf },
    DirectoryIoError { message: String },
    NoSuitesFound,
    SendMessageError,
    WorkerError { message: String },
}

impl ApplicationError {
    pub fn suite_io_error(error: IoError) -> ApplicationError {
        ApplicationError::SuiteIoError {
            message: format!("{}", error),
        }
    }

    pub fn suite_yaml_error(error: YamlError) -> ApplicationError {
        ApplicationError::SuiteYamlError {
            message: format!("{}", error),
        }
    }

    pub fn suite_is_directory<P>(path: P) -> ApplicationError
    where
        P: Into<PathBuf>,
    {
        ApplicationError::SuiteIsDirectory { path: path.into() }
    }

    pub fn directory_io_error(error: IoError) -> ApplicationError {
        ApplicationError::DirectoryIoError {
            message: format!("{}", error),
        }
    }

    pub fn no_suites_found() -> ApplicationError {
        ApplicationError::NoSuitesFound
    }

    pub fn send_message_error<T>(_: SendError<T>) -> ApplicationError {
        ApplicationError::SendMessageError
    }

    pub fn worker_error(error: WorkerError) -> ApplicationError {
        ApplicationError::WorkerError {
            message: format!("{}", error),
        }
    }
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            ApplicationError::SuiteIoError { ref message } => write!(f, "IO error - {}", message),
            ApplicationError::SuiteYamlError { ref message } => {
                write!(f, "YAML error - {}", message)
            }
            ApplicationError::SuiteIsDirectory { ref path } => {
                write!(f, "Is directory - {}", path.display())
            }
            ApplicationError::DirectoryIoError { ref message } => {
                write!(f, "IO error - {}", message)
            }
            ApplicationError::NoSuitesFound => write!(f, "No suites found"),
            ApplicationError::SendMessageError => write!(f, "Send error, channel already closed"),
            ApplicationError::WorkerError { ref message } => {
                write!(f, "Worker error - {}", message)
            }
        }
    }
}

impl Error for ApplicationError {
    fn description(&self) -> &str {
        match *self {
            ApplicationError::SuiteIoError { .. } => "Suite IO error",
            ApplicationError::SuiteYamlError { .. } => "Suite YAML error",
            ApplicationError::SuiteIsDirectory { .. } => "Suite is directory",
            ApplicationError::DirectoryIoError { .. } => "Directory IO error",
            ApplicationError::NoSuitesFound => "No suites found",
            ApplicationError::SendMessageError => "Send message error",
            ApplicationError::WorkerError { .. } => "Worker error",
        }
    }
}
