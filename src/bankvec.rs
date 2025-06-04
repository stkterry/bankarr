use std::{array, hint::unreachable_unchecked, mem::{ManuallyDrop, MaybeUninit}, ops::{Deref, DerefMut}, ptr};

use crate::BankArr;


#[derive(Debug, Clone)]
pub enum BankVec<T, const C: usize> {
    Vec(Vec<T>),
    Bank(BankArr<T, C>)
}

impl<T: PartialEq, const C: usize> PartialEq for BankVec<T, C> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Vec(l0), Self::Vec(r0)) => l0 == r0,
            (Self::Bank(l0), Self::Bank(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl<T, const C: usize, const N: usize> From<[T; N]> for BankVec<T, C> 
{
    fn from(arr: [T; N]) -> Self {
        if N <= C {
            let mut arr = ManuallyDrop::new(arr);
            let mut data = [const { MaybeUninit::<T>::uninit() }; C];
            unsafe { ptr::copy_nonoverlapping(
                arr.as_ptr().cast(), 
                data.as_mut_ptr(), 
                N
            );}
            unsafe { ManuallyDrop::drop(&mut arr) };
            Self::Bank(BankArr { data, len: N })
        } else {
            let vec = arr.into_iter().collect::<Vec<T>>();
            Self::Vec(vec)
        }
    }
    
}

impl<T, const C: usize> From<Vec<T>> for BankVec<T, C> {
    fn from(vec: Vec<T>) -> Self {
        if vec.len() <= C {
            let mut bank = BankArr {
                data: [const { MaybeUninit::<T>::uninit() }; C],
                len: vec.len()
            };
            bank.data
                .iter_mut()
                .zip(vec.into_iter())
                .for_each(|(b, v)| { b.write(v); });

            Self::Bank(bank)

        } else { Self::Vec(vec) }
    }
}




impl <T, const C: usize> Deref for BankVec<T, C> {
    type Target = [T];
    fn deref(&self) -> &Self::Target { self.as_slice() }
}

impl <T, const C: usize> DerefMut for BankVec<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target { self.as_mut_slice() }
}



impl<T, const C: usize> BankVec<T, C> {

    #[inline]
    pub fn new() -> Self {
        Self::Bank(BankArr::new())
    }

    #[inline]
    pub const fn is_arr(&self) -> bool {
        match self {
            BankVec::Vec(_) => false,
            BankVec::Bank(_) => true,
        }
    }

    #[inline]
    pub const fn is_vec(&self) -> bool {
        match self {
            BankVec::Vec(_) => true,
            BankVec::Bank { .. } => false,
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::Vec(items) => items,
            Self::Bank(bank) => bank.as_slice(),
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            Self::Vec(items) => items,
            Self::Bank(bank) => bank.as_mut_slice(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            BankVec::Vec(vec) => vec.len(),
            BankVec::Bank(bank) => bank.len(),
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        match self {
            BankVec::Vec(vec) => vec.capacity(),
            BankVec::Bank(_) => C,
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        match self {
            Self::Vec(items) => items.push(value),
            Self::Bank(bank) => unsafe {
                if bank.len < C { bank.push_unchecked(value); } 
                else { self.into_vec_unchecked().push(value); }
            },
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        match self {
            Self::Vec(vec) => {
                let popped = vec.pop();
                if vec.len() == C { unsafe { self.into_bank_unchecked(); } }
                popped
            },
            Self::Bank(bank) => bank.pop(),
        }
    }

    
    pub fn insert(&mut self, index: usize, element: T) {
        match self {
            Self::Vec(vec) => vec.insert(index, element),
            Self::Bank(bank) => match bank.len == C {
                true => unsafe { self.into_vec_unchecked().insert(index, element); },
                false => { bank.insert(index, element); }
            }
        }
    }
    
    pub fn remove(&mut self, index: usize) -> T {
        match self {
            BankVec::Vec(vec) => {
                let removed = vec.remove(index);
                if vec.len() == C { unsafe { self.into_bank_unchecked(); } }
                removed
            },
            BankVec::Bank(bank) => bank.remove(index),
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        match self {
            BankVec::Vec(vec) => {
                let removed = vec.swap_remove(index);
                if vec.len() == C { unsafe { self.into_bank_unchecked(); }}
                removed
            },
            BankVec::Bank(bank) => bank.swap_remove(index),
        }
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        match self {
            Self::Vec(items) => items.reserve_exact(additional),
            _ => { }
        }
    }

    #[inline(always)]
    unsafe fn as_bank_unchecked_mut(&mut self) -> &mut BankArr<T, C> {
        match self {
            BankVec::Bank(bank) => bank,
            _ => unsafe { unreachable_unchecked() }
        }
    }

    #[inline(always)]
    unsafe fn as_vec_unchecked_mut(&mut self) -> &mut Vec<T> {
        match self {
            BankVec::Vec(vec) => vec,
            _ => unsafe { unreachable_unchecked() }
        }
    }

    #[inline]
    unsafe fn into_bank_unchecked(&mut self) -> &mut BankArr<T, C> {

        let vec = match self {
            Self::Vec(vec) => vec,
            _ => unsafe { unreachable_unchecked() }
        };
        debug_assert!(vec.len() == C);

        let bank = BankArr {
            data: unsafe {
                let mut drain = vec.drain(..);
                array::from_fn(|_| MaybeUninit::new(drain.next().unwrap_unchecked()))
            },
            len: C,
        };
        
        *self = Self::Bank(bank);
        
        unsafe { self.as_bank_unchecked_mut() }
    }

    #[inline]
    unsafe fn into_vec_unchecked(&mut self) -> &mut Vec<T> {
        let bank = match self {
            Self::Bank(bank) => bank,
            _ => unsafe { unreachable_unchecked() }
        };
        let vec: Vec<T> = bank.data
            .iter()
            .map(|v| unsafe { v.assume_init_read() } )
            .collect();

        *self = Self::Vec(vec);

        unsafe { self.as_vec_unchecked_mut() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type B = BankVec<u32, 3>;

    #[test]
    fn is_variant() {
        let bankarr = B::from([]);
        let bankvec = B::from([1, 2, 3, 4]);

        assert!(bankarr.is_arr());
        assert!(!bankarr.is_vec());

        assert!(bankvec.is_vec());
        assert!(!bankvec.is_arr());
    }

    #[test]
    fn push() {
        let mut bank = B::new();
        bank.push(1);
        bank.push(2);
        bank.push(3);
        assert!(bank.is_arr());
        
        assert_eq!(bank[..1], [1]);
        assert_eq!(bank[..], [1, 2, 3]);
        
        bank.push(4);
        assert!(bank.is_vec());
        bank.push(5);
        assert_eq!(bank[..], [1, 2, 3, 4, 5]);
    }


    #[test]
    fn pop() {
        let mut bank = B::from([3, 4, 5, 6]);
        
        assert!(bank.is_vec());
        assert_eq!(bank.pop(), Some(6));

        assert!(bank.is_arr());
        assert_eq!(bank.pop(), Some(5))
    }

    #[test]
    fn remove() {
        let mut bank = B::from([3, 4, 5, 6]);

        assert!(bank.is_vec());
        let removed = bank.remove(1);
        assert_eq!(removed, 4);
        assert_eq!(&bank[..], &[3, 5, 6]);

        assert!(bank.is_arr());
        let removed = bank.remove(1);
        assert_eq!(removed, 5);
        assert_eq!(&bank[..], &[3, 6]);
    }

    #[test]
    fn swap_remove() {
        let mut bank = BankVec::<String, 3>::from(["aa".to_string(), "bb".to_string(), "cc".to_string(), "dd".to_string()]);
        
        assert!(bank.is_vec());
        let removed = bank.swap_remove(0);
        assert_eq!(removed, "aa".to_string());

        assert!(bank.is_arr());
        let removed = bank.swap_remove(1);
        assert_eq!(removed, "bb".to_string());

        assert_eq!(bank[..], ["dd".to_string(), "cc".to_string()])
    }

    #[test]
    fn insert() {
        let mut bank = B::from([3, 5]);

        bank.insert(2, 6);
        assert!(bank.is_arr());
        bank.insert(1, 4);
        
        assert!(bank.is_vec());
        bank.insert(4, 7);

        assert_eq!(bank[..], [3, 4, 5, 6, 7]);
    }

    #[test]
    #[should_panic]
    fn insert_out_of_bounds() {
        let mut bank = B::from([3, 4, 5]);

        bank.insert(4, 0);
    }

    #[test]
    fn reserve_exact() {
        let mut bank = B::from([3, 4, 5]);
        bank.reserve_exact(1);
        assert_eq!(bank.capacity(), 3);
        bank.push(4);
        bank.reserve_exact(1);
        assert_eq!(bank.capacity(), 6);
    }
    

    #[test]
    fn iter_mut() {
        let mut bank = BankVec::<&'static str, 3>::from(["a", "b", "c"]);
        assert!(bank.is_arr());
        let mut iter = bank.iter_mut();
        for s in ["a", "b", "c"] {
            assert_eq!(iter.next(), Some(s).as_mut());
        }
        assert_eq!(iter.next(), None);


        bank.push("d");
        assert!(bank.is_vec());
        let mut iter = bank.iter_mut();
        for s in ["a", "b", "c", "d"] {
            assert_eq!(iter.next(), Some(s).as_mut());
        }
        assert_eq!(iter.next(), None);
    }


    #[test]
    fn iter() {
        let mut bank = BankVec::<&'static str, 3>::from(["a", "b", "c"]);
        assert!(bank.is_arr());
        let mut iter = bank.iter();
        for s in ["a", "b", "c"] {
            assert_eq!(iter.next(), Some(s).as_ref());
        }
        assert_eq!(iter.next(), None);


        bank.push("d");
        assert!(bank.is_vec());
        let mut iter = bank.iter();
        for s in ["a", "b", "c", "d"] {
            assert_eq!(iter.next(), Some(s).as_ref());
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn as_slice() {
        let mut bank = B::from([3, 4, 5]);
        assert!(bank.is_arr());
        assert_eq!(&bank[..], bank.as_slice());

        bank.push(6);
        assert!(bank.is_vec());
        assert_eq!(&bank[..], bank.as_slice());
    }

    #[test]
    fn as_slice_mut() {
        let mut bank = B::from([3, 4, 5]);
        assert!(bank.is_arr());
        assert_eq!(bank.as_slice(), [3, 4, 5]);

        bank.push(6);
        assert!(bank.is_vec());
        assert_eq!(bank.as_mut_slice(), [3, 4, 5, 6]);
    }

    #[test]
    fn from_vec() {
        let bankarr = B::from(vec![3, 4, 5]);
        assert_eq!(bankarr.len(), 3);
        assert_eq!(*bankarr.as_slice(), [3, 4, 5]);
        
        let bankvec = B::from(vec![3, 4, 5, 6]);
        assert_eq!(bankvec.len(), 4);
        assert_eq!(*bankvec.as_slice(), [3, 4, 5, 6]);
    }

    #[test]
    fn clone() {
        let bankarr = B::new();
        let bankvec = B::from([3, 4, 5, 6]);

        assert!(bankarr == bankarr.clone());
        assert!(bankvec == bankvec.clone());
        assert!(bankvec != bankarr);
    }

}