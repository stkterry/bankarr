use std::{array, hint::unreachable_unchecked, mem::{ManuallyDrop, MaybeUninit}, ops::{Deref, DerefMut, Index, IndexMut}, ptr, slice::SliceIndex};

use crate::BankArr;



#[derive(Debug, Clone)]
pub enum BankVec<T, const C: usize> {
    Dyn(Vec<T>),
    Inline(BankArr<T, C>)
}

impl<T: PartialEq, const C: usize> PartialEq for BankVec<T, C> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Dyn(l0), Self::Dyn(r0)) => l0 == r0,
            (Self::Inline(l0), Self::Inline(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl<T: PartialEq, const C: usize, const N: usize> PartialEq<[T; N]> for BankVec<T, C> {
    fn eq(&self, other: &[T; N]) -> bool {
        self.len() == other.len() && self.as_slice() == other.as_slice()
    }
}

impl<T: PartialEq, const C: usize, const N: usize> PartialEq<&[T; N]> for BankVec<T, C> {
    fn eq(&self, other: &&[T; N]) -> bool {
        self.len() == other.len() && self.as_slice() == *other
    }
}

impl<T: PartialEq, const C: usize> PartialEq<Vec<T>> for BankVec<T, C> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.len() == other.len() && self.as_slice() == other
    }
}

impl<T: PartialEq, const C: usize> PartialEq<[T]> for BankVec<T, C> {
    fn eq(&self, other: &[T]) -> bool {
        self.len() == other.len() && self.as_slice() == other
    }
}

impl<T: PartialEq, const C: usize> PartialEq<&[T]> for BankVec<T, C> {
    fn eq(&self, other: &&[T]) -> bool {
        self.len() == other.len() && self.as_slice() == *other
    }
}

impl<T, const C: usize, const N: usize> From<[T; N]> for BankVec<T, C> 
{
    /// Create a new instance from an array.
    /// 
    /// The array consumed may be smaller or larger than the specified bank size `C`.
    /// If the consumed array is larger, than the variant becomes [`Inline`](BankVec::Inline)
    /// otherwise it's a [`Dyn`](BankVec::Dyn)
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let bank1 = BankVec::<i32, 3>::from([1, 2]);
    /// assert!(bank1.is_inline());
    /// 
    /// let bank2 = BankVec::<i32, 3>::from([1, 2, 3, 4]);
    /// assert!(bank2.is_dyn());
    /// ```
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
            Self::Inline(BankArr { data, len: N })
        } else {
            let vec = arr.into_iter().collect::<Vec<T>>();
            Self::Dyn(vec)
        }
    }
    
}

impl<T, const C: usize> From<Vec<T>> for BankVec<T, C> {

    /// Create a new instance from a vec.
    /// 
    /// The vec consumed may be smaller or larger than the specified bank size `C`.
    /// If the consumed vec is larger, than the variant is [`Dyn`](BankVec::Dyn),
    /// otherwise [`Inline`](BankVec::Inline).
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let bank1 = BankVec::<i32, 3>::from(vec![1, 2]);
    /// assert!(bank1.is_inline());
    /// 
    /// let bank2 = BankVec::<i32, 3>::from(vec![1, 2, 3, 4]);
    /// assert!(bank2.is_dyn());
    /// ```
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

            Self::Inline(bank)

        } else { Self::Dyn(vec) }
    }
}

impl <T, const C: usize> Deref for BankVec<T, C> {
    type Target = [T];
    fn deref(&self) -> &Self::Target { self.as_slice() }
}

impl <T, const C: usize> DerefMut for BankVec<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target { self.as_mut_slice() }
}


impl<T, const C: usize, I: SliceIndex<[T]>> Index<I> for BankVec<T, C> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, const C: usize, I: SliceIndex<[T]>> IndexMut<I> for BankVec<T, C> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}


impl<T, const C: usize> BankVec<T, C> {

    /// Constructs a new, empty `BankVec<T, C>`.
    /// 
    /// This *will* allocate space for the entire bank. 
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<i32, 3>::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::Inline(BankArr::new())
    }


    /// Returns true if the variant is [`Inline`](BankVec::Inline), otherwise false.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankVec;
    /// let bank = BankVec::<i32, 3>::from([0; 2]);
    /// assert!(bank.is_inline());
    /// ```
    #[inline]
    pub const fn is_inline(&self) -> bool {
        match self {
            BankVec::Dyn(_) => false,
            BankVec::Inline(_) => true,
        }
    }

    /// Returns true if the underlying structure is a [`Dyn`](BankVec::Dyn), otherwise false.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankVec;
    /// let bank = BankVec::<i32, 3>::from([0; 4]);
    /// assert!(bank.is_dyn());
    /// ```
    #[inline]
    pub const fn is_dyn(&self) -> bool {
        match self {
            BankVec::Dyn(_) => true,
            BankVec::Inline { .. } => false,
        }
    }

    /// Extracts a slice containing the entire bank.
    /// 
    /// Equivalent to `&bank[..]`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use std::io::{self, Write};
    /// use bankarr::BankVec;
    /// 
    /// let bank = BankVec::<u8, 3>::from([1, 2, 3, 4]);
    /// io::sink().write(bank.as_slice()).unwrap();
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::Dyn(items) => items,
            Self::Inline(bank) => bank.as_slice(),
        }
    }

    /// Extracts a mutable slice containing the entire bank.
    /// 
    /// Equivalent to `&mut bank[..]`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use std::io::{self, Read};
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<u8, 3>::from([0; 4]);
    /// io::repeat(0b101).read_exact(bank.as_mut_slice()).unwrap();
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            Self::Dyn(items) => items,
            Self::Inline(bank) => bank.as_mut_slice(),
        }
    }


    /// Returns the length of the bank.
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<i32, 3>::new();
    /// assert_eq!(bank.len(), 0);
    /// 
    /// bank.push(5);
    /// assert_eq!(bank.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            BankVec::Dyn(vec) => vec.len(),
            BankVec::Inline(bank) => bank.len(),
        }
    }

    /// Returns the number of elements the bank can hold without reallocating.
    /// 
    /// If the variant is [`Inline`](BankVec::Inline), this will be fixed as `C`.
    #[inline]
    pub fn capacity(&self) -> usize {
        match self {
            BankVec::Dyn(vec) => vec.capacity(),
            BankVec::Inline(_) => C,
        }
    }

    /// Appends an element to the back of the collection.
    /// 
    /// If the resulting length would exceed `C`, the variant is converted into
    /// [`Dyn`](BankVec::Dyn).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<i32, 3>::from([1, 2]);
    /// bank.push(3);
    /// assert!(bank.is_inline()); // Still a fixed size data structure
    /// bank.push(4); /// Length exceeds `C`
    /// assert!(bank.is_dyn()); // Now points to a Vec
    /// assert_eq!(bank, [1, 2, 3, 4]);
    /// ```
    /// 
    /// # Time Complexity
    /// 
    /// Takes *O*(1) time if the new bank length does not exceed, or has already 
    /// exceeded, `C`, otherwise *O*(`C` + 1) time is needed to move the data 
    /// into a new variant.
    #[inline]
    pub fn push(&mut self, value: T) {
        match self {
            Self::Dyn(items) => items.push(value),
            Self::Inline(bank) => unsafe {
                if bank.len < C { bank.push_unchecked(value); } 
                else { self.into_vec_unchecked().push(value); }
            },
        }
    }

    /// Removes the last element of the bank and returns it, or None if it is empty.
    /// 
    /// If the `len <= C` after `pop`, the variant is converted to [`Inline`](BankVec::Inline).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<i32, 3>::from([1, 2, 3, 4]);
    /// assert!(bank.is_dyn()); // BankVec::Dyn
    /// assert_eq!(bank.pop(), Some(4));
    /// assert!(bank.is_inline()); // BankVec::Inline
    /// 
    /// ```
    /// 
    /// # Time Complexity
    /// 
    /// Takes *O*(1) time if the length after pop is greater or smaller than `C`,
    /// otherwise *O*(`C`) is needed to transform the variant.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        match self {
            Self::Dyn(vec) => {
                let popped = vec.pop();
                if vec.len() == C { unsafe { self.into_bank_unchecked(); } }
                popped
            },
            Self::Inline(bank) => bank.pop(),
        }
    }

    /// Inserts an element at position `index` within the bank, shifting all elements
    /// after it to the right.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<i32, 3>::from([1, 3]);
    /// 
    /// bank.insert(1, 2);
    /// 
    /// assert_eq!(bank, [1, 2, 3]);
    /// ```
    /// 
    /// # Time Complexity
    /// 
    /// Takes *O*(`BankVec::len - index`) time. All items after the insertion 
    /// index must be shifted right. In the worst cast, all elements are 
    /// shifted when insertion index is 0.  Should the new `len` exceed `C`, the
    /// variant is changed into [`Dyn`](BankVec::Dyn)
    pub fn insert(&mut self, index: usize, element: T) {
        match self {
            Self::Dyn(vec) => vec.insert(index, element),
            Self::Inline(bank) => match bank.len == C {
                true => unsafe { self.into_vec_unchecked().insert(index, element); },
                false => { bank.insert(index, element); }
            }
        }
    }
    
    /// Removes and returns the element at position `index` within the bank, 
    /// shifting all elements after it to the left.
    /// 
    /// This function has, at worst, *O*(n) performance. If you don't need to
    /// preserve the order of elements, use [`swap_remove`](BankArr::swap_remove)
    /// instead.
    /// 
    /// If the `len <= C` after `remove`, the variant becomes [`Inline`](BankVec::Inline).
    /// 
    /// # Panics
    /// 
    /// Panics if the `index` is out of bounds.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<i32, 3>::from([1, 2, 3]);
    /// assert_eq!(bank.remove(1), 2);
    /// assert_eq!(bank, [1, 3]);
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        match self {
            BankVec::Dyn(vec) => {
                let removed = vec.remove(index);
                if vec.len() == C { unsafe { self.into_bank_unchecked(); } }
                removed
            },
            BankVec::Inline(bank) => bank.remove(index),
        }
    }

    /// Remove aan element from the bank and returns it.
    /// 
    /// The removed element is replaced by the last element in the bank.  This
    /// doesnt preserve ordering ofthe remaining elements but **is** *O*(1).
    /// If you need to preserve ordering, use [`remove`](BankVec::remove).
    /// 
    /// If `BankVec::len <= C` after removal, the variant is transformed to 
    /// [`Inline`](BankVec::Inline).
    /// 
    /// # Panics
    /// 
    /// Panics if the `index` is out of bounds
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<i32, 5>::from([1, 2, 3, 4, 5]);
    /// assert_eq!(bank.swap_remove(2), 3);
    /// assert_eq!(bank, [1, 2, 5, 4]);
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> T {
        match self {
            BankVec::Dyn(vec) => {
                let removed = vec.swap_remove(index);
                if vec.len() == C { unsafe { self.into_bank_unchecked(); }}
                removed
            },
            BankVec::Inline(bank) => bank.swap_remove(index),
        }
    }

    /// Reserves the minimum capacity for at least additional more elements to be 
    /// inserted in the given BankVec. This will not deliberately over-allocate 
    /// to speculatively avoid frequent allocations. After calling reserve_exact, 
    /// capacity will be greater than or equal to self.len() + additional. Does 
    /// nothing if the capacity is already sufficient or the underlying variant
    /// is currenly [`Inline`](BankVec::Inline).
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<i32, 3>::from([1, 2, 3]);
    /// assert_eq!(bank.capacity(), 3);
    /// bank.reserve_exact(10); // Bank is currently `Inline`, so this does nothing.
    /// assert_eq!(bank.capacity(), 3);
    /// bank.push(4); // Converted to `Dyn` here.
    /// bank.reserve_exact(10); // Bank is `Dyn`.
    /// assert_eq!(bank.capacity(), 14);
    /// ```
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        match self {
            Self::Dyn(items) => items.reserve_exact(additional),
            _ => { }
        }
    }


    #[inline(always)]
    unsafe fn as_bank_unchecked_mut(&mut self) -> &mut BankArr<T, C> {
        match self {
            BankVec::Inline(bank) => bank,
            _ => unsafe { unreachable_unchecked() }
        }
    }

    #[inline(always)]
    unsafe fn as_vec_unchecked_mut(&mut self) -> &mut Vec<T> {
        match self {
            BankVec::Dyn(vec) => vec,
            _ => unsafe { unreachable_unchecked() }
        }
    }

    #[inline]
    unsafe fn into_bank_unchecked(&mut self) -> &mut BankArr<T, C> {

        let vec = match self {
            Self::Dyn(vec) => vec,
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
        
        *self = Self::Inline(bank);
        
        unsafe { self.as_bank_unchecked_mut() }
    }

    #[inline]
    unsafe fn into_vec_unchecked(&mut self) -> &mut Vec<T> {
        let bank = match self {
            Self::Inline(bank) => bank,
            _ => unsafe { unreachable_unchecked() }
        };
        let vec: Vec<T> = bank.data
            .iter()
            .map(|v| unsafe { v.assume_init_read() } )
            .collect();

        *self = Self::Dyn(vec);

        unsafe { self.as_vec_unchecked_mut() }
    }

    #[inline]
    pub fn as_dyn(&self) -> &Vec<T> {
        match self {
            BankVec::Dyn(vec) => vec,
            _ => panic!("Expected Inline")
        }
    }

    #[inline]
    pub fn as_inline(&self) -> &BankArr<T, C> {
        match self {
            BankVec::Inline(bank) => bank,
            _ => panic!("Expected Vec")
        }
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

        assert!(bankarr.is_inline());
        assert!(!bankarr.is_dyn());

        assert!(bankvec.is_dyn());
        assert!(!bankvec.is_inline());
    }

    #[test]
    fn as_variant() {
        let bankarr = B::from([1, 2, 3]);
        let bankvec = B::from([1, 2, 3, 4]); 

        assert_eq!(bankvec.as_dyn()[0], 1);
        assert_eq!(bankarr.as_inline()[0], 1);
    }

    #[test]
    #[should_panic]
    fn as_dyn_panics() {
        let bank = B::from([1, 2, 3]);
        let _ = bank.as_dyn();
    }

    #[test]
    #[should_panic]
    fn as_inline_panics() {
        let bank = B::from([1, 2, 3, 4]);
        let _ = bank.as_inline();
    }

    #[test]
    fn index() {
        let mut bank = B::from([1, 2, 3]);
        assert_eq!(bank[0], 1);
        assert_eq!(bank[2], 3);

        bank.push(4);
        assert_eq!(bank[3], 4);
    }

    #[test]
    fn index_mut() {
        let mut bank = B::from([1, 2, 3]);
        bank[0] = 7;
        assert_eq!(bank[0], 7);
        bank.push(4);
        bank[3] = 6;
        assert_eq!(bank[3], 6);

    }

    #[test]
    fn push() {
        let mut bank = B::new();
        bank.push(1);
        bank.push(2);
        bank.push(3);
        assert!(bank.is_inline());
        
        assert_eq!(bank[..1], [1]);
        assert_eq!(bank, [1, 2, 3]);
        
        bank.push(4);
        assert!(bank.is_dyn());
        bank.push(5);
        assert_eq!(bank, [1, 2, 3, 4, 5]);
    }


    #[test]
    fn pop() {
        let mut bank = B::from([3, 4, 5, 6]);
        
        assert!(bank.is_dyn());
        assert_eq!(bank.pop(), Some(6));

        assert!(bank.is_inline());
        assert_eq!(bank.pop(), Some(5))
    }

    #[test]
    fn remove() {
        let mut bank = B::from([3, 4, 5, 6]);

        assert!(bank.is_dyn());
        let removed = bank.remove(1);
        assert_eq!(removed, 4);
        assert_eq!(bank, [3, 5, 6]);

        assert!(bank.is_inline());
        let removed = bank.remove(1);
        assert_eq!(removed, 5);
        assert_eq!(bank, [3, 6]);
    }

    #[test]
    fn swap_remove() {
        let mut bank = BankVec::<String, 3>::from(["aa".to_string(), "bb".to_string(), "cc".to_string(), "dd".to_string()]);
        
        assert!(bank.is_dyn());
        let removed = bank.swap_remove(0);
        assert_eq!(removed, "aa".to_string());

        assert!(bank.is_inline());
        let removed = bank.swap_remove(1);
        assert_eq!(removed, "bb".to_string());

        assert_eq!(bank, ["dd".to_string(), "cc".to_string()])
    }

    #[test]
    fn insert() {
        let mut bank = B::from([3, 5]);

        bank.insert(2, 6);
        assert!(bank.is_inline());
        bank.insert(1, 4);
        
        assert!(bank.is_dyn());
        bank.insert(4, 7);

        assert_eq!(bank, [3, 4, 5, 6, 7]);
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
        assert!(bank.is_inline());
        let mut iter = bank.iter_mut();
        for s in ["a", "b", "c"] {
            assert_eq!(iter.next(), Some(s).as_mut());
        }
        assert_eq!(iter.next(), None);


        bank.push("d");
        assert!(bank.is_dyn());
        let mut iter = bank.iter_mut();
        for s in ["a", "b", "c", "d"] {
            assert_eq!(iter.next(), Some(s).as_mut());
        }
        assert_eq!(iter.next(), None);
    }


    #[test]
    fn iter() {
        let mut bank = BankVec::<&'static str, 3>::from(["a", "b", "c"]);
        assert!(bank.is_inline());
        let mut iter = bank.iter();
        for s in ["a", "b", "c"] {
            assert_eq!(iter.next(), Some(s).as_ref());
        }
        assert_eq!(iter.next(), None);


        bank.push("d");
        assert!(bank.is_dyn());
        let mut iter = bank.iter();
        for s in ["a", "b", "c", "d"] {
            assert_eq!(iter.next(), Some(s).as_ref());
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn as_slice() {
        let mut bank = B::from([3, 4, 5]);
        assert!(bank.is_inline());
        assert_eq!(bank.as_slice(), [3, 4, 5]);

        bank.push(6);
        assert!(bank.is_dyn());
        assert_eq!(bank.as_slice(), [3, 4, 5, 6]);
    }

    #[test]
    fn as_slice_mut() {
        let mut bank = B::from([3, 4, 5]);
        assert!(bank.is_inline());
        assert_eq!(bank.as_slice(), [3, 4, 5]);

        bank.push(6);
        assert!(bank.is_dyn());
        assert_eq!(bank.as_mut_slice(), [3, 4, 5, 6]);
    }

    #[test]
    fn from_vec() {
        let bankarr = B::from(vec![3, 4, 5]);
        assert_eq!(bankarr, [3, 4, 5]);
        
        let bankvec = B::from(vec![3, 4, 5, 6]);
        assert_eq!(bankvec, [3, 4, 5, 6]);
    }

    #[test]
    fn clone() {
        let bankarr = B::new();
        let bankvec = B::from([3, 4, 5, 6]);

        assert!(bankarr == bankarr.clone());
        assert!(bankvec == bankvec.clone());
        assert!(bankvec != bankarr);
    }


    #[test]
    fn partial_eq() {
        let mut bank = BankVec::<i32, 2>::from([1, 2]);
        let vec = vec![1, 2];
        assert_eq!(bank, [1, 2]);
        assert_eq!(bank, &[1, 2]);
        assert_eq!(bank, *[1, 2].as_slice());
        assert_eq!(bank, vec.as_slice());
        assert_eq!(bank, vec);

        bank.push(3); // Variant transforms to `Dyn`
        let vec = vec![1, 2, 3];
        assert_eq!(bank, [1, 2, 3]);
        assert_eq!(bank, &[1, 2, 3]);
        assert_eq!(bank, *[1, 2, 3].as_slice());
        assert_eq!(bank, vec.as_slice());
        assert_eq!(bank, vec);
    }
}