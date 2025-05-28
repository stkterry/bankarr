use std::{mem::{self, MaybeUninit}, ptr::{self, NonNull}};



pub(super) struct RawIter<T> {
    start: *const T,
    end: *const T,
}

impl <T> RawIter<T> {
    pub(super) unsafe fn new(slice: &[MaybeUninit<T>]) -> Self {
        let start: *const T = slice.as_ptr().cast();
        Self {
            start,
            end: match (mem::size_of::<T>() == 0, slice.len()) {
                (true, count) => (slice.as_ptr() as usize + count) as *const _,
                (_, 0) => start,
                (_, count) => unsafe { start.add(count) }
            } 
        }
    }

    #[inline]
    pub(super) fn next(&mut self) -> Option<T> {
        match (self.start == self.end, mem::size_of::<T>() == 0) {
            (true, _) => None,
            (_, true) => unsafe {
                self.start = (self.start as usize + 1) as *const _;
                Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
            },
            (_, false) => unsafe {
                let item = Some(ptr::read(self.start));
                self.start = self.start.offset(1);
                item                
            }
        }
    }

    #[inline]
    pub(super) fn next_back(&mut self) -> Option<T> {
        match (self.start == self.end, mem::size_of::<T>() == 0) {
            (true, _) => None,
            (_, true) => unsafe {
                self.end = (self.end as usize - 1) as *const _;
                Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
            },
            (_, false) => unsafe {
                self.end = self.end.offset(-1);
                Some(ptr::read(self.end))
            }
        }
    }

    #[inline]
    pub(super) fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize) 
            / mem::size_of::<T>().max(1);
        (len, Some(len))
    }

}