use std::{mem, ptr::NonNull};


pub(super) struct RawIter<T> {
    start: *const T,
    end: *const T,

}

impl <T> RawIter<T> {

    const IS_ZST: bool = mem::size_of::<T>() == 0;

    pub(super) unsafe fn new(start: *const T, len: usize) -> Self {

        Self {
            start,
            end: match (Self::IS_ZST, len) {
                (true, count) => (start as usize + count) as *const _,
                (_, 0) => start,
                (_, count) => unsafe { start.add(count) }
            } 
        }
    }

    #[inline]
    pub(super) fn next(&mut self) -> Option<T> {
        match (self.start == self.end, Self::IS_ZST) {
            (true, _) => None,
            (_, true) => unsafe {
                self.end = (self.end as usize - 1) as *const _;
                Some(NonNull::<T>::dangling().read())
            },
            (_, false) => unsafe {
                let item = Some(self.start.read());
                self.start = self.start.offset(1);
                item                
            }
        }
    }

    #[inline]
    pub(super) fn next_back(&mut self) -> Option<T> {
        match (self.start == self.end, Self::IS_ZST) {
            (true, _) => None,
            (_, true) => unsafe {
                self.end = (self.end as usize - 1) as *const _;
                Some(NonNull::<T>::dangling().read())
            },
            (_, false) => unsafe {
                self.end = self.end.offset(-1);
                Some(self.end.read())
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

impl<T> Drop for RawIter<T> {
    fn drop(&mut self) {
        while let Some(_v) = self.next() {}
    }
}