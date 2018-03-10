use dto::QueryClause;
use dto::TestCase;

#[derive(Debug)]
pub enum WorkerMessage {
    SuiteSkip {
        suite_index: usize,
        clause: QueryClause,
    },
    CaseSkip {
        suite_index: usize,
        case_index: usize,
        clause: QueryClause,
    },
    CaseRun {
        suite_index: usize,
        case_index: usize,
        case: TestCase,
    },
}

impl WorkerMessage {
    pub fn suite_skip(suite_index: usize, clause: &QueryClause) -> WorkerMessage {
        WorkerMessage::SuiteSkip {
            suite_index,
            clause: clause.clone(),
        }
    }

    pub fn case_skip(suite_index: usize, case_index: usize, clause: &QueryClause) -> WorkerMessage {
        WorkerMessage::CaseSkip {
            suite_index,
            case_index,
            clause: clause.clone(),
        }
    }

    pub fn case_run(suite_index: usize, case_index: usize, case: &TestCase) -> WorkerMessage {
        WorkerMessage::CaseRun {
            suite_index,
            case_index,
            case: case.clone(),
        }
    }
}
