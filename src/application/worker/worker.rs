use std::fmt::Display;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::thread::Builder;
use std::thread::JoinHandle;

use postgres::Connection;
use postgres::Result as PgResult;
use postgres::rows::Row;
use postgres::TlsMode;

use dto::ColumnClause;
use dto::Condition;
use dto::NRowsClause;
use dto::QueryClause;
use dto::TestCase;
use dto::Value;

use super::QueryResult;
use super::WorkerError;
use super::WorkerMessage;
use super::WorkerReply;
use super::WorkerResult;

#[derive(Debug)]
pub struct Worker {
    message_channel: Receiver<WorkerMessage>,
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

impl Worker {
    pub fn new(
        message_channel: Receiver<WorkerMessage>,
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
        for message in self.message_channel {
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

    fn execute_case(connection: &Connection, case: &TestCase) -> QueryResult {
        let query = case.query();
        let rows = match connection.query(query, &[]) {
            Ok(rows) => rows,
            Err(err) => return err.into(),
        };

        if let Some(n_rows) = case.n_rows() {
            query_result!(Worker::assert_n_rows(rows.len(), n_rows));
        }

        if !case.columns().is_empty() {
            for row in &rows {
                for column in case.columns() {
                    query_result!(Worker::assert_column(&row, column));
                }
            }
        }

        QueryResult::Success
    }

    fn execute_clause(connection: &Connection, clause: &QueryClause) -> QueryResult {
        let query = clause.query();
        let rows = match connection.query(query, &[]) {
            Ok(rows) => rows,
            Err(err) => return err.into(),
        };
        let actual_rows = rows.len();

        Worker::assert_n_rows(actual_rows, clause.n_rows())
    }

    fn assert_column(row: &Row, column: &ColumnClause) -> QueryResult {
        let value = column.value();

        match *value {
            Value::Integer(ref value) => Worker::assert_column_integer(row, column, *value),
            Value::Float(ref value) => Worker::assert_column_float(row, column, *value),
            Value::String(ref value) => Worker::assert_column_string(row, column, value),
        }
    }

    fn assert_column_string(
        row: &Row,
        column: &ColumnClause,
        expected_value: &String,
    ) -> QueryResult {
        let condition = column.condition();
        let name = column.name();
        let actual_value: Option<PgResult<String>> = row.get_opt(name);

        match actual_value {
            None => QueryResult::fail(format!("Column {} does not exists", name)),
            Some(Err(err)) => QueryResult::fail(format!("Failed to get {} value - {}", name, err)),
            Some(Ok(ref actual_value)) => Worker::test_condition(
                format!("Column {}", name),
                condition,
                expected_value,
                actual_value,
            ),
        }
    }

    fn assert_column_float(row: &Row, column: &ColumnClause, expected_value: f64) -> QueryResult {
        let condition = column.condition();
        let name = column.name();
        let actual_value: Option<PgResult<f64>> = row.get_opt(name);

        match actual_value {
            None => QueryResult::fail(format!("Column {} does not exists", name)),
            Some(Err(err)) => QueryResult::fail(format!("Failed to get {} value - {}", name, err)),
            Some(Ok(actual_value)) => Worker::test_condition(
                format!("Column {}", name),
                condition,
                expected_value,
                actual_value,
            ),
        }
    }

    fn assert_column_integer(row: &Row, column: &ColumnClause, expected_value: i64) -> QueryResult {
        let condition = column.condition();
        let name = column.name();
        let actual_value: Option<PgResult<i64>> = row.get_opt(name);

        match actual_value {
            None => QueryResult::fail(format!("Column {} does not exists", name)),
            Some(Err(err)) => QueryResult::fail(format!("Failed to get {} value - {}", name, err)),
            Some(Ok(actual_value)) => Worker::test_condition(
                format!("Column {}", name),
                condition,
                expected_value,
                actual_value,
            ),
        }
    }

    fn assert_n_rows(actual_rows: usize, n_rows: &NRowsClause) -> QueryResult {
        let condition = n_rows.condition();
        let expected_rows = n_rows.value();

        Worker::test_condition("N rows", condition, expected_rows, actual_rows)
    }

    fn test_condition<S, T>(name: S, condition: Condition, expected: T, actual: T) -> QueryResult
    where
        S: Display,
        T: PartialEq + PartialOrd + Display,
    {
        match condition {
            Condition::Equal => QueryResult::from_condition(
                actual == expected,
                format!("{} failed: {} == {}", name, actual, expected),
            ),
            Condition::NotEqual => QueryResult::from_condition(
                actual != expected,
                format!("{} failed: {} != {}", name, actual, expected),
            ),
            Condition::Less => QueryResult::from_condition(
                actual < expected,
                format!("{} failed: {} < {}", name, actual, expected),
            ),
            Condition::Greater => QueryResult::from_condition(
                actual > expected,
                format!("{} failed: {} > {}", name, actual, expected),
            ),
            Condition::LessOrEqual => QueryResult::from_condition(
                actual <= expected,
                format!("{} failed: {} <= {}", name, actual, expected),
            ),
            Condition::GreaterOrEqual => QueryResult::from_condition(
                actual >= expected,
                format!("{} failed: {} >= {}", name, actual, expected),
            ),
        }
    }
}
