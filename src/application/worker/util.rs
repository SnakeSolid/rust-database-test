use std::fmt::Display;

use postgres::Result as PgResult;
use postgres::rows::Row;
use postgres::types::FromSql;

use dto::ColumnClause;
use dto::Condition;
use dto::NRowsClause;
use dto::Value;

use super::QueryResult;

#[inline]
pub fn assert_column(row: &Row, column: &ColumnClause) -> QueryResult {
    let value = column.value();

    match *value {
        Value::Integer(ref value) => assert_column_value(row, column, value),
        Value::Float(ref value) => assert_column_value(row, column, value),
        Value::String(ref value) => assert_column_value(row, column, value),
    }
}

#[inline]
pub fn assert_n_rows(actual_rows: usize, n_rows: &NRowsClause) -> QueryResult {
    let condition = n_rows.condition();
    let expected_rows = n_rows.value();

    assert_condition("N rows", condition, expected_rows, actual_rows)
}

#[inline]
fn assert_column_value<T>(row: &Row, column: &ColumnClause, expected_value: &T) -> QueryResult
where
    T: FromSql + PartialEq + PartialOrd + Display,
{
    let condition = column.condition();
    let name = column.name();
    let actual_value: Option<PgResult<T>> = row.get_opt(name);

    match actual_value {
        None => QueryResult::fail(format!("Column {} does not exists", name)),
        Some(Err(err)) => QueryResult::fail(format!("Failed to get {} value - {}", name, err)),
        Some(Ok(ref actual_value)) => assert_condition(
            format!("Column {}", name),
            condition,
            expected_value,
            actual_value,
        ),
    }
}

#[inline]
fn assert_condition<S, T>(name: S, condition: Condition, expected: T, actual: T) -> QueryResult
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
