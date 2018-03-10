use dto::TestCase;
use dto::TestSuite;

#[derive(Debug)]
pub struct Formatter {
    tests_passed: usize,
    tests_skipped: usize,
    tests_failed: usize,
}

impl Formatter {
    pub fn header(&self) {
        println!("running tests...");
        println!("");
    }

    pub fn footer(&self) {
        println!("");

        if self.tests_failed == 0 {
            println!(
                "test result: ok. {} passed; {} failed; {} skipped",
                self.tests_passed, self.tests_failed, self.tests_skipped
            );
        } else {
            println!(
                "test result: fail. {} passed; {} failed; {} skipped",
                self.tests_passed, self.tests_failed, self.tests_skipped
            );
        }
    }

    pub fn case_passed(&mut self, suite: &TestSuite, case: &TestCase) {
        let suite_name = suite.description().unwrap_or(suite.name());
        let case_name = case.description().unwrap_or(case.name());

        println!("  * {}::{} .. passed", suite_name, case_name);

        self.tests_passed += 1;
    }

    pub fn case_failed(&mut self, suite: &TestSuite, case: &TestCase, message: &str) {
        let suite_name = suite.description().unwrap_or(suite.name());
        let case_name = case.description().unwrap_or(case.name());

        println!("  * {}::{} .. failed", suite_name, case_name);
        println!("    - {}", message);

        self.tests_failed += 1;
    }

    pub fn case_skipped(&mut self, suite: &TestSuite, case: &TestCase) {
        let suite_name = suite.description().unwrap_or(suite.name());
        let case_name = case.description().unwrap_or(case.name());

        println!("  * {}::{} .. skipped", suite_name, case_name);

        self.tests_skipped += 1;
    }

    pub fn suite_started(&mut self, suite: &TestSuite) {
        let suite_name = suite.description().unwrap_or(suite.name());

        println!("* {} .. started", suite_name);
    }

    pub fn suite_skipped(&mut self, suite: &TestSuite) {
        let suite_name = suite.description().unwrap_or(suite.name());

        println!("* {} .. skipped", suite_name);

        self.tests_skipped += suite.cases().len();
    }

    pub fn suite_error(&mut self, suite: &TestSuite, message: &str) {
        let suite_name = suite.description().unwrap_or(suite.name());

        println!("* {} .. error", suite_name);
        println!("  - {}", message);

        self.tests_skipped += suite.cases().len();
    }
}

impl Default for Formatter {
    fn default() -> Formatter {
        Formatter {
            tests_passed: 0,
            tests_skipped: 0,
            tests_failed: 0,
        }
    }
}
