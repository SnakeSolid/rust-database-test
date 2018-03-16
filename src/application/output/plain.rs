use super::Formatter;

#[derive(Debug)]
pub struct PlainFormatter {}

impl Formatter for PlainFormatter {
    fn header(&self) {
        println!();
        println!("running tests...");
        println!();
    }

    fn footer(&self, passed: usize, skipped: usize, failed: usize) {
        println!();
        print!("test result: ");

        if passed == 0 && failed == 0 {
            print!("skipped");
        } else if failed > 0 {
            print!("failed");
        } else {
            print!("passed");
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
        println!("suite {} .. error", suite_name);
        println!("  - {}", message);
    }

    fn suite_skipped(&mut self, suite_name: &str) {
        println!("suite {} .. skipped", suite_name);
    }

    fn case_passed(&mut self, suite_name: &str, case_name: &str) {
        println!("test {}::{} .. passed", suite_name, case_name);
    }

    fn case_failed(&mut self, suite_name: &str, case_name: &str, message: &str) {
        println!("test {}::{} .. failed", suite_name, case_name);
        println!("    - {}", message);
    }

    fn case_skipped(&mut self, suite_name: &str, case_name: &str) {
        println!("test {}::{} .. skipped", suite_name, case_name);
    }
}

impl Default for PlainFormatter {
    fn default() -> PlainFormatter {
        PlainFormatter {}
    }
}
