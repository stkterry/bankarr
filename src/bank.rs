use std::{marker::PhantomData, mem::{self, MaybeUninit}, ops::{Deref, DerefMut}, ptr::{self, NonNull}, slice};

pub struct Bank<T, const C: usize> {
    data: [MaybeUninit<T>; C],
    len: usize,
    // max_drop: usize,
}

impl <T, const C: usize> Drop for Bank<T, C> {
    fn drop(&mut self) {
        unsafe {
            self.data
                .get_unchecked_mut(0..self.len)
                .into_iter()
                .for_each(|v| v.assume_init_drop());

        }
    }
}

impl <T, const C: usize> Deref for Bank<T, C> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        // We are tracking initialized values via len, ensuring the slice is not UB
        unsafe { mem::transmute::<_, &[T]>(self.data.get_unchecked(0..self.len)) }
    }
}

impl <T, const C: usize> DerefMut for Bank<T, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // We are tracking initialized values via len, ensuring the slice is not UB
        unsafe { mem::transmute::<_, &mut [T]>(self.data.get_unchecked_mut(0..self.len)) }
    }
}

impl<T, const C: usize> IntoIterator for Bank<T, C> {
    type Item = T;
    type IntoIter = BankIter<T, C>;
    
    fn into_iter(self) -> Self::IntoIter {
        let start: *const T = self.data.as_ptr().cast();
        let end: *const T = unsafe { start.add(self.len) };
        let iter = RawBankIter { start, end };
        Self::IntoIter { _bank: self, iter }
    }
} 

impl <T, const C: usize, const N: usize> From<[T; N]> for Bank<T, C> {
    fn from(value: [T; N]) -> Self {
        assert!(N <= C);
        let mut data: [MaybeUninit<T>; C] = [const { MaybeUninit::uninit() }; C];
    
        value.into_iter()
            .zip(data.iter_mut())
            .for_each(|(src, dst)| { dst.write(src); } );

        Self { data, len: N, }
    }
}

impl <T, const C: usize> From<Vec<T>> for Bank<T, C> {
    fn from(value: Vec<T>) -> Self {
        let count = value.len();
        assert!(count <= C);
        let mut data: [MaybeUninit<T>; C] = [const { MaybeUninit::uninit() }; C];

        value.into_iter()
            .zip(data.iter_mut())
            .for_each(|(src, dst)| { dst.write(src); });

        Self { data, len: count, }
    }
}

impl <T, const C: usize> Bank<T, C> {

    pub fn new() -> Self {
        Self {
            data: [const { MaybeUninit::uninit() }; C],
            len: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) -> bool {
        if self.len == C { return false }
        unsafe { self.data.get_unchecked_mut(self.len).write(value); }
        self.len += 1;
        true
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        match self.len == 0 {
            true => None,
            false => unsafe {
                self.len -= 1;
                Some(self.data.get_unchecked_mut(self.len).assume_init_read())
            }
        }
    }

    pub fn insert(&mut self, index: usize, value: T) -> bool {
        assert!(index <= self.len, "Index out of bounds");
        if self.len == C { return false }

        unsafe {
            let ptr = self.data.as_mut_ptr().add(index);
            ptr::copy(ptr, ptr.add(1), self.len - index);
            ptr::write(ptr, MaybeUninit::new(value));
        }
        self.len += 1;
        true
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "Index out of bounds");
        self.len -= 1;
        unsafe {
            let removed = self.data.get_unchecked(index).assume_init_read();
            let ptr = self.data.as_mut_ptr().add(index);
            ptr::copy(ptr.add(1), ptr, self.len - index);
            removed
        }
    }

    pub fn drain(&mut self) -> BankDrain<T> {
        let iter = unsafe { 
            RawBankIter::new(self.data.get_unchecked(0..self.len)) 
        };
        self.len = 0;

        BankDrain { _slice: PhantomData, iter }
    }

    pub const fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data.as_ptr().cast(), self.len) }
    }

    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data.as_mut_ptr().cast(), self.len) }
    }
}


pub struct RawBankIter<T> {
    start: *const T,
    end: *const T,
}

impl <T> RawBankIter<T> {
    unsafe fn new(slice: &[MaybeUninit<T>]) -> Self {
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
    fn next(&mut self) -> Option<T> {
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
    fn next_back(&mut self) -> Option<T> {
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
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize) 
            / mem::size_of::<T>().max(1);
        (len, Some(len))
    }

}

pub struct BankIter<T, const C: usize> {
    _bank: Bank<T, C>,
    iter: RawBankIter<T>
}

impl <T, const C: usize> Iterator for BankIter<T, C> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl <T, const C: usize> DoubleEndedIterator for BankIter<T, C> {
    fn next_back(&mut self) -> Option<Self::Item> { self.iter.next_back() }
}

impl <T, const C: usize> Drop for BankIter<T, C> {
    fn drop(&mut self) { for _ in &mut *self {} }
}

pub struct BankDrain<'a, T: 'a> {
    _slice: PhantomData<&'a mut [T]>,
    iter: RawBankIter<T>
}

impl <'a, T> Iterator for BankDrain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl <'a, T> DoubleEndedIterator for BankDrain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> { self.iter.next_back() }
}

impl <'a, T> Drop for BankDrain<'a, T> {
    fn drop(&mut self) { for _ in &mut *self { } }
}







#[cfg(test)]
mod tests {
    use super::*;

    type B = Bank<u32, 4>;

    #[test]
    fn push() {
        let mut bank = B::new();
        let p1 = bank.push(3);
        let p2 = bank.push(4);

        assert!(p1 && p2);
        assert_eq!(bank[0], 3);
        assert_eq!(bank[1], 4);
        assert_eq!(bank.len(), 2);
    }

    #[test]
    fn push_to_full() {
        let mut bank = B::new();
        for i in 0..4 { bank.push(i); }
        let false_because_full = bank.push(4);

        assert_eq!(false_because_full, false);
    }

    #[test]
    fn pop() {
        let mut bank = B::from([3, 4]);
        let removed = bank.pop();

        assert_eq!(removed, Some(4));
        assert_eq!(bank.len(), 1);
    }

    #[test]
    fn remove() {
        let mut bank = B::from([3, 4, 5]);
        let removed = bank.remove(1);
        
        assert_eq!(removed, 4);
        assert_eq!(&bank[..], &[3, 5]);
    }

    #[test]
    #[should_panic]
    fn remove_out_of_bounds() {
        let mut bank = B::from([3, 4, 5]);
        bank.remove(3);
    }

    
    #[test]
    fn insert() {
        let mut bank = B::from([3, 5, 6]);
        let did_insert = bank.insert(1, 4);
        let didnt_insert = bank.insert(2, 0);

        assert_eq!(did_insert, true);
        assert_eq!(didnt_insert, false);
        assert_eq!(&bank[..], &[3, 4, 5, 6]);
    }

    #[test]
    #[should_panic]
    fn insert_out_of_bounds() {
        let mut bank = B::from([3, 4]);

        bank.insert(3, 0);
    }

    #[test]
    fn drain() {
        let mut bank = B::from([3, 4, 5]);
        let drained = bank.drain()
            .into_iter().collect::<Vec<u32>>();

        assert_eq!(bank.len(), 0);
        assert_eq!(drained, vec![3, 4, 5]);
    }

    #[test]
    fn iter() {
        let bank = B::from([3, 4, 5]);
        let collected = bank.iter()
            .map(|v| *v)
            .collect::<Vec<u32>>();

        assert_eq!(&bank[..], &collected); 
    }

    #[test]
    fn iter_mut() {
        let mut bank = B::from([3, 4, 5]);
        let collected = bank.iter_mut()
            .map(|v| *v)
            .collect::<Vec<u32>>();

        assert_eq!(&bank[..], &collected); 
    }

    #[test]
    fn into_iter() {
        let bank = B::from([3, 4, 5]);
        let collected = bank.into_iter()
            .collect::<Vec<u32>>();

        assert_eq!(&collected, &[3, 4, 5]); 
    }

    #[test]
    fn as_slice() {
        let bank = B::from([3, 4, 5]);
        assert_eq!(&bank[..], bank.as_slice())
    }

    #[test]
    fn as_slice_mut() {
        let mut bank = B::from([3, 4, 5]);
        let mut bank2 = B::from([3, 4, 5]);

        assert_eq!(bank.as_mut_slice(), bank2.as_mut_slice());
    }

    #[test]
    fn dropping_types() {
        let mut bank: Bank<_, 4> = Bank::from(["aa".to_string(), "bb".to_string()]);

        let popped = bank.pop();
        let add_new = bank.push("ff".to_string());
        let removed = bank.remove(0);
        let inserted = bank.insert(0, "dd".to_string());

        assert_eq!(popped, Some("bb".to_string()));
        assert_eq!(add_new, true);
        assert_eq!(removed, "aa".to_string());
        assert_eq!(inserted, true);
        assert_eq!(&bank[..], &["dd".to_string(), "ff".to_string()])
    }

}