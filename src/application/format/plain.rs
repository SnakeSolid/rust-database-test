use dto::TestCase;
use dto::TestSuite;

use super::Formatter;

#[derive(Debug)]
pub struct PlainFormatter {
    tests_passed: usize,
    tests_skipped: usize,
    tests_failed: usize,
}

impl Formatter for PlainFormatter {
    fn header(&self) {
        println!("");
        println!("running tests...");
        println!("");
    }

    fn footer(&self) {
        println!("");
        print!("test result: ");

        if self.tests_passed == 0 && self.tests_failed == 0 {
            print!("skipped");
        } else if self.tests_failed > 0 {
            print!("failed");
        } else {
            print!("passed");
        }

        println!(
            ". {} passed; {} failed; {} skipped",
            self.tests_passed, self.tests_failed, self.tests_skipped
        );
        println!("");
    }

    fn case_passed(&mut self, suite: &TestSuite, case: &TestCase) {
        let suite_name = suite.description().unwrap_or(suite.name());
        let case_name = case.description().unwrap_or(case.name());

        println!("  * {}::{} .. passed", suite_name, case_name);

        self.tests_passed += 1;
    }

    fn case_failed(&mut self, suite: &TestSuite, case: &TestCase, message: &str) {
        let suite_name = suite.description().unwrap_or(suite.name());
        let case_name = case.description().unwrap_or(case.name());

        println!("  * {}::{} .. failed", suite_name, case_name);
        println!("    - {}", message);

        self.tests_failed += 1;
    }

    fn case_skipped(&mut self, suite: &TestSuite, case: &TestCase) {
        let suite_name = suite.description().unwrap_or(suite.name());
        let case_name = case.description().unwrap_or(case.name());

        println!("  * {}::{} .. skipped", suite_name, case_name);

        self.tests_skipped += 1;
    }

    fn suite_started(&mut self, suite: &TestSuite) {
        let suite_name = suite.description().unwrap_or(suite.name());

        println!("* {} .. started", suite_name);
    }

    fn suite_skipped(&mut self, suite: &TestSuite) {
        let suite_name = suite.description().unwrap_or(suite.name());

        println!("* {} .. skipped", suite_name);

        self.tests_skipped += suite.cases().len();
    }

    fn suite_error(&mut self, suite: &TestSuite, message: &str) {
        let suite_name = suite.description().unwrap_or(suite.name());

        println!("* {} .. error", suite_name);
        println!("  - {}", message);

        self.tests_skipped += suite.cases().len();
    }
}

impl Default for PlainFormatter {
    fn default() -> PlainFormatter {
        PlainFormatter {
            tests_passed: 0,
            tests_skipped: 0,
            tests_failed: 0,
        }
    }
}
