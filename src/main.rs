#[macro_use]
extern crate serde_derive;

extern crate clap;
extern crate postgres;
extern crate serde_yaml;
extern crate term;

use std::process;

use clap::App;
use clap::Arg;

mod application;
mod config;
mod dto;
mod validate;

use application::Application;
use application::ApplicationResult;
use application::ApplicationStatus;
use config::BEQUIET;
use config::BEVERBOSE;
use config::Configuration;
use config::DATABASE;
use config::EXTENSIONS;
use config::FILTER;
use config::HOSTNAME;
use config::NWORKERS;
use config::PASSWORD;
use config::PORT;
use config::RECURSIVE;
use config::SUITES;
use config::TEXTMODE;
use config::USERNAME;

fn main() {
    process::exit(match start_app() {
        Ok(ApplicationStatus::Success) => 0,
        Ok(ApplicationStatus::Fail) => 1,
        Err(_) => 2,
    });
}

fn start_app() -> ApplicationResult<ApplicationStatus> {
    let matches = App::new("Database Test")
        .version("0.1")
        .author("Anton Shabanov <snakesolid@ngs.ru>")
        .about("Executes simple test suites for PostgreSQL databases.")
        .arg(
            Arg::with_name(HOSTNAME)
                .short("h")
                .long("host-name")
                .required(true)
                .takes_value(true)
                .value_name("HOSTNAME")
                .default_value("localhost")
                .help("PostgreSQL host name or IP address")
                .display_order(1),
        )
        .arg(
            Arg::with_name(PORT)
                .short("p")
                .long("port")
                .required(true)
                .takes_value(true)
                .value_name("PORT")
                .default_value("5432")
                .validator(validate::is_port)
                .help("PostgreSQL port")
                .display_order(2),
        )
        .arg(
            Arg::with_name(USERNAME)
                .short("u")
                .long("user-name")
                .required(true)
                .takes_value(true)
                .value_name("USERNAME")
                .help("PostgreSQL user name")
                .display_order(3),
        )
        .arg(
            Arg::with_name(PASSWORD)
                .short("w")
                .long("password")
                .required(true)
                .takes_value(true)
                .value_name("PASSWORD")
                .help("PostgreSQL password")
                .display_order(4),
        )
        .arg(
            Arg::with_name(DATABASE)
                .short("d")
                .long("database")
                .required(true)
                .takes_value(true)
                .value_name("DATABASE")
                .help("PostgreSQL database")
                .display_order(5),
        )
        .arg(
            Arg::with_name(NWORKERS)
                .short("n")
                .long("n-workers")
                .takes_value(true)
                .value_name("NWORKERS")
                .default_value("4")
                .validator(validate::is_n_workers)
                .help("Number of worker threads")
                .display_order(6),
        )
        .arg(
            Arg::with_name(RECURSIVE)
                .short("r")
                .long("recursive")
                .help("Read all files under each directory, recursively")
                .display_order(7),
        )
        .arg(
            Arg::with_name(EXTENSIONS)
                .short("e")
                .long("extensions")
                .takes_value(true)
                .multiple(true)
                .value_name("EXTENSIONS")
                .help("File extension filter for recursive search")
                .display_order(8),
        )
        .arg(
            Arg::with_name(FILTER)
                .short("f")
                .long("filter")
                .takes_value(true)
                .value_name("FILTER")
                .help("Filter test suites by name")
                .display_order(9),
        )
        .arg(
            Arg::with_name(TEXTMODE)
                .short("t")
                .long("text-mode")
                .help("Use plain text mode instead of color")
                .display_order(10),
        )
        .arg(
            Arg::with_name(BEVERBOSE)
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Be verbose, can be applied several times")
                .conflicts_with(BEQUIET)
                .display_order(11),
        )
        .arg(
            Arg::with_name(BEQUIET)
                .short("q")
                .long("quiet")
                .multiple(true)
                .help("Be quiet, can be applied several times")
                .conflicts_with(BEVERBOSE)
                .display_order(12),
        )
        .arg(
            Arg::with_name(SUITES)
                .required(true)
                .multiple(true)
                .last(true)
                .value_name("SUITES")
                .validator(validate::is_exists)
                .help("Test suites to execute"),
        )
        .get_matches();

    let config = match Configuration::from_matches(matches) {
        Ok(config) => config,
        Err(err) => panic!("{}", err),
    };
    let result = Application::new(&config).run();

    if let Err(ref err) = result {
        println!("{}", err);
    }

    result
}
