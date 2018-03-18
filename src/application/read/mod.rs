use std::fs::File;
use std::fs;
use std::path::Path;

use serde_yaml;

use application::ApplicationError;
use application::ApplicationResult;
use config::Configuration;
use dto::TestSuite;

mod filter;

use self::filter::Filter;

#[derive(Debug)]
pub struct SuiteReader<'a> {
    config: &'a Configuration,
    filter: Box<Filter>,
    suites: Vec<TestSuite>,
}

impl<'a> SuiteReader<'a> {
    pub fn new(config: &'a Configuration) -> SuiteReader<'a> {
        SuiteReader {
            config,
            filter: filter::create_filter(config),
            suites: Vec::default(),
        }
    }

    pub fn read(mut self) -> ApplicationResult<Vec<TestSuite>> {
        let recursive = self.config.recursive();
        let extensions = self.config.extensions();

        for path in self.config.suites() {
            if path.is_file() {
                let reader = File::open(path).map_err(ApplicationError::suite_io_error)?;
                let suite =
                    serde_yaml::from_reader(reader).map_err(ApplicationError::suite_yaml_error)?;

                self.add_suite(suite);
            } else if recursive && path.is_dir() {
                self.read_recursively(path, extensions)?;
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

    fn add_suite(&mut self, suite: TestSuite) {
        if self.filter.start_suite(&suite) {
            self.suites.push(suite);
        }
    }

    fn read_recursively(
        &mut self,
        dir_path: &Path,
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

                self.add_suite(suite);
            } else if metadata.is_dir() {
                self.read_recursively(&path, extensions)?;
            }
        }

        Ok(())
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
