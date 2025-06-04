use std::marker::PhantomData;
use super::RawIter;

pub struct Drain<'a, T: 'a> {
    _slice: PhantomData<&'a mut [T]>,
    iter: RawIter<T>
}

impl <'a, T: 'a> Drain<'a, T> {
    #[inline]
    pub(super) const fn new(iter: RawIter<T>) -> Self {
        Self { _slice: PhantomData, iter }
    }
}

impl <'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl <'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> { self.iter.next_back() }
}

impl <'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) { for _ in &mut *self { } }
}


