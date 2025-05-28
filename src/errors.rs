use std::fmt;


#[derive(Debug, Clone)]
pub struct BankFullError {}

impl fmt::Display for BankFullError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bank is full")
    }
}