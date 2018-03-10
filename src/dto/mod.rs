macro_rules! to_option_ref {
    ($e : expr) => {
        match $e {
            Some(ref value) => Some(value),
            None => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Condition {
    #[serde(rename = "=")] Equal,
    #[serde(rename = "!=")] NotEqual,
    #[serde(rename = "<")] Less,
    #[serde(rename = ">")] Greater,
    #[serde(rename = "<=")] LessOrEqual,
    #[serde(rename = ">=")] GreaterOrEqual,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NRowsClause {
    condition: Condition,
    value: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnClause {
    condition: Condition,
    name: String,
    value: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryClause {
    query: String,
    n_rows: NRowsClause,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestCase {
    name: String,
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] skip: Option<QueryClause>,
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")] n_rows: Option<NRowsClause>,
    #[serde(skip_serializing_if = "Vec::is_empty")] columns: Vec<ColumnClause>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestSuite {
    name: String,
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] skip: Option<QueryClause>,
    #[serde(skip_serializing_if = "Vec::is_empty")] cases: Vec<TestCase>,
}

impl TestSuite {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        to_option_ref!(self.description)
    }

    pub fn skip(&self) -> Option<&QueryClause> {
        to_option_ref!(self.skip)
    }

    pub fn cases(&self) -> &Vec<TestCase> {
        &self.cases
    }
}

impl TestCase {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        to_option_ref!(self.description)
    }

    pub fn skip(&self) -> Option<&QueryClause> {
        to_option_ref!(self.skip)
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn n_rows(&self) -> Option<&NRowsClause> {
        to_option_ref!(self.n_rows)
    }

    pub fn columns(&self) -> &Vec<ColumnClause> {
        &self.columns
    }
}

impl QueryClause {
    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn n_rows(&self) -> &NRowsClause {
        &self.n_rows
    }
}

impl NRowsClause {
    pub fn condition(&self) -> Condition {
        self.condition.clone()
    }

    pub fn value(&self) -> usize {
        self.value
    }
}

impl ColumnClause {
    pub fn condition(&self) -> Condition {
        self.condition.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}
