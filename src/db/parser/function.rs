use crate::db::parser::select::SelectItem;
use itertools::Itertools;
use std::fmt;

#[derive(Debug)]
pub struct FuncCall {
    function_name: String,
    params: Vec<SelectItem>,
}

impl FuncCall {
    pub fn new(function_name: String, params: Vec<SelectItem>) -> Self {
        Self {
            function_name,
            params,
        }
    }
}

impl fmt::Display for FuncCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let identifiers = self.params.iter().join(", ");
        write!(f, "{}({})", self.function_name.to_uppercase(), identifiers)
    }
}
