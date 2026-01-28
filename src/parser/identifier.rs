use std::fmt;

#[derive(Debug)]
pub struct Identifier {
    pub value: VType,
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            VType::Num(num) => write!(f, "{}", num),
            VType::Str(ref value) => write!(f, "{}", value),
            VType::Null => write!(f, "NULL"),
        }
    }
}

#[derive(Debug)]
pub enum VType {
    Num(i64),
    Str(String),
    Null,
}
