use std::borrow::Borrow;
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
    let name: &str = column.name().borrow();
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
        Condition::Equal => make_query_result(
            actual == expected,
            format!("{} failed: {} == {}", name, actual, expected),
        ),
        Condition::NotEqual => make_query_result(
            actual != expected,
            format!("{} failed: {} != {}", name, actual, expected),
        ),
        Condition::Less => make_query_result(
            actual < expected,
            format!("{} failed: {} < {}", name, actual, expected),
        ),
        Condition::Greater => make_query_result(
            actual > expected,
            format!("{} failed: {} > {}", name, actual, expected),
        ),
        Condition::LessOrEqual => make_query_result(
            actual <= expected,
            format!("{} failed: {} <= {}", name, actual, expected),
        ),
        Condition::GreaterOrEqual => make_query_result(
            actual >= expected,
            format!("{} failed: {} >= {}", name, actual, expected),
        ),
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
