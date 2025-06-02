use std::hint::unreachable_unchecked;

use crate::Bank;


enum Slot<T, const C: usize> {
    Bank(Bank<T, C>),
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


pub struct Banklist<T, const C: usize> {
    banks: Vec<Slot<T, C>>,
    next_available: Slot<T, C>,
}

impl <T, const C: usize> Banklist<T, C> {

    #[inline]
    pub const fn new() -> Self {
        Self { 
            banks: Vec::new(),
            next_available: Slot::Empty,
        }
    }

    // #[inline]
    // pub fn push(&mut self) -> usize {
        
    // }

}