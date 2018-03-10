use std::fs::File;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::SyncSender;

use serde_yaml;

use config::Configuration;
use dto::QueryClause;
use dto::TestCase;
use dto::TestSuite;

use super::error::ApplicationError;
use super::error::ApplicationResult;
use super::worker::QueryResult;
use super::worker::Worker;
use super::worker::WorkerMessage;
use super::worker::WorkerReply;

#[derive(Debug)]
pub struct Application<'a> {
    config: &'a Configuration,
    n_messages: usize,
    tests_passed: usize,
    tests_skipped: usize,
    tests_failed: usize,
}

impl<'a> Application<'a> {
    pub fn new(config: &'a Configuration) -> Application<'a> {
        Application {
            config,
            n_messages: 0,
            tests_passed: 0,
            tests_skipped: 0,
            tests_failed: 0,
        }
    }

    pub fn run(mut self) -> ApplicationResult<()> {
        let suites = self.read_suites()?;
        let n_cases = self.get_n_cases(&suites);
        let (message_sender, message_receiver) = sync_channel(n_cases);
        let (reply_sender, reply_receiver) = sync_channel(n_cases);

        self.display_header();

        let worker_handler = Worker::new(
            message_receiver,
            reply_sender,
            self.config.hostname(),
            self.config.port(),
            self.config.database(),
            self.config.username(),
            self.config.password(),
        ).start()
            .map_err(ApplicationError::worker_error)?;

        for (suite_index, suite) in suites.iter().enumerate() {
            if let Some(skip) = suite.skip() {
                self.send_suite_skip(&message_sender, suite_index, skip)?;
            } else {
                self.send_suite(&message_sender, suite_index, suite)?;
            }
        }

        for reply in reply_receiver {
            match reply {
                WorkerReply::SuiteSkip {
                    suite_index,
                    result,
                } => {
                    if let QueryResult::Success = result {
                        self.display_suite_skipped(&suites[suite_index]);

                        self.tests_skipped += suites[suite_index].cases().len();
                    } else {
                        self.send_suite(&message_sender, suite_index, &suites[suite_index])?;
                    }
                }
                WorkerReply::CaseSkip {
                    suite_index,
                    case_index,
                    result,
                } => {
                    if let QueryResult::Success = result {
                        self.display_case_skipped(
                            &suites[suite_index],
                            &suites[suite_index].cases()[case_index],
                        );

                        self.tests_skipped += 1;
                    } else {
                        self.send_case_run(
                            &message_sender,
                            suite_index,
                            case_index,
                            &suites[suite_index].cases()[case_index],
                        )?;
                    }
                }
                WorkerReply::CaseRun {
                    suite_index,
                    case_index,
                    result,
                } => {
                    if let QueryResult::Success = result {
                        self.display_case_passed(
                            &suites[suite_index],
                            &suites[suite_index].cases()[case_index],
                        );

                        self.tests_passed += 1;
                    } else {
                        self.display_case_failed(
                            &suites[suite_index],
                            &suites[suite_index].cases()[case_index],
                        );

                        self.tests_failed += 1;
                    }
                }
            }

            self.n_messages -= 1;

            if self.n_messages == 0 {
                break;
            }
        }

        drop(message_sender);

        worker_handler.join().unwrap();

        self.display_footer();

        Ok(())
    }

    fn display_footer(&self) {
        println!("");
        println!(
            "test result: ok. {} passed; {} failed; {} skipped",
            self.tests_passed, self.tests_failed, self.tests_skipped
        );
    }

    fn display_case_passed(&self, suite: &TestSuite, case: &TestCase) {
        let suite_name = suite.description().unwrap_or(suite.name());
        let case_name = case.description().unwrap_or(case.name());

        println!("  * {}::{} passed", suite_name, case_name);
    }

    fn display_case_failed(&self, suite: &TestSuite, case: &TestCase) {
        let suite_name = suite.description().unwrap_or(suite.name());
        let case_name = case.description().unwrap_or(case.name());

        println!("  * {}::{} failed", suite_name, case_name);
    }

    fn display_case_skipped(&self, suite: &TestSuite, case: &TestCase) {
        let suite_name = suite.description().unwrap_or(suite.name());
        let case_name = case.description().unwrap_or(case.name());

        println!("  * {}::{} skipped", suite_name, case_name);
    }

    fn display_suite_skipped(&self, suite: &TestSuite) {
        let suite_name = suite.description().unwrap_or(suite.name());

        println!("* {} skipped", suite_name);
    }

    fn display_header(&self) {
        println!("running tests...");
        println!("");
    }

    fn send_suite_skip(
        &mut self,
        sender: &SyncSender<WorkerMessage>,
        suite_index: usize,
        clause: &QueryClause,
    ) -> ApplicationResult<()> {
        let message = WorkerMessage::suite_skip(suite_index, clause);

        self.n_messages += 1;

        sender
            .send(message)
            .map_err(ApplicationError::send_message_error)
    }

    fn send_suite(
        &mut self,
        sender: &SyncSender<WorkerMessage>,
        suite_index: usize,
        suite: &TestSuite,
    ) -> ApplicationResult<()> {
        for (case_index, case) in suite.cases().iter().enumerate() {
            if let Some(skip) = case.skip() {
                self.send_case_skip(sender, suite_index, case_index, skip)?;
            } else {
                self.send_case_run(sender, suite_index, case_index, case)?;
            }
        }

        Ok(())
    }

    fn send_case_skip(
        &mut self,
        sender: &SyncSender<WorkerMessage>,
        suite_index: usize,
        case_index: usize,
        clause: &QueryClause,
    ) -> ApplicationResult<()> {
        let message = WorkerMessage::case_skip(suite_index, case_index, clause);

        self.n_messages += 1;

        sender
            .send(message)
            .map_err(ApplicationError::send_message_error)
    }

    fn send_case_run(
        &mut self,
        sender: &SyncSender<WorkerMessage>,
        suite_index: usize,
        case_index: usize,
        case: &TestCase,
    ) -> ApplicationResult<()> {
        let message = WorkerMessage::case_run(suite_index, case_index, case);

        self.n_messages += 1;

        sender
            .send(message)
            .map_err(ApplicationError::send_message_error)
    }

    fn get_n_cases(&self, suites: &Vec<TestSuite>) -> usize {
        suites.iter().map(|s| s.cases().len()).sum()
    }

    fn read_suites(&self) -> ApplicationResult<Vec<TestSuite>> {
        let mut result = Vec::default();

        for path in self.config.suites() {
            let reader = File::open(path).map_err(ApplicationError::suite_io_error)?;
            let suite =
                serde_yaml::from_reader(reader).map_err(ApplicationError::suite_yaml_error)?;

            result.push(suite);
        }

        Ok(result)
    }
}
