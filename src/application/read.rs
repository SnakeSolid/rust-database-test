use std::fs::File;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde_yaml;

use dto::TestSuite;

use super::error::ApplicationError;
use super::error::ApplicationResult;

#[derive(Debug)]
pub struct SuiteReader {
    suites: Vec<TestSuite>,
}

impl SuiteReader {
    pub fn read(
        &mut self,
        paths: &[PathBuf],
        recursive: bool,
        extensions: Option<&Vec<String>>,
    ) -> ApplicationResult<()> {
        for path in paths {
            if path.is_file() {
                let reader = File::open(path).map_err(ApplicationError::suite_io_error)?;
                let suite =
                    serde_yaml::from_reader(reader).map_err(ApplicationError::suite_yaml_error)?;

                self.suites.push(suite);
            } else if recursive && path.is_dir() {
                self.read_recursively(path, extensions)?;
            } else {
                return Err(ApplicationError::suite_is_directory(path));
            }
        }

        Ok(())
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

                self.suites.push(suite);
            } else if metadata.is_dir() {
                self.read_recursively(&path, extensions)?;
            }
        }

        Ok(())
    }

    pub fn suites(self) -> Vec<TestSuite> {
        self.suites
    }
}

impl Default for SuiteReader {
    fn default() -> SuiteReader {
        SuiteReader {
            suites: Vec::default(),
        }
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
