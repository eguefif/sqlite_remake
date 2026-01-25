#[allow(dead_code)]
pub struct Query {
    select: Vec<String>,
    pub from: String,
}

impl Query {
    pub fn new() -> Self {
        Self {
            select: vec![],
            from: "".to_string(),
        }
    }

    pub fn push_select(&mut self, value: String) {
        self.select.push(value);
    }

    pub fn set_from(&mut self, from: String) {
        self.from = from;
    }
}
