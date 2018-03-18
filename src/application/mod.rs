use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::SyncSender;
use std::sync::Mutex;
use std::thread::JoinHandle;

use config::Configuration;
use dto::TestSuite;

mod bus;
mod error;
mod output;
mod read;
mod status;
mod worker;

pub use self::error::ApplicationError;
pub use self::error::ApplicationResult;
pub use self::output::Output;
pub use self::read::SuiteReader;
pub use self::status::ApplicationStatus;

use self::bus::MessageBus;
use self::bus::MessageSender;
use self::worker::QueryResult;
use self::worker::Worker;
use self::worker::WorkerMessage;
use self::worker::WorkerReply;

#[derive(Debug)]
pub struct Application<'a> {
    config: &'a Configuration,
    output: Box<Output>,
    suites: Vec<TestSuite>,
    status: ApplicationStatus,
}

impl<'a> Application<'a> {
    pub fn new(config: &'a Configuration) -> ApplicationResult<Application<'a>> {
        Ok(Application {
            config,
            output: output::create_output(config),
            suites: SuiteReader::new(config).read()?,
            status: ApplicationStatus::Success,
        })
    }

    pub fn run(mut self) -> ApplicationResult<ApplicationStatus> {
        let n_cases = self.get_n_cases();
        let (message_sender, message_receiver) = sync_channel(n_cases);
        let (reply_sender, reply_receiver) = sync_channel(n_cases);
        let workers = self.spawn_workers(message_receiver, reply_sender)?;
        let mut bus = MessageBus::new(message_sender, reply_receiver);

        self.output.header();
        self.send_start_suites(&mut bus)?;

        bus.message_loop(|sender, reply| match reply {
            WorkerReply::SuiteSkip {
                suite_index,
                result,
            } => self.on_suite_skip(sender, suite_index, result),
            WorkerReply::CaseSkip {
                suite_index,
                case_index,
                result,
            } => self.on_case_skip(sender, suite_index, case_index, result),
            WorkerReply::CaseRun {
                suite_index,
                case_index,
                result,
            } => self.on_case_run(suite_index, case_index, result),
        })?;

        self.output.footer();
        self.join_workers(workers);

        Ok(self.status)
    }

    fn send_start_suites(&mut self, bus: &mut MessageBus) -> ApplicationResult<()> {
        for (suite_index, suite) in self.suites.iter().enumerate() {
            if let Some(skip) = suite.skip() {
                bus.send_suite_skip(suite_index, skip)?;
            } else {
                self.output.suite_started(suite);

                bus.send_suite(suite_index, suite)?;
            }
        }

        Ok(())
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
            QueryResult::Success => self.output.case_passed(suite, case),
            QueryResult::Fail { ref message } | QueryResult::Error { ref message } => {
                self.status = ApplicationStatus::Fail;
                self.output.case_failed(suite, case, message);
            }
        }

        Ok(())
    }

    fn on_case_skip(
        &mut self,
        sender: &mut MessageSender,
        suite_index: usize,
        case_index: usize,
        result: QueryResult,
    ) -> ApplicationResult<()> {
        let suite = &self.suites[suite_index];
        let case = &suite.cases()[case_index];

        match result {
            QueryResult::Success => self.output.case_skipped(suite, case),
            QueryResult::Fail { .. } => sender.send_case_run(suite_index, case_index, case)?,
            QueryResult::Error { ref message } => {
                self.status = ApplicationStatus::Fail;
                self.output.case_failed(suite, case, message);
            }
        }

        Ok(())
    }

    fn on_suite_skip(
        &mut self,
        sender: &mut MessageSender,
        suite_index: usize,
        result: QueryResult,
    ) -> ApplicationResult<()> {
        let suite = &self.suites[suite_index];

        match result {
            QueryResult::Success => self.output.suite_skipped(suite),
            QueryResult::Fail { .. } => {
                self.output.suite_started(suite);
                sender.send_suite(suite_index, suite)?;
            }
            QueryResult::Error { ref message } => {
                self.status = ApplicationStatus::Fail;
                self.output.suite_failed(suite, message)
            }
        }

        Ok(())
    }

    fn get_n_cases(&self) -> usize {
        self.suites.iter().map(|s| s.cases().len()).sum()
    }
}
