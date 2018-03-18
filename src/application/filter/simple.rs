use dto::TestSuite;

pub use super::Filter;

const SEPARATOR: char = ',';

#[derive(Debug)]
pub struct SimpleFilter {
    filter: Vec<String>,
}

impl SimpleFilter {
    pub fn create(filter_expression: &str) -> SimpleFilter {
        SimpleFilter {
            filter: filter_expression
                .split(SEPARATOR)
                .filter(|e| !e.is_empty())
                .map(|e| e.into())
                .collect(),
        }
    }
}

impl Filter for SimpleFilter {
    fn start_suite(&self, suite: &TestSuite) -> bool {
        let suite_name = suite.name();

        self.filter.iter().any(|e| suite_name.contains(e))
    }
}
