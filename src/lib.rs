mod bank;

use std::{ hint::unreachable_unchecked };

pub use bank::Bank;



enum Slot<T, const C: usize> {
    Bank(Bank<T,C>),
    Next(usize),
    Empty
}
impl <T, const C: usize> Slot<T, C> {
    unsafe fn as_bank_unchecked(&mut self) -> &mut Bank<T, C> {
        match self {
            Slot::Bank(bank) => bank,
            _ => unsafe { unreachable_unchecked() }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;


}
