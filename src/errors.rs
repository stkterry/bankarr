use std::fmt;


#[derive(Debug, Clone)]
pub struct BankFullError {}

impl fmt::Display for BankFullError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bank is full")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let err = BankFullError {};

        assert_eq!(err.to_string(), "bank is full");
    }
}
