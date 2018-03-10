use std::fs::File;
use std::sync::mpsc::sync_channel;

use serde_yaml;

use config::Configuration;
use dto::TestSuite;

use super::bus::MessageBus;
use super::error::ApplicationError;
use super::error::ApplicationResult;
use super::format::Formatter;
use super::worker::QueryResult;
use super::worker::Worker;
use super::worker::WorkerReply;

#[derive(Debug)]
pub struct Application<'a> {
    config: &'a Configuration,
    formatter: &'a mut Formatter,
    suites: Vec<TestSuite>,
}

impl<'a> Application<'a> {
    pub fn new(config: &'a Configuration, formatter: &'a mut Formatter) -> Application<'a> {
        Application {
            config,
            formatter,
            suites: Vec::default(),
        }
    }

    pub fn run(mut self) -> ApplicationResult<()> {
        self.read_suites()?;

        let n_cases = self.get_n_cases();
        let (message_sender, message_receiver) = sync_channel(n_cases);
        let (reply_sender, reply_receiver) = sync_channel(n_cases);
        let bus = MessageBus::new(message_sender, reply_receiver);

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

        for (suite_index, suite) in self.suites.iter().enumerate() {
            if let Some(skip) = suite.skip() {
                bus.send_suite_skip(suite_index, skip)?;
            } else {
                bus.send_suite(suite_index, suite)?;
            }
        }

        bus.message_loop(|bus, reply| match reply {
            WorkerReply::SuiteSkip {
                suite_index,
                result,
            } => self.on_suite_skip(bus, suite_index, result),
            WorkerReply::CaseSkip {
                suite_index,
                case_index,
                result,
            } => self.on_case_skip(bus, suite_index, case_index, result),
            WorkerReply::CaseRun {
                suite_index,
                case_index,
                result,
            } => self.on_case_run(suite_index, case_index, result),
        })?;

        worker_handler.join().unwrap();

        self.formatter.footer();

        Ok(())
    }

    fn on_case_run(
        &mut self,
        suite_index: usize,
        case_index: usize,
        result: QueryResult,
    ) -> ApplicationResult<()> {
        let suite = &self.suites[suite_index];
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
        bus: &MessageBus,
        suite_index: usize,
        case_index: usize,
        result: QueryResult,
    ) -> ApplicationResult<()> {
        let suite = &self.suites[suite_index];
        let case = &suite.cases()[case_index];

        match result {
            QueryResult::Success => self.formatter.case_skipped(suite, case),
            QueryResult::Fail { .. } => bus.send_case_run(suite_index, case_index, case)?,
            QueryResult::Error { ref message } => self.formatter.case_failed(suite, case, message),
        }

        Ok(())
    }

    fn on_suite_skip(
        &mut self,
        bus: &MessageBus,
        suite_index: usize,
        result: QueryResult,
    ) -> ApplicationResult<()> {
        let suite = &self.suites[suite_index];

        match result {
            QueryResult::Success => self.formatter.suite_skipped(suite),
            QueryResult::Fail { .. } => {
                self.formatter.suite_started(suite);
                bus.send_suite(suite_index, suite)?;
            }
            QueryResult::Error { ref message } => self.formatter.suite_error(suite, message),
        }

        Ok(())
    }

    fn get_n_cases(&self) -> usize {
        self.suites.iter().map(|s| s.cases().len()).sum()
    }

    fn read_suites(&mut self) -> ApplicationResult<()> {
        for path in self.config.suites() {
            let reader = File::open(path).map_err(ApplicationError::suite_io_error)?;
            let suite =
                serde_yaml::from_reader(reader).map_err(ApplicationError::suite_yaml_error)?;

            self.suites.push(suite);
        }

        Ok(())
    }
}
