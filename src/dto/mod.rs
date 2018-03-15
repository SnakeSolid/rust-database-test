#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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
#[serde(untagged)]
pub enum Values {
    Integer(Vec<i64>),
    Float(Vec<f64>),
    String(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ColumnClause {
    Compare {
        name: String,
        condition: Condition,
        value: Value,
    },
    Range {
        name: String,
        from: Value,
        to: Value,
    },
    Any {
        name: String,
        any: Values,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NRowsClause {
    condition: Condition,
    value: usize,
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
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::default")]
    columns: Vec<ColumnClause>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestSuite {
    name: String,
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] skip: Option<QueryClause>,
    #[serde(skip_serializing_if = "Vec::is_empty")] cases: Vec<TestCase>,
}

impl TestSuite {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn skip(&self) -> Option<&QueryClause> {
        self.skip.as_ref()
    }

    pub fn cases(&self) -> &Vec<TestCase> {
        &self.cases
    }
}

impl TestCase {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn skip(&self) -> Option<&QueryClause> {
        self.skip.as_ref()
    }

    pub fn query(&self) -> &String {
        &self.query
    }

    pub fn n_rows(&self) -> Option<&NRowsClause> {
        self.n_rows.as_ref()
    }

    pub fn columns(&self) -> &Vec<ColumnClause> {
        &self.columns
    }
}

impl QueryClause {
    pub fn query(&self) -> &String {
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
