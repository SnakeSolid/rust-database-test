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
use super::format::Formatter;
use super::worker::QueryResult;
use super::worker::Worker;
use super::worker::WorkerMessage;
use super::worker::WorkerReply;

#[derive(Debug)]
pub struct Application<'a> {
    config: &'a Configuration,
    formatter: &'a mut Formatter,
    n_messages: usize,
}

impl<'a> Application<'a> {
    pub fn new(config: &'a Configuration, formatter: &'a mut Formatter) -> Application<'a> {
        Application {
            config,
            formatter,
            n_messages: 0,
        }
    }

    pub fn run(mut self) -> ApplicationResult<()> {
        let suites = self.read_suites()?;
        let n_cases = self.get_n_cases(&suites);
        let (message_sender, message_receiver) = sync_channel(n_cases);
        let (reply_sender, reply_receiver) = sync_channel(n_cases);

        self.formatter.header();

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
                    self.on_suite_skip(&message_sender, &suites[suite_index], suite_index, result)?
                }
                WorkerReply::CaseSkip {
                    suite_index,
                    case_index,
                    result,
                } => self.on_case_skip(
                    &message_sender,
                    &suites[suite_index],
                    suite_index,
                    case_index,
                    result,
                )?,
                WorkerReply::CaseRun {
                    suite_index,
                    case_index,
                    result,
                } => self.on_case_run(&suites[suite_index], case_index, result)?,
            }

            self.n_messages -= 1;

            if self.n_messages == 0 {
                break;
            }
        }

        drop(message_sender);

        worker_handler.join().unwrap();

        self.formatter.footer();

        Ok(())
    }

    fn on_case_run(
        &mut self,
        suite: &TestSuite,
        case_index: usize,
        result: QueryResult,
    ) -> ApplicationResult<()> {
        let case = &suite.cases()[case_index];

        match result {
            QueryResult::Success => self.formatter.case_passed(suite, case),
            QueryResult::Fail { ref message } | QueryResult::Error { ref message } => {
                self.formatter.case_failed(suite, case, message)
            }
        }

        Ok(())
    }

    fn on_case_skip(
        &mut self,
        sender: &SyncSender<WorkerMessage>,
        suite: &TestSuite,
        suite_index: usize,
        case_index: usize,
        result: QueryResult,
    ) -> ApplicationResult<()> {
        let case = &suite.cases()[case_index];

        match result {
            QueryResult::Success => self.formatter.case_skipped(suite, case),
            QueryResult::Fail { .. } => self.send_case_run(&sender, suite_index, case_index, case)?,
            QueryResult::Error { ref message } => self.formatter.case_failed(suite, case, message),
        }

        Ok(())
    }

    fn on_suite_skip(
        &mut self,
        sender: &SyncSender<WorkerMessage>,
        suite: &TestSuite,
        suite_index: usize,
        result: QueryResult,
    ) -> ApplicationResult<()> {
        match result {
            QueryResult::Success => self.formatter.suite_skipped(suite),
            QueryResult::Fail { .. } => {
                self.formatter.suite_started(suite);
                self.send_suite(sender, suite_index, suite)?;
            }
            QueryResult::Error { ref message } => self.formatter.suite_error(suite, message),
        }

        Ok(())
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
