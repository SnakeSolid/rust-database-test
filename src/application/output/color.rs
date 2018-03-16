use term::color::Color;
use term::color::GREEN;
use term::color::RED;
use term::color::YELLOW;
use term::Error as TermError;
use term::Result as TermResult;
use term::StdoutTerminal;
use term;

use super::Formatter;

#[derive(Debug)]
pub struct ColorFormatter {}

impl Formatter for ColorFormatter {
    fn header(&self) {
        println!();
        println!("running tests...");
        println!();
    }

    fn footer(&self, passed: usize, skipped: usize, failed: usize) {
        println!();
        print!("test result: ");

        if passed == 0 && failed == 0 {
            print_with_color(YELLOW, "skipped");
        } else if failed > 0 {
            print_with_color(RED, "failed");
        } else {
            print_with_color(GREEN, "passed");
        }

        println!(
            ". {} passed; {} failed; {} skipped",
            passed, failed, skipped
        );
        println!();
    }

    fn suite_started(&mut self, suite_name: &str) {
        println!("suite {} .. started", suite_name);
    }

    fn suite_failed(&mut self, suite_name: &str, message: &str) {
        print!("suite {} .. ", suite_name);
        println_with_color(RED, "error");
        println!("  - {}", message);
    }

    fn suite_skipped(&mut self, suite_name: &str) {
        print!("suite {} .. ", suite_name);
        println_with_color(YELLOW, "skipped");
    }

    fn case_passed(&mut self, suite_name: &str, case_name: &str) {
        print!("test {}::{} .. ", suite_name, case_name);
        println_with_color(GREEN, "passed");
    }

    fn case_failed(&mut self, suite_name: &str, case_name: &str, message: &str) {
        print!("test {}::{} .. ", suite_name, case_name);
        println_with_color(RED, "failed");
        println!("    - {}", message);
    }

    fn case_skipped(&mut self, suite_name: &str, case_name: &str) {
        print!("test {}::{} .. ", suite_name, case_name);
        println_with_color(YELLOW, "skipped");
    }
}

impl Default for ColorFormatter {
    fn default() -> ColorFormatter {
        ColorFormatter {}
    }
}

fn try_terminal<F>(callback: F) -> TermResult<()>
where
    F: Fn(&mut StdoutTerminal) -> TermResult<()>,
{
    match term::stdout() {
        Some(mut f) => callback(f.as_mut()),
        None => Err(TermError::NotSupported),
    }
}

fn print_with_color(color: Color, value: &str) {
    if let Err(_) = try_terminal(|f| {
        f.fg(color)?;
        write!(f, "{}", value)?;
        f.reset()
    }) {
        print!("{}", value);
    }
}

fn println_with_color(color: Color, value: &str) {
    let result = try_terminal(|f| {
        f.fg(color)?;
        write!(f, "{}", value)?;
        f.reset()?;
        writeln!(f, "")?;

        Ok(())
    });

    if result.is_err() {
        println!("{}", value);
    }
}
