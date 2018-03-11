mod color;
mod plain;

pub use self::color::ColorFormatter;
pub use self::plain::PlainFormatter;

use dto::TestCase;
use dto::TestSuite;

pub trait Formatter {
    fn header(&self);
    fn footer(&self);
    fn case_passed(&mut self, suite: &TestSuite, case: &TestCase);
    fn case_failed(&mut self, suite: &TestSuite, case: &TestCase, message: &str);
    fn case_skipped(&mut self, suite: &TestSuite, case: &TestCase);
    fn suite_started(&mut self, suite: &TestSuite);
    fn suite_skipped(&mut self, suite: &TestSuite);
    fn suite_error(&mut self, suite: &TestSuite, message: &str);
}
