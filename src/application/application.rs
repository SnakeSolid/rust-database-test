use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::SyncSender;
use std::sync::Mutex;
use std::thread::JoinHandle;

use config::Configuration;
use dto::TestSuite;

use super::bus::MessageBus;
use super::error::ApplicationError;
use super::error::ApplicationResult;
use super::format::Formatter;
use super::read::SuiteReader;
use super::status::ApplicationStatus;
use super::worker::QueryResult;
use super::worker::Worker;
use super::worker::WorkerMessage;
use super::worker::WorkerReply;

pub struct Application<'a> {
    config: &'a Configuration,
    formatter: &'a mut Formatter,
    suites: Vec<TestSuite>,
    status: ApplicationStatus,
}

impl<'a> Application<'a> {
    pub fn new(config: &'a Configuration, formatter: &'a mut Formatter) -> Application<'a> {
        Application {
            config,
            formatter,
            suites: Vec::default(),
            status: ApplicationStatus::Success,
        }
    }

    pub fn run(mut self) -> ApplicationResult<ApplicationStatus> {
        self.read_suites()?;

        let n_cases = self.get_n_cases();
        let (message_sender, message_receiver) = sync_channel(n_cases);
        let (reply_sender, reply_receiver) = sync_channel(n_cases);
        let bus = MessageBus::new(message_sender, reply_receiver);
        let workers = self.spawn_workers(message_receiver, reply_sender)?;

        self.formatter.header();

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

        self.formatter.footer();
        self.join_workers(workers);

        Ok(self.status)
    }

    fn join_workers(&self, workers: Vec<JoinHandle<()>>) {
        for worker in workers {
            match worker.join() {
                Ok(_) => {}
                Err(_) => println!("Failed to join worker thread"),
            }
        }
    }

    fn spawn_workers(
        &self,
        message_receiver: Receiver<WorkerMessage>,
        reply_sender: SyncSender<WorkerReply>,
    ) -> ApplicationResult<Vec<JoinHandle<()>>> {
        let n_workers = self.config.n_workers();
        let message_receiver = Arc::new(Mutex::new(message_receiver));
        let mut workers = Vec::with_capacity(n_workers);

        for _ in 0..n_workers {
            let message_receiver = message_receiver.clone();
            let reply_sender = reply_sender.clone();
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

            workers.push(worker_handler);
        }

        Ok(workers)
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
                self.status = ApplicationStatus::Fail;
                self.formatter.case_failed(suite, case, message);
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
            QueryResult::Error { ref message } => {
                self.status = ApplicationStatus::Fail;
                self.formatter.case_failed(suite, case, message);
            }
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
            QueryResult::Error { ref message } => {
                self.status = ApplicationStatus::Fail;
                self.formatter.suite_error(suite, message)
            }
        }

        Ok(())
    }

    fn get_n_cases(&self) -> usize {
        self.suites.iter().map(|s| s.cases().len()).sum()
    }

    fn read_suites(&mut self) -> ApplicationResult<()> {
        let mut reader = SuiteReader::default();
        reader.read(self.config.suites(), self.config.recursive())?;

        self.suites.extend(reader.suites());

        Ok(())
    }
}
