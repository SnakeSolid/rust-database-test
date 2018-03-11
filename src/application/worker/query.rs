use std::error::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum QueryResult {
    Success,
    Fail { message: String },
    Error { message: String },
}

impl QueryResult {
    #[inline]
    pub fn success() -> QueryResult {
        QueryResult::Success
    }

    #[inline]
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
        QueryResult::Error {
            message: format!("{}", error),
        }
    }
}
