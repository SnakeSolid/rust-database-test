use std::fmt::Display;

use postgres::Result as PgResult;
use postgres::rows::Row;
use postgres::types::FromSql;

use dto::ColumnClause;
use dto::Condition;
use dto::NRowsClause;
use dto::Value;
use dto::Values;

use super::QueryResult;

#[inline]
pub fn assert_column(row: &Row, column: &ColumnClause) -> QueryResult {
    match *column {
        ColumnClause::Compare {
            ref name,
            condition,
            ref value,
        } => assert_column_compare(row, condition, name, value),
        ColumnClause::Range {
            ref name,
            ref from,
            ref to,
        } => assert_column_range(row, name, from, to),
        ColumnClause::Any { ref name, ref any } => assert_column_any(row, name, any),
        ColumnClause::Contains {
            ref name,
            ref contains,
        } => assert_column_contains(row, name, contains),
    }
}

#[inline]
pub fn assert_n_rows(actual_rows: usize, n_rows: &NRowsClause) -> QueryResult {
    let condition = n_rows.condition();
    let expected_rows = n_rows.value();

    assert_condition("N rows", condition, expected_rows, actual_rows)
}

#[inline]
fn assert_column_compare(row: &Row, conditon: Condition, name: &str, value: &Value) -> QueryResult {
    match *value {
        Value::Integer(ref value) => with_row_value(row, name, |actual| {
            assert_condition(format!("Column '{}'", name), conditon, value, actual)
        }),
        Value::Float(ref value) => with_row_value(row, name, |actual| {
            assert_condition(format!("Column '{}'", name), conditon, value, actual)
        }),
        Value::String(ref value) => with_row_value(row, name, |actual| {
            assert_condition(format!("Column '{}'", name), conditon, value, actual)
        }),
    }
}

#[inline]
fn assert_condition<S, T>(text: S, condition: Condition, expected: T, actual: T) -> QueryResult
where
    S: Display,
    T: PartialEq + PartialOrd + Display,
{
    match condition {
        Condition::Equal => make_query_result(
            actual == expected,
            format!("{} failed: {} == {}", text, actual, expected),
        ),
        Condition::NotEqual => make_query_result(
            actual != expected,
            format!("{} failed: {} != {}", text, actual, expected),
        ),
        Condition::Less => make_query_result(
            actual < expected,
            format!("{} failed: {} < {}", text, actual, expected),
        ),
        Condition::Greater => make_query_result(
            actual > expected,
            format!("{} failed: {} > {}", text, actual, expected),
        ),
        Condition::LessOrEqual => make_query_result(
            actual <= expected,
            format!("{} failed: {} <= {}", text, actual, expected),
        ),
        Condition::GreaterOrEqual => make_query_result(
            actual >= expected,
            format!("{} failed: {} >= {}", text, actual, expected),
        ),
    }
}

#[inline]
fn assert_column_range(row: &Row, name: &str, from: &Value, to: &Value) -> QueryResult {
    match (from, to) {
        (&Value::Integer(ref from), &Value::Integer(ref to)) => {
            with_row_value(row, name, |actual| assert_range(name, from, to, actual))
        }
        (&Value::Float(ref from), &Value::Float(ref to)) => {
            with_row_value(row, name, |actual| assert_range(name, from, to, actual))
        }
        (&Value::String(ref from), &Value::String(ref to)) => {
            with_row_value(row, name, |actual| assert_range(name, from, to, actual))
        }
        _ => QueryResult::fail("Parameters 'from' and 'to' have incompatible types"),
    }
}

#[inline]
fn assert_range<S, T>(name: S, from: T, to: T, actual: T) -> QueryResult
where
    S: Display,
    T: PartialOrd + Display,
{
    make_query_result(
        actual >= from && actual <= to,
        format!(
            "Column '{}' failed: {} in [ {} .. {} ]",
            name, actual, from, to
        ),
    )
}

#[inline]
fn assert_column_any(row: &Row, name: &str, values: &Values) -> QueryResult {
    match *values {
        Values::Integer(ref values) => {
            with_row_value(row, name, |actual| assert_any(name, values, actual))
        }
        Values::Float(ref values) => {
            with_row_value(row, name, |actual| assert_any(name, values, actual))
        }
        Values::String(ref values) => {
            with_row_value(row, name, |actual| assert_any(name, values, actual))
        }
    }
}

#[inline]
fn assert_any<S, T>(name: S, values: &[T], actual: &T) -> QueryResult
where
    S: Display,
    T: PartialEq + Display,
{
    make_query_result(
        values.iter().any(|v| v == actual),
        format!(
            "Column '{}' failed: {} any [ {} ]",
            name,
            actual,
            join_values(values, ", ")
        ),
    )
}

#[inline]
fn join_values<D>(values: &[D], separator: &str) -> String
where
    D: Display,
{
    let mut result = String::default();
    let mut it = values.iter();

    if let Some(value) = it.next() {
        result.push_str(&format!("{}", value));

        while let Some(value) = it.next() {
            result.push_str(separator);
            result.push_str(&format!("{}", value));
        }
    }

    result
}

#[inline]
fn assert_column_contains(row: &Row, name: &str, sub_string: &String) -> QueryResult {
    with_row_value(row, name, |value| assert_contains(name, value, sub_string))
}

#[inline]
fn assert_contains<S>(name: S, value: &String, sub_string: &String) -> QueryResult
where
    S: Display,
{
    make_query_result(
        value.contains(sub_string),
        format!(
            "Column '{}' failed: '{}' contains '{}'",
            name, value, sub_string
        ),
    )
}

#[inline]
fn with_row_value<F, T>(row: &Row, name: &str, callback: F) -> QueryResult
where
    F: FnOnce(&T) -> QueryResult,
    T: FromSql + Display,
{
    let actual_value: Option<PgResult<T>> = row.get_opt(name);

    match actual_value {
        None => QueryResult::fail(format!("Column {} does not exists", name)),
        Some(Err(err)) => QueryResult::fail(format!("Failed to get {} value - {}", name, err)),
        Some(Ok(ref actual_value)) => callback(actual_value),
    }
}

#[inline]
fn make_query_result<S>(condition: bool, message: S) -> QueryResult
where
    S: Into<String>,
{
    if condition {
        QueryResult::success()
    } else {
        QueryResult::fail(message)
    }
}

#[cfg(test)]
mod test {
    use serde_yaml;

    use dto::NRowsClause;

    use super::assert_n_rows;
    use super::QueryResult;

    #[test]
    fn n_rows_success_if_actual_eq_expected() {
        let n_rows: NRowsClause = serde_yaml::from_str("{ condition : =, value : 3 }").unwrap();
        let assert_2_rows = assert_n_rows(2, &n_rows);
        let assert_3_rows = assert_n_rows(3, &n_rows);
        let assert_4_rows = assert_n_rows(4, &n_rows);

        assert_eq!(QueryResult::fail("N rows failed: 2 == 3"), assert_2_rows);
        assert_eq!(QueryResult::success(), assert_3_rows);
        assert_eq!(QueryResult::fail("N rows failed: 4 == 3"), assert_4_rows);
    }

    #[test]
    fn n_rows_success_if_actual_ne_expected() {
        let n_rows: NRowsClause = serde_yaml::from_str("{ condition : '!=', value : 3 }").unwrap();
        let assert_2_rows = assert_n_rows(2, &n_rows);
        let assert_3_rows = assert_n_rows(3, &n_rows);
        let assert_4_rows = assert_n_rows(4, &n_rows);

        assert_eq!(QueryResult::success(), assert_2_rows);
        assert_eq!(QueryResult::fail("N rows failed: 3 != 3"), assert_3_rows);
        assert_eq!(QueryResult::success(), assert_4_rows);
    }

    #[test]
    fn n_rows_success_if_actual_lt_expected() {
        let n_rows: NRowsClause = serde_yaml::from_str("{ condition : <, value : 3 }").unwrap();
        let assert_2_rows = assert_n_rows(2, &n_rows);
        let assert_3_rows = assert_n_rows(3, &n_rows);
        let assert_4_rows = assert_n_rows(4, &n_rows);

        assert_eq!(QueryResult::success(), assert_2_rows);
        assert_eq!(QueryResult::fail("N rows failed: 3 < 3"), assert_3_rows);
        assert_eq!(QueryResult::fail("N rows failed: 4 < 3"), assert_4_rows);
    }

    #[test]
    fn n_rows_success_if_actual_gt_expected() {
        let n_rows: NRowsClause = serde_yaml::from_str("{ condition : >, value : 3 }").unwrap();
        let assert_2_rows = assert_n_rows(2, &n_rows);
        let assert_3_rows = assert_n_rows(3, &n_rows);
        let assert_4_rows = assert_n_rows(4, &n_rows);

        assert_eq!(QueryResult::fail("N rows failed: 2 > 3"), assert_2_rows);
        assert_eq!(QueryResult::fail("N rows failed: 3 > 3"), assert_3_rows);
        assert_eq!(QueryResult::success(), assert_4_rows);
    }

    #[test]
    fn n_rows_success_if_actual_le_expected() {
        let n_rows: NRowsClause = serde_yaml::from_str("{ condition : <=, value : 3 }").unwrap();
        let assert_2_rows = assert_n_rows(2, &n_rows);
        let assert_3_rows = assert_n_rows(3, &n_rows);
        let assert_4_rows = assert_n_rows(4, &n_rows);

        assert_eq!(QueryResult::success(), assert_2_rows);
        assert_eq!(QueryResult::success(), assert_3_rows);
        assert_eq!(QueryResult::fail("N rows failed: 4 <= 3"), assert_4_rows);
    }

    #[test]
    fn n_rows_success_if_actual_ge_expected() {
        let n_rows: NRowsClause = serde_yaml::from_str("{ condition : >=, value : 3 }").unwrap();
        let assert_2_rows = assert_n_rows(2, &n_rows);
        let assert_3_rows = assert_n_rows(3, &n_rows);
        let assert_4_rows = assert_n_rows(4, &n_rows);

        assert_eq!(QueryResult::fail("N rows failed: 2 >= 3"), assert_2_rows);
        assert_eq!(QueryResult::success(), assert_3_rows);
        assert_eq!(QueryResult::success(), assert_4_rows);
    }
}
