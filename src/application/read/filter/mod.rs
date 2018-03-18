use std::fmt::Debug;

use config::Configuration;
use dto::TestSuite;

mod empty;
mod simple;

use self::empty::EmptyFilter;
use self::simple::SimpleFilter;

pub fn create_filter(config: &Configuration) -> Box<Filter> {
    if let Some(ref filter) = config.filter() {
        Box::new(SimpleFilter::create(filter))
    } else {
        Box::new(EmptyFilter::default())
    }
}

pub trait Filter: Debug {
    fn start_suite(&self, suite: &TestSuite) -> bool;
}
