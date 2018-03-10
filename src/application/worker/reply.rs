use super::query::QueryResult;

#[derive(Debug)]
pub enum WorkerReply {
    SuiteSkip {
        suite_index: usize,
        result: QueryResult,
    },
    CaseSkip {
        suite_index: usize,
        case_index: usize,
        result: QueryResult,
    },
    CaseRun {
        suite_index: usize,
        case_index: usize,
        result: QueryResult,
    },
}

impl WorkerReply {
    pub fn suite_skip(suite_index: usize, result: QueryResult) -> WorkerReply {
        WorkerReply::SuiteSkip {
            suite_index,
            result,
        }
    }

    pub fn case_skip(suite_index: usize, case_index: usize, result: QueryResult) -> WorkerReply {
        WorkerReply::CaseSkip {
            suite_index,
            case_index,
            result,
        }
    }

    pub fn case_run(suite_index: usize, case_index: usize, result: QueryResult) -> WorkerReply {
        WorkerReply::CaseRun {
            suite_index,
            case_index,
            result,
        }
    }
}
