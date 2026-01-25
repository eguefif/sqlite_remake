pub struct Query {
    select: String,
    from: String,
}

impl Query {
    pub fn new(query_str: String) -> Self {
        let mut splits = query_str.split(' ');
        let mut select = String::new();
        let from;
        loop {
            let Some(split) = splits.next() else {
                panic!("Except From in Query");
            };
            if split.to_lowercase() == "from" {
                from = splits.next().unwrap().to_string();
                break;
            } else {
                select.push_str(split);
            }
        }
        Self { select, from }
    }
}
