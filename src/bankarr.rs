
mod raw_iter;
mod drain;

use std::{mem::{ManuallyDrop, MaybeUninit}, ops::{Deref, DerefMut}, ptr, slice};
use crate::errors::BankFullError;
use raw_iter::RawIter;
use drain::Drain;

#[derive(Debug)]
pub struct BankArr<T, const C: usize> {
    pub(crate) data: [MaybeUninit<T>; C],
    pub(crate) len: usize,
}

impl<T: PartialEq, const C: usize> PartialEq for BankArr<T, C> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice() && self.len == other.len
    }
}

impl<T: Clone, const C: usize> Clone for BankArr<T, C> {
    fn clone(&self) -> Self {

        let mut data = [const { MaybeUninit::<T>::uninit() }; C];

        data.iter_mut()
            .zip(self.iter())
            .for_each(|(b, a)| { b.write(a.clone()); });
        
        Self { data, len: self.len }
    }
}

impl <T, const C: usize> Drop for BankArr<T, C> {
    fn drop(&mut self) {
        unsafe {
            self.data
                .get_unchecked_mut(0..self.len)
                .into_iter()
                .for_each(|v| v.assume_init_drop());
        }
    }
}

impl <T, const C: usize> Deref for BankArr<T, C> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target { self.as_slice() }
}

impl <T, const C: usize> DerefMut for BankArr<T, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { self.as_mut_slice() }
}


impl <T, const C: usize, const N: usize> From<[T; N]> for BankArr<T, C> {
    fn from(arr: [T; N]) -> Self {
        assert!(N <= C);
        
        let arr = ManuallyDrop::new(arr);
        let mut bank = Self {
            data: [const { MaybeUninit::uninit() }; C],
            len: N
        };
        
        unsafe { ptr::copy_nonoverlapping(
            arr.as_ptr().cast(), 
            bank.data.as_mut_ptr(), 
            N
        )}
        bank
    }
}

impl <T, const C: usize> From<Vec<T>> for BankArr<T, C> {
    fn from(vec: Vec<T>) -> Self {
        let len = vec.len();
        assert!(len <= C);

        let mut bank = Self {
            data: [const { MaybeUninit::uninit() }; C],
            len,
        };

        unsafe { ptr::copy_nonoverlapping(
            vec.as_ptr().cast(), 
            bank.data.as_mut_ptr(), 
            len
        );}

        bank.data
            .iter_mut()
            .zip(vec.into_iter())
            .for_each(|(b, v)| { b.write(v); });
        bank
    }
}

impl <T, const C: usize> BankArr<T, C> {

    pub const fn new() -> Self {
        Self {
            data: [const { MaybeUninit::uninit() }; C],
            len: 0,
        }
    }

    #[inline(always)]
    pub const fn len(&self) -> usize { self.len }

    
    #[inline]
    pub fn push(&mut self, value: T) {
        assert!(self.len < C);
        unsafe { self.push_unchecked(value) }
    }

    #[inline]
    pub fn try_push(&mut self, value: T) -> Result<(), BankFullError> {
        if self.len == C { return Err(BankFullError {}) }
        unsafe { self.push_unchecked(value) }
        Ok(())
    }

    #[inline(always)]
    pub unsafe fn push_unchecked(&mut self, value: T) {
        debug_assert!(self.len < C);
        unsafe { self.data.get_unchecked_mut(self.len).write(value); }
        self.len += 1;
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

    pub fn insert(&mut self, index: usize, element: T) -> bool {
        assert!(index <= self.len, "Index out of bounds");
        if self.len == C { return false }

        unsafe {
            let ptr = self.data.as_mut_ptr().add(index);
            ptr::copy(ptr, ptr.add(1), self.len - index);
            ptr::write(ptr, MaybeUninit::new(element));
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

    pub fn swap_remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "Index out of bounds");
        self.len -= 1;
        unsafe {
            self.data.swap(index, self.len);
            self.data.get_unchecked(self.len).assume_init_read()
        }

    }

    pub fn drain(&mut self) -> Drain<T> {
        let iter = unsafe { 
            RawIter::new(self.data.as_ptr().cast(), self.len) 
        };
        self.len = 0;

        Drain::new(iter)
    }

    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        // We are tracking initialized values via len, ensuring the slice is not UB
        unsafe { slice::from_raw_parts(self.data.as_ptr().cast(), self.len) }
    }

    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        // We are tracking initialized values via len, ensuring the slice is not UB
        unsafe { slice::from_raw_parts_mut(self.data.as_mut_ptr().cast(), self.len) }
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    type B = BankArr<u32, 4>;

    #[test]
    fn push() {
        let mut bank = B::new();
        bank.push(3);
        bank.push(4);

        assert_eq!(bank[0], 3);
        assert_eq!(bank[1], 4);
        assert_eq!(bank.len(), 2);
    }

    #[test]
    #[should_panic]
    fn push_to_full() {
        let mut bank = B::new();
        for i in 0..4 { bank.push(i); }
        bank.push(4);
    }

    #[test]
    fn try_push() {
        let mut bank = B::from([3, 4, 5]);
        assert!(bank.try_push(6).is_ok());
        assert!(bank.try_push(7).is_err());
    }

    #[test]
    fn pop() {
        let mut bank = B::from([3, 4]);
        let removed = bank.pop();

        assert_eq!(removed, Some(4));
        assert_eq!(bank.len(), 1);

        let mut bank = B::new();
        assert_eq!(bank.pop(), None);
    }

    #[test]
    fn remove() {
        let mut bank = B::from([3, 4, 5]);
        let removed = bank.remove(1);
        
        assert_eq!(removed, 4);
        assert_eq!(&bank[..], &[3, 5]);
    }

    #[test]
    fn swap_remove() {
        let mut bank: BankArr<String, 3> = BankArr::from(["aa".to_string(), "bb".to_string(), "cc".to_string()]);
        let removed = bank.swap_remove(0);

        assert_eq!(removed, "aa".to_string());
        assert_eq!(&bank[..], &["cc".to_string(), "bb".to_string()]);
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

        let mut bank = B::from([3, 4]);
        let mut drain = bank.drain();
        assert_eq!(drain.next_back(), Some(4));
        assert_eq!(drain.next(), Some(3));
        assert_eq!(drain.next(), None);
    }

    #[test]
    fn drain_zst() {
        let mut bank = BankArr::<(), 2>::from([(), ()]);
        let mut drain = bank.drain();
        assert_eq!(drain.next(), Some(()));
        assert_eq!(drain.next_back(), Some(()));
        assert_eq!(drain.next(), None);
        assert_eq!(drain.next_back(), None);
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
    fn as_slice() {
        let bank = B::from([3, 4, 5]);
        assert_eq!(bank.as_slice(), [3, 4, 5])
    }

    #[test]
    fn as_slice_mut() {
        let mut bank = B::from([3, 4, 5]);
        assert_eq!(bank.as_mut_slice(), [3, 4, 5]);
    }

    #[test]
    fn dropping_types() {
        let mut bank: BankArr<_, 4> = BankArr::from(vec!["aa".to_string(), "bb".to_string()]);

        let popped = bank.pop();
        bank.push("ff".to_string());
        let removed = bank.remove(0);
        let inserted = bank.insert(0, "dd".to_string());

        assert_eq!(popped, Some("bb".to_string()));
        assert_eq!(removed, "aa".to_string());
        assert_eq!(inserted, true);
        assert_eq!(&bank[..], &["dd".to_string(), "ff".to_string()])
    }

    #[test]
    fn clone() {
        let bank = BankArr::<_, 2>::from(["aa".to_string(), "bb".to_string()]);
        assert_eq!(bank, bank.clone());
    }
}