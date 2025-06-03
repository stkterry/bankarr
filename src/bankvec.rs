use std::{hint::unreachable_unchecked, mem::{ManuallyDrop, MaybeUninit}, ops::{Deref, DerefMut}, ptr, slice};

use crate::BankArr;



pub enum BankVec<T, const C: usize> {
    Vec(Vec<T>),
    Bank(BankArr<T, C>)
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

            Self::Bank(BankArr { data, len: C })

        } else {
            let vec = arr.into_iter().collect::<Vec<T>>();
            Self::Vec(vec)
        }
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

    pub fn new() -> Self {
        Self::Bank(BankArr::new())
    }

    pub const fn is_arr(&self) -> bool {
        match self {
            BankVec::Vec(_) => false,
            BankVec::Bank(_) => true,
        }
    }

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
    pub fn push(&mut self, value: T) {
        match self {
            Self::Vec(items) => items.push(value),
            Self::Bank(bank) => unsafe {
                if bank.len < C { bank.push_unchecked(value); } 
                else {
                    let mut vec: Vec<T> = bank.data
                        .iter()
                        .map(|v| v.assume_init_read() )
                        .collect();
                    vec.push(value);

                    *self = Self::Vec(vec);
                }
            },
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        match self {
            Self::Vec(vec) => {
                let popped = vec.pop();
                if vec.len() == C { unsafe { self.into_bank(); } }
                popped
            },
            Self::Bank(bank) => bank.pop(),
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
    unsafe fn into_bank(&mut self) {
        let vec = match self {
            Self::Vec(vec) => vec,
            _ => unsafe { unreachable_unchecked() }
        };
        debug_assert!(vec.len() == C);

        let mut bank = BankArr {
            data: [const { MaybeUninit::<T>::uninit() }; C],
            len: C,
        };

        unsafe { ptr::copy_nonoverlapping(
            vec.as_ptr().cast(), 
            bank.data.as_mut_ptr(), 
            C
        )}

        *self = Self::Bank(bank);
    }

    #[inline(always)]
    unsafe fn into_vec(&mut self) {
        let bank = match self {
            Self::Bank(bank) => bank,
            _ => unsafe { unreachable_unchecked() }
        };
        let vec: Vec<T> = bank.data
            .iter()
            .map(|v| unsafe { v.assume_init_read() } )
            .collect();

        *self = Self::Vec(vec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push() {
        let mut bank = BankVec::<i32, 3>::new();
        bank.push(1);
        bank.push(2);
        bank.push(3);
        
        assert_eq!(bank[..1], [1]);
        assert_eq!(bank[..], [1, 2, 3]);
        
        bank.push(4);

        assert_eq!(bank[..], [1, 2, 3, 4]);

    }


    #[test]
    fn overflow() {
        let mut bank = BankVec::<i32, 3>::from([1, 2, 3]);
        assert!(bank.is_arr());

        bank.push(4);
        assert!(bank.is_vec()); 
    }
}