#[derive(Debug)]
pub struct TestCounters {
    passed: usize,
    skipped: usize,
    failed: usize,
}

impl TestCounters {
    pub fn inc_passed(&mut self) {
        self.passed += 1;
    }

    pub fn inc_skipped(&mut self) {
        self.skipped += 1;
    }

    pub fn inc_failed(&mut self) {
        self.failed += 1;
    }

    pub fn add_skipped(&mut self, value: usize) {
        self.skipped += value;
    }

    pub fn add_failed(&mut self, value: usize) {
        self.failed += value;
    }

    pub fn passed(&self) -> usize {
        self.passed
    }

    pub fn skipped(&self) -> usize {
        self.skipped
    }

    pub fn failed(&self) -> usize {
        self.failed
    }
}

impl Default for TestCounters {
    fn default() -> TestCounters {
        TestCounters {
            passed: 0,
            skipped: 0,
            failed: 0,
        }
    }
}
