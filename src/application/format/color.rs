use term::color::Color;
use term::color::GREEN;
use term::color::RED;
use term::color::YELLOW;
use term::Error as TermError;
use term::Result as TermResult;
use term::StdoutTerminal;
use term;

use dto::TestCase;
use dto::TestSuite;

use super::Formatter;

#[derive(Debug)]
pub struct ColorFormatter {
    tests_passed: usize,
    tests_skipped: usize,
    tests_failed: usize,
}

impl Formatter for ColorFormatter {
    fn header(&self) {
        println!();
        println!("running tests...");
        println!();
    }

    fn footer(&self) {
        println!();
        print!("test result: ");

        if self.tests_passed == 0 && self.tests_failed == 0 {
            print_with_color(YELLOW, "skipped");
        } else if self.tests_failed > 0 {
            print_with_color(RED, "failed");
        } else {
            print_with_color(GREEN, "passed");
        }

        println!(
            ". {} passed; {} failed; {} skipped",
            self.tests_passed, self.tests_failed, self.tests_skipped
        );
        println!();
    }

    fn case_passed(&mut self, suite: &TestSuite, case: &TestCase) {
        let suite_name = suite.description().unwrap_or_else(|| suite.name());
        let case_name = case.description().unwrap_or_else(|| case.name());

        print!("test {}::{} .. ", suite_name, case_name);
        println_with_color(GREEN, "passed");

        self.tests_passed += 1;
    }

    fn case_failed(&mut self, suite: &TestSuite, case: &TestCase, message: &str) {
        let suite_name = suite.description().unwrap_or_else(|| suite.name());
        let case_name = case.description().unwrap_or_else(|| case.name());

        print!("test {}::{} .. ", suite_name, case_name);
        println_with_color(RED, "failed");
        println!("    - {}", message);

        self.tests_failed += 1;
    }

    fn case_skipped(&mut self, suite: &TestSuite, case: &TestCase) {
        let suite_name = suite.description().unwrap_or_else(|| suite.name());
        let case_name = case.description().unwrap_or_else(|| case.name());

        print!("test {}::{} .. ", suite_name, case_name);
        println_with_color(YELLOW, "skipped");

        self.tests_skipped += 1;
    }

    fn suite_started(&mut self, suite: &TestSuite) {
        let suite_name = suite.description().unwrap_or_else(|| suite.name());

        println!("suite {} .. started", suite_name);
    }

    fn suite_skipped(&mut self, suite: &TestSuite) {
        let suite_name = suite.description().unwrap_or_else(|| suite.name());

        print!("suite {} .. ", suite_name);
        println_with_color(YELLOW, "skipped");

        self.tests_skipped += suite.cases().len();
    }

    fn suite_error(&mut self, suite: &TestSuite, message: &str) {
        let suite_name = suite.description().unwrap_or_else(|| suite.name());

        print!("suite {} .. ", suite_name);
        println_with_color(RED, "error");
        println!("  - {}", message);

        self.tests_failed += suite.cases().len();
    }
}

impl Default for ColorFormatter {
    fn default() -> ColorFormatter {
        ColorFormatter {
            tests_passed: 0,
            tests_skipped: 0,
            tests_failed: 0,
        }
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
