use std::mem::ManuallyDrop;

use super::{Bank, RawIter};


pub struct IntoIter<T, const C: usize> {
    _bank: ManuallyDrop<Bank<T, C>>,
    iter: RawIter<T>
}

impl <T, const C: usize> IntoIter<T, C> {
    #[inline]
    pub(super) const fn new(bank: Bank<T, C>, iter: RawIter<T>) -> Self {
        let bank = ManuallyDrop::new(bank);
        Self { _bank: bank, iter }
    } 
}

impl <T, const C: usize> Iterator for IntoIter<T, C> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl <T, const C: usize> DoubleEndedIterator for IntoIter<T, C> {
    fn next_back(&mut self) -> Option<Self::Item> { self.iter.next_back() }
}

impl <T, const C: usize> Drop for IntoIter<T, C> {
    fn drop(&mut self) { for _ in &mut *self {} }
}