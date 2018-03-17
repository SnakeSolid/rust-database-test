use std::fs::File;
use std::fs;
use std::path::Path;

use serde_yaml;

use application::ApplicationError;
use application::ApplicationResult;
use config::Configuration;
use dto::TestSuite;

#[derive(Debug)]
pub struct SuiteReader<'a> {
    config: &'a Configuration,
    suites: Vec<TestSuite>,
}

impl<'a> SuiteReader<'a> {
    pub fn new(config: &'a Configuration) -> SuiteReader<'a> {
        SuiteReader {
            config,
            suites: Vec::default(),
        }
    }

    pub fn read(mut self) -> ApplicationResult<Vec<TestSuite>> {
        let filter = self.config.filter();
        let recursive = self.config.recursive();
        let extensions = self.config.extensions();

        for path in self.config.suites() {
            if path.is_file() {
                let reader = File::open(path).map_err(ApplicationError::suite_io_error)?;
                let suite =
                    serde_yaml::from_reader(reader).map_err(ApplicationError::suite_yaml_error)?;

                if match_filter(&suite, filter) {
                    self.suites.push(suite);
                }
            } else if recursive && path.is_dir() {
                self.read_recursively(path, filter, extensions)?;
            } else {
                return Err(ApplicationError::suite_is_directory(path));
            }
        }

        if !self.suites.is_empty() {
            Ok(self.suites)
        } else {
            Err(ApplicationError::no_suites_found())
        }
    }

    fn read_recursively(
        &mut self,
        dir_path: &Path,
        filter: Option<&String>,
        extensions: Option<&Vec<String>>,
    ) -> ApplicationResult<()> {
        for entry in fs::read_dir(dir_path).map_err(ApplicationError::directory_io_error)? {
            let entry = entry.map_err(ApplicationError::directory_io_error)?;
            let metadata = entry
                .metadata()
                .map_err(ApplicationError::directory_io_error)?;
            let path = entry.path();

            if metadata.is_file() && is_extension_matches(&path, extensions) {
                let reader = File::open(path).map_err(ApplicationError::suite_io_error)?;
                let suite =
                    serde_yaml::from_reader(reader).map_err(ApplicationError::suite_yaml_error)?;

                if match_filter(&suite, filter) {
                    self.suites.push(suite);
                }
            } else if metadata.is_dir() {
                self.read_recursively(&path, filter, extensions)?;
            }
        }

        Ok(())
    }
}

fn match_filter(suite: &TestSuite, filter: Option<&String>) -> bool {
    match filter {
        Some(filter) if suite.name().contains(filter) => true,
        Some(_) => false,
        None => true,
    }
}

fn is_extension_matches(file_path: &Path, extensions: Option<&Vec<String>>) -> bool {
    match (extensions, file_path.extension()) {
        (Some(extensions), Some(file_extension)) => {
            extensions.iter().any(|e| e.as_str() == file_extension)
        }
        (Some(_), None) => false,
        (None, _) => true,
    }
}

#[cfg(test)]
mod test {
    use serde_yaml;

    use dto::TestSuite;

    use super::match_filter;

    #[test]
    fn shold_match_filter_when_filter_none() {
        let suite: TestSuite = serde_yaml::from_str("{ name : abcdef, cases : [] }").unwrap();
        let filter = None;

        assert!(match_filter(&suite, filter));
    }

    #[test]
    fn shold_match_filter_when_filter_some_left() {
        let suite: TestSuite = serde_yaml::from_str("{ name : abcdef, cases : [] }").unwrap();
        let filter = "abcd".into();
        let filter = Some(&filter);

        assert!(match_filter(&suite, filter));
    }

    #[test]
    fn shold_match_filter_when_filter_some_right() {
        let suite: TestSuite = serde_yaml::from_str("{ name : abcdef, cases : [] }").unwrap();
        let filter = "cdef".into();
        let filter = Some(&filter);

        assert!(match_filter(&suite, filter));
    }

    #[test]
    fn shold_not_match_filter_when_filter_some_else() {
        let suite: TestSuite = serde_yaml::from_str("{ name : abcdef, cases : [] }").unwrap();
        let filter = "defg".into();
        let filter = Some(&filter);

        assert!(!match_filter(&suite, filter));
    }
}
