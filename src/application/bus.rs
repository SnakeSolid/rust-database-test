use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;

use dto::QueryClause;
use dto::TestCase;
use dto::TestSuite;

use super::error::ApplicationError;
use super::error::ApplicationResult;
use super::worker::WorkerMessage;
use super::worker::WorkerReply;

#[derive(Debug)]
pub struct MessageBus {
    message_sender: SyncSender<WorkerMessage>,
    reply_receiver: Receiver<WorkerReply>,
    n_messages: AtomicUsize,
}

impl MessageBus {
    pub fn new(
        message_sender: SyncSender<WorkerMessage>,
        reply_receiver: Receiver<WorkerReply>,
    ) -> MessageBus {
        MessageBus {
            message_sender,
            reply_receiver,
            n_messages: AtomicUsize::new(0),
        }
    }

    pub fn message_loop<F>(self, mut callback: F) -> ApplicationResult<()>
    where
        F: FnMut(&MessageBus, WorkerReply) -> ApplicationResult<()>,
    {
        for reply in &self.reply_receiver {
            callback(&self, reply)?;

            self.n_messages.fetch_sub(1, Ordering::Relaxed);

            if self.n_messages.load(Ordering::Relaxed) == 0 {
                break;
            }
        }

        Ok(())
    }

    pub fn send_suite_skip(
        &self,
        suite_index: usize,
        clause: &QueryClause,
    ) -> ApplicationResult<()> {
        let message = WorkerMessage::suite_skip(suite_index, clause);

        self.n_messages.fetch_add(1, Ordering::Relaxed);
        self.message_sender
            .send(message)
            .map_err(ApplicationError::send_message_error)
    }

    pub fn send_suite(&self, suite_index: usize, suite: &TestSuite) -> ApplicationResult<()> {
        for (case_index, case) in suite.cases().iter().enumerate() {
            if let Some(skip) = case.skip() {
                self.send_case_skip(suite_index, case_index, skip)?;
            } else {
                self.send_case_run(suite_index, case_index, case)?;
            }
        }

        Ok(())
    }

    pub fn send_case_skip(
        &self,
        suite_index: usize,
        case_index: usize,
        clause: &QueryClause,
    ) -> ApplicationResult<()> {
        let message = WorkerMessage::case_skip(suite_index, case_index, clause);

        self.n_messages.fetch_add(1, Ordering::Relaxed);
        self.message_sender
            .send(message)
            .map_err(ApplicationError::send_message_error)
    }

    pub fn send_case_run(
        &self,
        suite_index: usize,
        case_index: usize,
        case: &TestCase,
    ) -> ApplicationResult<()> {
        let message = WorkerMessage::case_run(suite_index, case_index, case);

        self.n_messages.fetch_add(1, Ordering::Relaxed);
        self.message_sender
            .send(message)
            .map_err(ApplicationError::send_message_error)
    }
}
