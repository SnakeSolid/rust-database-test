use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;

use dto::QueryClause;
use dto::TestSuite;

use super::error::ApplicationResult;
use super::worker::WorkerMessage;
use super::worker::WorkerReply;

mod sender;

pub use self::sender::MessageSender;

#[derive(Debug)]
pub struct MessageBus {
    message_sender: MessageSender,
    reply_receiver: Receiver<WorkerReply>,
}

impl MessageBus {
    pub fn new(
        message_sender: SyncSender<WorkerMessage>,
        reply_receiver: Receiver<WorkerReply>,
    ) -> MessageBus {
        MessageBus {
            message_sender: MessageSender::new(message_sender),
            reply_receiver,
        }
    }

    pub fn send_suite_skip(
        &mut self,
        suite_index: usize,
        clause: &QueryClause,
    ) -> ApplicationResult<()> {
        self.message_sender.send_suite_skip(suite_index, clause)
    }

    pub fn send_suite(&mut self, suite_index: usize, suite: &TestSuite) -> ApplicationResult<()> {
        self.message_sender.send_suite(suite_index, suite)
    }

    pub fn message_loop<F>(mut self, mut callback: F) -> ApplicationResult<()>
    where
        F: FnMut(&mut MessageSender, WorkerReply) -> ApplicationResult<()>,
    {
        if self.message_sender.has_messages() {
            for reply in &self.reply_receiver {
                callback(&mut self.message_sender, reply)?;

                self.message_sender.dec_messages();

                if !self.message_sender.has_messages() {
                    break;
                }
            }
        }

        Ok(())
    }
}
