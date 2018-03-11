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
use application::ColorFormatter;
use application::Formatter;
use application::PlainFormatter;
use config::Configuration;

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
            Arg::with_name("hostname")
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
            Arg::with_name("port")
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
            Arg::with_name("username")
                .short("u")
                .long("user-name")
                .required(true)
                .takes_value(true)
                .value_name("USERNAME")
                .help("PostgreSQL user name")
                .display_order(3),
        )
        .arg(
            Arg::with_name("password")
                .short("w")
                .long("password")
                .required(true)
                .takes_value(true)
                .value_name("PASSWORD")
                .help("PostgreSQL password")
                .display_order(4),
        )
        .arg(
            Arg::with_name("database")
                .short("d")
                .long("database")
                .required(true)
                .takes_value(true)
                .value_name("DATABASE")
                .help("PostgreSQL database")
                .display_order(5),
        )
        .arg(
            Arg::with_name("nworkers")
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
            Arg::with_name("textmode")
                .short("t")
                .long("text-mode")
                .help("Use plain text mode instead of color")
                .display_order(7),
        )
        .arg(
            Arg::with_name("suites")
                .required(true)
                .multiple(true)
                .last(true)
                .value_name("SUITES")
                .validator(validate::is_file)
                .help("Test suites to execute"),
        )
        .get_matches();

    let config = match Configuration::from_matches(matches) {
        Ok(config) => config,
        Err(err) => panic!("{}", err),
    };
    let mut formatter: Box<Formatter> = if config.text_mode() {
        Box::new(PlainFormatter::default())
    } else {
        Box::new(ColorFormatter::default())
    };
    let result = Application::new(&config, formatter.as_mut()).run();

    if let Err(ref err) = result {
        println!("{}", err);
    }

    result
}
