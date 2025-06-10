use std::{fmt::Debug, iter::FusedIterator, ops, ptr::{self, NonNull}, slice};


#[inline]
const fn ptr_copy<'a, T>(elt: &'a T) -> T { unsafe { ptr::read(elt as *const T) } }


// This function was effectively pulled verbatim from the unstable `slice_range`
// feature in `core::slice::index`
#[inline]
pub(crate) fn slice_range<R>(range: R, bounds: ops::RangeTo<usize>) -> ops::Range<usize>
where
    R: ops::RangeBounds<usize>,
{

    let len = bounds.end;

    let start = match range.start_bound() {
        ops::Bound::Included(&start) => start,
        ops::Bound::Unbounded => 0,
        _ => unreachable!(),
        //ops::Bound::Excluded(start) => start.checked_add(1)
        //    .expect("attempted to index slice from after maximum usize"),
    };

    let end = match range.end_bound() {
        ops::Bound::Included(end) => end.checked_add(1)
            .expect("attempted to index slice up to maximum usize"),
        ops::Bound::Excluded(&end) => end,
        ops::Bound::Unbounded => len,
    };

    if start > end {
        panic!("slice index starts at {start} but ends at {end}")
    }

    if end > len {
        panic!("range end index {end} out of range for slice of length {len}")
    }

    ops::Range { start, end }
}


pub trait Drainable<'a, T> {
    fn drain_parts(&'a mut self) -> (NonNull<T>, &'a mut usize);
}

pub struct Drain<'a, T, B: 'a + Drainable<'a, T>> {
    pub(super) tail_start: usize,
    pub(super) tail_len: usize,
    pub(super) iter: slice::Iter<'a, T>,
    pub(super) bank: NonNull<B>,
}

#[cfg(not(tarpaulin_include))]
impl<'a, T: 'a + Debug, B: Drainable<'a, T>> Debug for Drain<'a, T, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Drain").field(&self.iter.as_slice()).finish()
    }
}

unsafe impl<'a, T: Sync, B: Drainable<'a, T>> Sync for Drain<'a, T, B> {}
unsafe impl<'a, T: Send, B: Drainable<'a, T>> Send for Drain<'a, T, B> {}

impl<'a, T: 'a, B: Drainable<'a, T>> Iterator for Drain<'a, T, B> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> { self.iter.next().map(ptr_copy) }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, T: 'a, B: Drainable<'a, T>> DoubleEndedIterator for Drain<'a, T, B> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(ptr_copy)
    }
}

impl<'a, T: 'a, B: Drainable<'a, T>> ExactSizeIterator for Drain<'a, T, B> {
    #[inline]
    fn len(&self) -> usize { self.iter.len() }
}

impl<'a, T: 'a, B: Drainable<'a, T>> FusedIterator for Drain<'a, T, B> {}

impl<'a, T: 'a, B: Drainable<'a, T>> Drop for Drain<'a, T, B> {
    fn drop(&mut self) {
        self.for_each(drop);

        if self.tail_len > 0 {
            let (ptr, len) = unsafe { self.bank.as_mut().drain_parts() };
            let start = *len;
            let tail = self.tail_start;

            if tail != start {
                unsafe { ptr.add(start).copy_from(ptr.add(tail), self.tail_len) }
            }

            *len = start + self.tail_len;
        }
    }
}


#[cfg(test)]
mod tests {

    use crate::{BankArr, BankVec};

    use super::*;
    use std::panic;

    #[test]
    fn slice_range_() {

        // unbounded start, unbounded end
        assert_eq!(slice_range(.., ..10), ops::Range { start: 0, end: 10 });

        // bounded start, excluded end
        assert_eq!(slice_range(1..5, ..10), ops::Range { start: 1, end: 5 });

        // bounded start, included end
        assert_eq!(slice_range(1..=5, ..10), ops::Range { start: 1, end: 6 });

        // start is greater than end
        assert!(panic::catch_unwind(|| slice_range(5..0, ..10)).is_err());

        // end is greater than limit
        assert!(panic::catch_unwind(|| slice_range(0..11, ..10)).is_err());
    }

    #[test]
    fn drain_len() {
        let mut bank = BankVec::<i32, 3>::from([1, 2, 3]);
        let mut drain = bank.drain(..);

        assert_eq!(drain.len(), 3);
        let _ = drain.next();
        assert_eq!(drain.len(), 2);
    }

    #[test]
    fn drain_drop() {
        let mut bank = BankVec::<i32, 3>::from([1, 2, 3, 4]);

        // Tail is greater than zero
        // Nothing more needs to be done here
        let _ = bank.drain(..2);

        // Same as above...
        let mut bank = BankArr::<i32, 4>::from([1, 2, 3, 4]);
        let _ = bank.drain(..2);
    }
}