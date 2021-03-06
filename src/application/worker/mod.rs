use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::Mutex;
use std::thread::Builder;
use std::thread::JoinHandle;

use postgres::Connection;
use postgres::TlsMode;

use dto::QueryClause;
use dto::TestCase;

mod error;
mod message;
mod query;
mod reply;
mod util;

pub use self::error::WorkerError;
pub use self::error::WorkerResult;
pub use self::message::WorkerMessage;
pub use self::query::QueryResult;
pub use self::reply::WorkerReply;

#[derive(Debug)]
pub struct Worker {
    message_channel: Arc<Mutex<Receiver<WorkerMessage>>>,
    reply_channel: SyncSender<WorkerReply>,
    hostname: String,
    port: u16,
    database: String,
    username: String,
    password: String,
}

macro_rules! query_result {
    ($e : expr) => {
        match $e {
            QueryResult::Success => {}
            _ => return $e,
        }
    }
}

macro_rules! try_query_result {
    ($e : expr) => {
        match $e {
            Ok(result) => result,
            Err(err) => return err.into(),
        }
    }
}

impl Worker {
    pub fn new(
        message_channel: Arc<Mutex<Receiver<WorkerMessage>>>,
        reply_channel: SyncSender<WorkerReply>,
        hostname: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
    ) -> Worker {
        Worker {
            message_channel,
            reply_channel,
            hostname: hostname.into(),
            port,
            database: database.into(),
            username: username.into(),
            password: password.into(),
        }
    }

    pub fn start(self) -> WorkerResult<JoinHandle<()>> {
        let url = format!(
            "postgresql://{3}:{4}@{0}:{1}/{2}",
            self.hostname, self.port, self.database, self.username, self.password,
        );
        let connection = Connection::connect(url, TlsMode::None)?;

        Builder::new()
            .spawn(|| self.run(connection))
            .map_err(WorkerError::spawn_io_error)
    }

    fn run(self, connection: Connection) {
        while let Ok(message) = self.next_message() {
            match message {
                WorkerMessage::SuiteSkip {
                    suite_index,
                    ref clause,
                } => {
                    let result = Worker::execute_clause(&connection, clause);
                    let reply = WorkerReply::suite_skip(suite_index, result);

                    self.reply_channel.send(reply).unwrap();
                }
                WorkerMessage::CaseSkip {
                    suite_index,
                    case_index,
                    ref clause,
                } => {
                    let result = Worker::execute_clause(&connection, clause);
                    let reply = WorkerReply::case_skip(suite_index, case_index, result);

                    self.reply_channel.send(reply).unwrap();
                }
                WorkerMessage::CaseRun {
                    suite_index,
                    case_index,
                    ref case,
                } => {
                    let result = Worker::execute_case(&connection, case);
                    let reply = WorkerReply::case_run(suite_index, case_index, result);

                    self.reply_channel.send(reply).unwrap();
                }
            }
        }
    }

    fn next_message(&self) -> Result<WorkerMessage, ()> {
        let guard = self.message_channel.lock().map_err(|_| ())?;

        guard.recv().map_err(|_| ())
    }

    fn execute_case(connection: &Connection, case: &TestCase) -> QueryResult {
        let query = case.query();
        let transaction = try_query_result!(connection.transaction());
        transaction.set_rollback();

        let rows = try_query_result!(transaction.query(query, &[]));

        if let Some(n_rows) = case.n_rows() {
            query_result!(util::assert_n_rows(rows.len(), n_rows));
        }

        if !case.columns().is_empty() {
            for row in &rows {
                for column in case.columns() {
                    query_result!(util::assert_column(&row, column));
                }
            }
        }

        QueryResult::Success
    }

    fn execute_clause(connection: &Connection, clause: &QueryClause) -> QueryResult {
        let query = clause.query();
        let rows = try_query_result!(connection.query(query, &[]));
        let actual_rows = rows.len();

        util::assert_n_rows(actual_rows, clause.n_rows())
    }
}
