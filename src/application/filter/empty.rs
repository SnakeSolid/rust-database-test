use dto::TestSuite;

pub use super::Filter;

#[derive(Debug)]
pub struct EmptyFilter {}

impl Default for EmptyFilter {
    fn default() -> EmptyFilter {
        EmptyFilter {}
    }
}

impl Filter for EmptyFilter {
    fn start_suite(&self, _: &TestSuite) -> bool {
        true
    }
}
