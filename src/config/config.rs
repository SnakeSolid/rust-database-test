use std::path::PathBuf;

use clap::ArgMatches;

use super::ConfigurationResult;
use super::ConfigurationError;

#[derive(Debug)]
pub struct Configuration {
    hostname: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    n_workers: usize,
    suites: Vec<PathBuf>,
}

impl Configuration {
    pub fn from_matches<'a>(matches: ArgMatches<'a>) -> ConfigurationResult<Configuration> {
        Ok(Configuration {
            hostname: matches
                .value_of("hostname")
                .ok_or(ConfigurationError::EmptyHostname)?
                .into(),
            port: matches
                .value_of("port")
                .ok_or(ConfigurationError::EmptyPort)?
                .parse()
                .map_err(ConfigurationError::wrong_port)?,
            database: matches
                .value_of("database")
                .ok_or(ConfigurationError::EmptyDatabase)?
                .into(),
            username: matches
                .value_of("username")
                .ok_or(ConfigurationError::EmptyUsername)?
                .into(),
            password: matches
                .value_of("password")
                .ok_or(ConfigurationError::EmptyPassword)?
                .into(),
            n_workers: matches
                .value_of("nworkers")
                .ok_or(ConfigurationError::EmptyNWorkers)?
                .parse()
                .map_err(ConfigurationError::wrong_n_workers)?,
            suites: matches
                .values_of("suites")
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

    pub fn suites(&self) -> &Vec<PathBuf> {
        &self.suites
    }
}
