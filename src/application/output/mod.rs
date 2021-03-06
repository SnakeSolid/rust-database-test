use std::fmt::Debug;

mod color;
mod counter;
mod plain;

use config::Configuration;
use dto::TestCase;
use dto::TestSuite;

pub use self::color::ColorFormatter;
pub use self::plain::PlainFormatter;

use self::counter::TestCounters;

pub trait Formatter: Debug {
    fn header(&self);
    fn footer(&self, passed: usize, skipped: usize, failed: usize);
    fn suite_started(&mut self, suite_name: &str);
    fn suite_failed(&mut self, suite_name: &str, message: &str);
    fn suite_skipped(&mut self, suite_name: &str);
    fn case_passed(&mut self, suite_name: &str, case_name: &str);
    fn case_failed(&mut self, suite_name: &str, case_name: &str, message: &str);
    fn case_skipped(&mut self, suite_name: &str, case_name: &str);
}

pub trait Output: Debug {
    fn header(&self);
    fn footer(&self);
    fn suite_started(&mut self, suite: &TestSuite);
    fn suite_failed(&mut self, suite: &TestSuite, message: &str);
    fn suite_skipped(&mut self, suite: &TestSuite);
    fn case_passed(&mut self, suite: &TestSuite, case: &TestCase);
    fn case_failed(&mut self, suite: &TestSuite, case: &TestCase, message: &str);
    fn case_skipped(&mut self, suite: &TestSuite, case: &TestCase);
}

#[derive(Debug, PartialEq, PartialOrd)]
enum Verbosity {
    None = 1,
    Results = 2,
    Failed = 3,
    PassedFailed = 4,
    PassedSkippedFailed = 5,
    All = 6,
}

impl From<isize> for Verbosity {
    fn from(value: isize) -> Verbosity {
        match value {
            n if n <= 1 => Verbosity::None,
            2 => Verbosity::Results,
            3 => Verbosity::Failed,
            4 => Verbosity::PassedFailed,
            5 => Verbosity::PassedSkippedFailed,
            _ => Verbosity::All,
        }
    }
}

#[derive(Debug)]
struct OutputImpl<F>
where
    F: Formatter,
{
    formatter: F,
    counters: TestCounters,
    verbosity: Verbosity,
}

pub fn create_output(config: &Configuration) -> Box<Output> {
    let verbosity = config.verbosity().into();

    if config.text_mode() {
        Box::new(OutputImpl::<PlainFormatter>::new(verbosity))
    } else {
        Box::new(OutputImpl::<ColorFormatter>::new(verbosity))
    }
}

impl<F> OutputImpl<F>
where
    F: Formatter + Default,
{
    fn new(verbosity: Verbosity) -> OutputImpl<F> {
        OutputImpl {
            formatter: F::default(),
            counters: TestCounters::default(),
            verbosity,
        }
    }
}

impl<F> Output for OutputImpl<F>
where
    F: Formatter,
{
    fn header(&self) {
        if self.verbosity >= Verbosity::Failed {
            self.formatter.header();
        }
    }

    fn footer(&self) {
        if self.verbosity >= Verbosity::Results {
            self.formatter.footer(
                self.counters.passed(),
                self.counters.skipped(),
                self.counters.failed(),
            );
        }
    }

    fn suite_started(&mut self, suite: &TestSuite) {
        if self.verbosity >= Verbosity::All {
            let suite_name = suite.description().unwrap_or_else(|| suite.name());

            self.formatter.suite_started(suite_name);
        }
    }

    fn suite_failed(&mut self, suite: &TestSuite, message: &str) {
        if self.verbosity >= Verbosity::Failed {
            let suite_name = suite.description().unwrap_or_else(|| suite.name());

            self.formatter.suite_failed(suite_name, message);
        }

        self.counters.add_failed(suite.cases().len());
    }

    fn suite_skipped(&mut self, suite: &TestSuite) {
        if self.verbosity >= Verbosity::PassedSkippedFailed {
            let suite_name = suite.description().unwrap_or_else(|| suite.name());

            self.formatter.suite_skipped(suite_name);
        }

        self.counters.add_skipped(suite.cases().len());
    }

    fn case_passed(&mut self, suite: &TestSuite, case: &TestCase) {
        if self.verbosity >= Verbosity::PassedFailed {
            let suite_name = suite.description().unwrap_or_else(|| suite.name());
            let case_name = case.description().unwrap_or_else(|| case.name());

            self.formatter.case_passed(suite_name, case_name);
        }

        self.counters.inc_passed();
    }

    fn case_failed(&mut self, suite: &TestSuite, case: &TestCase, message: &str) {
        if self.verbosity >= Verbosity::Failed {
            let suite_name = suite.description().unwrap_or_else(|| suite.name());
            let case_name = case.description().unwrap_or_else(|| case.name());

            self.formatter.case_failed(suite_name, case_name, message);
        }

        self.counters.inc_failed();
    }

    fn case_skipped(&mut self, suite: &TestSuite, case: &TestCase) {
        if self.verbosity >= Verbosity::PassedSkippedFailed {
            let suite_name = suite.description().unwrap_or_else(|| suite.name());
            let case_name = case.description().unwrap_or_else(|| case.name());

            self.formatter.case_skipped(suite_name, case_name);
        }

        self.counters.inc_skipped();
    }
}
