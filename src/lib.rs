mod bank;
mod banklist;

pub(crate)mod errors;

use std::{ hint::unreachable_unchecked };

pub use bank::Bank;
pub use banklist::Banklist;







#[cfg(test)]
mod tests {
    use super::*;


}
