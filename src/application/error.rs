use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;
use std::sync::mpsc::SendError;

use serde_yaml::Error as YamlError;

use super::worker::WorkerError;

pub type ApplicationResult<T> = Result<T, ApplicationError>;

#[derive(Debug)]
pub enum ApplicationError {
    SuiteIoError { message: String },
    SuiteYamlError { message: String },
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
            ApplicationError::SendMessageError => "Send message error",
            ApplicationError::WorkerError { .. } => "Worker error",
        }
    }
}
