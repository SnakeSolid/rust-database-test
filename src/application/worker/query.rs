use std::error::Error;

#[derive(Debug)]
pub enum QueryResult {
    Success,
    Fail { message: String },
}

impl QueryResult {
    pub fn from_condition<S>(condition: bool, message: S) -> QueryResult
    where
        S: Into<String>,
    {
        if condition {
            QueryResult::Success
        } else {
            QueryResult::Fail {
                message: message.into(),
            }
        }
    }

    pub fn fail<S>(message: S) -> QueryResult
    where
        S: Into<String>,
    {
        QueryResult::Fail {
            message: message.into(),
        }
    }
}

impl<E> From<E> for QueryResult
where
    E: Error,
{
    fn from(error: E) -> QueryResult {
        QueryResult::Fail {
            message: format!("{}", error),
        }
    }
}
