use std::path::PathBuf;

use clap::ArgMatches;

mod error;

pub use self::error::ConfigurationError;
pub use self::error::ConfigurationResult;

pub const HOSTNAME: &str = "HOSTNAME";
pub const PORT: &str = "PORT";
pub const DATABASE: &str = "DATABASE";
pub const USERNAME: &str = "USERNAME";
pub const PASSWORD: &str = "PASSWORD";
pub const NWORKERS: &str = "NWORKERS";
pub const RECURSIVE: &str = "RECURSIVE";
pub const EXTENSIONS: &str = "EXTENSIONS";
pub const FILTER: &str = "FILTER";
pub const TEXTMODE: &str = "TEXTMODE";
pub const SUITES: &str = "SUITES";

#[derive(Debug)]
pub struct Configuration {
    hostname: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    n_workers: usize,
    recursive: bool,
    extensions: Option<Vec<String>>,
    filter: Option<String>,
    text_mode: bool,
    suites: Vec<PathBuf>,
}

impl Configuration {
    pub fn from_matches(matches: ArgMatches) -> ConfigurationResult<Configuration> {
        Ok(Configuration {
            hostname: matches
                .value_of(HOSTNAME)
                .ok_or(ConfigurationError::EmptyHostname)?
                .into(),
            port: matches
                .value_of(PORT)
                .ok_or(ConfigurationError::EmptyPort)?
                .parse()
                .map_err(ConfigurationError::wrong_port)?,
            database: matches
                .value_of(DATABASE)
                .ok_or(ConfigurationError::EmptyDatabase)?
                .into(),
            username: matches
                .value_of(USERNAME)
                .ok_or(ConfigurationError::EmptyUsername)?
                .into(),
            password: matches
                .value_of(PASSWORD)
                .ok_or(ConfigurationError::EmptyPassword)?
                .into(),
            n_workers: matches
                .value_of(NWORKERS)
                .ok_or(ConfigurationError::EmptyNWorkers)?
                .parse()
                .map_err(ConfigurationError::wrong_n_workers)?,
            recursive: matches.is_present(RECURSIVE),
            extensions: matches
                .values_of(EXTENSIONS)
                .map(|values| values.map(|value| value.into()).collect()),
            filter: matches.value_of(FILTER).map(|value| value.into()),
            text_mode: matches.is_present(TEXTMODE),
            suites: matches
                .values_of(SUITES)
                .ok_or(ConfigurationError::EmptySuites)?
                .map(|s| s.into())
                .collect(),
        })
    }

    pub fn hostname(&self) -> &String {
        &self.hostname
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn database(&self) -> &String {
        &self.database
    }

    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn password(&self) -> &String {
        &self.password
    }

    pub fn n_workers(&self) -> usize {
        self.n_workers
    }

    pub fn recursive(&self) -> bool {
        self.recursive
    }

    pub fn extensions(&self) -> Option<&Vec<String>> {
        self.extensions.as_ref()
    }

    pub fn filter(&self) -> Option<&String> {
        self.filter.as_ref()
    }

    pub fn text_mode(&self) -> bool {
        self.text_mode
    }

    pub fn suites(&self) -> &Vec<PathBuf> {
        &self.suites
    }
}
