use std::sync::mpsc::SyncSender;

use application::ApplicationError;
use application::ApplicationResult;
use application::WorkerMessage;
use dto::QueryClause;
use dto::TestCase;
use dto::TestSuite;

#[derive(Debug)]
pub struct MessageSender {
    sender: SyncSender<WorkerMessage>,
    n_messages: usize,
}

impl MessageSender {
    pub fn new(sender: SyncSender<WorkerMessage>) -> MessageSender {
        MessageSender {
            sender: sender,
            n_messages: 0,
        }
    }

    pub fn send_suite_skip(
        &mut self,
        suite_index: usize,
        clause: &QueryClause,
    ) -> ApplicationResult<()> {
        let message = WorkerMessage::suite_skip(suite_index, clause);

        self.inc_messages();
        self.sender
            .send(message)
            .map_err(ApplicationError::send_message_error)
    }

    pub fn send_suite(&mut self, suite_index: usize, suite: &TestSuite) -> ApplicationResult<()> {
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
        &mut self,
        suite_index: usize,
        case_index: usize,
        clause: &QueryClause,
    ) -> ApplicationResult<()> {
        let message = WorkerMessage::case_skip(suite_index, case_index, clause);

        self.inc_messages();
        self.sender
            .send(message)
            .map_err(ApplicationError::send_message_error)
    }

    pub fn send_case_run(
        &mut self,
        suite_index: usize,
        case_index: usize,
        case: &TestCase,
    ) -> ApplicationResult<()> {
        let message = WorkerMessage::case_run(suite_index, case_index, case);

        self.inc_messages();
        self.sender
            .send(message)
            .map_err(ApplicationError::send_message_error)
    }

    fn inc_messages(&mut self) {
        self.n_messages += 1;
    }

    pub fn dec_messages(&mut self) {
        self.n_messages -= 1;
    }

    pub fn has_messages(&self) -> bool {
        self.n_messages == 0
    }
}
