

use std::{mem::{ManuallyDrop, MaybeUninit}, ops::{self, Deref, DerefMut, Index, IndexMut}, ptr::{self, NonNull}, slice::{self, SliceIndex}};
use crate::{drain, errors::BankFullError};


/// A fixed-size contiguous growable array type.
/// 
/// Can shrink and grow up to `C` like a [`Vec`], but is fixed size in memory.
/// 
/// # Examples
/// 
/// ```
/// use bankarr::BankArr;
/// 
/// let mut bank = BankArr::<i32, 16>::new();
/// bank.push(3);
/// bank.push(7);
/// 
/// assert_eq!(bank.len(), 2);
/// assert_eq!(bank[0], 3);
/// 
/// assert_eq!(bank.pop(), Some(7));
/// assert_eq!(bank.len(), 1);
/// 
/// bank[0] = 19;
/// assert_eq!(bank[0], 19);
/// 
/// bank.extend([20, 21]);
/// for v in &bank {
///     println!("{v}");
/// }
/// 
/// assert_eq!(bank, [19, 20, 21]);   
/// ```
/// 
/// You can build a bank using the [`From`] trait.  Note: The consumed collection
/// may be less than or equal to the specified capacity.
/// ```
/// use bankarr::BankArr;
/// // Here the bank has half its allocation remaining.
/// let mut bank = BankArr::<i32, 4>::from([1, 2]); 
/// assert_eq!(bank.remaining_capacity(), 2);
/// 
/// ```
/// Trying to create a bank from a collection larger than its capacity will panic.
/// ```should_panic
/// use bankarr::BankArr;
/// let mut bank = BankArr::<i32, 3>::from([1, 2, 3, 4]); // panics! 
/// ```
/// 
/// # Indexing
/// 
/// `BankArr` allows access to values by index just as you'd get in a vec because
/// it implements the [`Index`] trait.
/// 
/// ```
/// use bankarr::BankArr;
/// 
/// let bank = BankArr::<i32, 3>::from([1, 2, 3]);
/// println!("{}", bank[1]); // prints `2`
/// ```
/// 
/// Indexing out of bounds will cause a panic.
/// ```should_panic
/// use bankarr::BankArr;
/// 
/// let bank = BankArr::<i32, 3>::from([1, 2, 3]);
/// println!("{}", bank[3]); // Panics!
/// ```
/// 
/// # Slicing
/// 
/// You can easily slice `BankArr`
/// ```
/// use bankarr::BankArr;
/// 
/// fn read_slice(slice: &[i32]) {
///     // ...
/// }
/// 
/// let bank = BankArr::<i32, 3>::from([1, 2, 3]);
/// read_slice(&bank);
/// 
/// // You can acquire a slice using the following as  well
/// let u: &[i32] = &bank;
/// let u: &[_] = &bank;
/// // etc.
/// 
/// ```
/// 
/// # Capacity
/// 
/// The capacity of a `BankArr` is determined by its generic, `C`.  At instantiation,
/// the full capacity is allocated and available.  This may also mean that creating
/// large banks can be expensive, though this is offset somewhat because the allocated
/// space is uninitialized. 
/// 
/// No methods may change the capacity of the bank, much the same as an array 
/// `[T; C]` has a fixed size.  Various methods such as [`push`](Self::push)
/// may fail if the bank is already at capacity. Generally there are safe 
/// alternatives, .i.e [`try_push`](Self::try_push) which return a [`Result`].
/// 
#[derive(Debug)]
pub struct BankArr<T, const C: usize> {
    pub(crate) data: [MaybeUninit<T>; C],
    pub(crate) len: usize,
}

impl <T, const C: usize> Deref for BankArr<T, C> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target { &self.as_slice() }
}

impl <T, const C: usize> DerefMut for BankArr<T, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { self.as_mut_slice() }
}

impl<T, const C: usize, I: SliceIndex<[T]>> Index<I> for BankArr<T, C> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, const C: usize, I: SliceIndex<[T]>> IndexMut<I> for BankArr<T, C> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl<'a, T, const C: usize> IntoIterator for &'a BankArr<T, C> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a, T, const C: usize> IntoIterator for &'a mut BankArr<T, C> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

impl<T: PartialEq, const C: usize> PartialEq for BankArr<T, C> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len &&
        self.as_slice() == other.as_slice()
    }
}

impl<T: PartialEq, const C: usize, const N: usize> PartialEq<[T; N]> for BankArr<T, C> {
    fn eq(&self, other: &[T; N]) -> bool {
        self.as_slice() == other
    }
}

impl<T: PartialEq, const C: usize, const N: usize> PartialEq<&[T; N]> for BankArr<T, C> {
    fn eq(&self, other: &&[T; N]) -> bool {
        self.len == other.len() && self.as_slice() == *other
    }
}

impl<T: PartialEq, const C: usize> PartialEq<Vec<T>> for BankArr<T, C> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.len == other.len() && self.as_slice() == other
    }
}

impl<T: PartialEq, const C: usize> PartialEq<[T]> for BankArr<T, C> {
    fn eq(&self, other: &[T]) -> bool {
        self.len == other.len() && self.as_slice() == other
    }
}

impl<T: PartialEq, const C: usize> PartialEq<&[T]> for BankArr<T, C> {
    fn eq(&self, other: &&[T]) -> bool {
        self.len == other.len() && self.as_slice() == *other
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

impl<T, const C: usize> Extend<T> for BankArr<T, C> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, items: I) {

        let (mut ptr, mut end) = unsafe {
            let ptr: *mut T = self.as_mut_ptr();
            let end: *const T = if Self::IS_ZST { (ptr as usize + C - self.len) as _ } 
                else { ptr.add(C) as _ };
            (ptr.add(self.len), end)
        };

        items.into_iter().for_each(|val| {
            match (ptr == end as _, Self::IS_ZST) {
                (true, _) => panic!("capacity exceeded during operation `extend`"),
                (_, true) => { end = (end as usize - 1) as _; },
                (_, false) => unsafe {
                    ptr.write(val);
                    ptr = ptr.add(1);
                }
            }
            self.len += 1;
        });
    }
}

#[cfg(not(tarpaulin_include))] // Drain's drop implicitly tests this
impl<'a, T, const C: usize> drain::Drainable<'a, T> for BankArr<T, C> {
    fn drain_parts(&'a mut self) -> (ptr::NonNull<T>, &'a mut usize) {
        (
            unsafe { NonNull::new_unchecked(self.as_mut_ptr()) },
            &mut self.len
        )
    }
}

impl <T, const C: usize, const N: usize> From<[T; N]> for BankArr<T, C> {

    /// Create a new instance from an array.
    /// 
    /// The array consumed may be smaller than the specified bank size `C`.
    /// 
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let bank = BankArr::<i32, 3>::from([1, 2]);
    /// 
    /// ```
    /// # Panics
    /// 
    /// Panics if the consumed array exceeds the length of the bank's size.
    /// ```should_panic
    /// use bankarr::BankArr;
    /// 
    /// let bank = BankArr::<i32, 2>::from([1, 2, 3]); // Panics!
    /// ```
    fn from(arr: [T; N]) -> Self {
        assert!(N <= C);
        
        let arr = ManuallyDrop::new(arr);
        let mut bank = Self {
            data: [const { MaybeUninit::uninit() }; C],
            len: N
        };

        //for (idx, val) in arr.into_iter().enumerate() { unsafe {
        //    *bank.data.get_unchecked_mut(idx) = MaybeUninit::new(val);
        //}}
        
        unsafe { ptr::copy_nonoverlapping(
            arr.as_ptr().cast(), 
            bank.as_mut_ptr(), 
            N
        )}
        bank
    }
}

impl <T, const C: usize> From<Vec<T>> for BankArr<T, C> {

    /// Create a new instance from vec.
    /// 
    /// The consumed vec may be smaller than the specified bank size `C`.
    /// 
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let bank = BankArr::<i32, 3>::from(vec![1, 2]);
    /// 
    /// ```
    /// # Panics
    /// 
    /// Panics if the consumed vec exceeds the length of the bank's size.
    /// ```should_panic
    /// use bankarr::BankArr;
    /// 
    /// let bank = BankArr::<i32, 2>::from(vec![1, 2, 3]); // Panics!
    /// ```
    fn from(vec: Vec<T>) -> Self {
        let len = vec.len();
        assert!(len <= C);

        let mut data = [const {MaybeUninit::uninit() }; C];

        for (idx, val) in vec.into_iter().enumerate() { unsafe { 
            *data.get_unchecked_mut(idx) = MaybeUninit::new(val);
        }}
        
        Self { data, len }
    }
}

impl <T, const C: usize> From<BankArr<T, C>> for Vec<T> {
    fn from(bank: BankArr<T, C>) -> Self {
        unsafe { 
            bank.data
                .get_unchecked(..bank.len)
                .iter()
                .map(|v| v.assume_init_read())
                .collect()
        }
    }
}

impl <T, const C: usize> Drop for BankArr<T, C> {
    fn drop(&mut self) {
        unsafe {
            ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len).drop_in_place();
        }
    }
}

impl <T, const C: usize> BankArr<T, C> {

    const IS_ZST: bool = std::mem::size_of::<T>() == 0;

    /// Constructs a new, empty `BankArr<T, C>`
    /// 
    /// This *will* allocate space for the entire bank.
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let mut bank = BankArr::<i32, 3>::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            data: [const { MaybeUninit::uninit() }; C],
            len: 0,
        }
    }

    /// Returns the length of the bank.
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let mut bank = BankArr::<i32, 3>::new();
    /// assert_eq!(bank.len(), 0);
    /// 
    /// bank.push(5);
    /// assert_eq!(bank.len(), 1);
    /// ```
    #[inline(always)]
    pub const fn len(&self) -> usize { self.len }

    /// Returns the remaining capacity of the bank.
    /// 
    /// Simply, `C - BankArr::len`.
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankArr;
    /// let mut bank = BankArr::<i32, 3>::new();
    /// assert_eq!(bank.remaining_capacity(), 3);
    /// bank.push(1);
    /// bank.push(2);
    /// assert_eq!(bank.remaining_capacity(), 1);
    /// ```
    #[inline(always)]
    pub const fn remaining_capacity(&self) -> usize { C - self.len }

    /// Appends an element to the back of the collection.
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity exceeds the size, `C`.
    /// For a panic-free `push`, see [`try_push`](BankArr::try_push).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let mut bank = BankArr::<i32, 3>::from([1, 2]);
    /// bank.push(3);
    /// assert_eq!(bank, [1, 2, 3]);
    /// ```
    /// 
    /// # Time Complexity
    /// 
    /// Takes *O*(1) time.
    #[inline]
    pub fn push(&mut self, value: T) {
        assert!(self.len < C);
        unsafe { self.push_unchecked(value) }
    }

    /// Attempts to append an element to the back of the collection.
    /// Returns a [`Result`] indicating success.
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let mut bank = BankArr::<i32, 3>::from([1, 2]);
    /// 
    /// let ok = bank.try_push(3);
    /// assert!(ok.is_ok());
    /// 
    /// let err = bank.try_push(4);
    /// assert!(err.is_err());
    /// ```
    /// 
    /// # Time Complexity
    /// 
    /// Takes *O*(1) time.
    #[inline]
    pub fn try_push(&mut self, value: T) -> Result<(), BankFullError> {
        if self.len == C { return Err(BankFullError {}) }
        unsafe { self.push_unchecked(value) }
        Ok(())
    }

    #[inline(always)]
    const fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as _
    }

    #[inline(always)]
    const fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as _
    }
    

    /// Appends an element to the back of the collection without doing bounds 
    /// checking.
    /// 
    /// # Safety
    /// 
    /// Calling this method on a filled `BankArr` is [undefined behavior](<https://doc.rust-lang.org/reference/behavior-considered-undefined.html>).
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let mut bank = BankArr::<i32, 3>::from([1, 2]);
    /// unsafe { bank.push_unchecked(3); }
    /// assert_eq!(bank, [1, 2, 3])
    /// ```
    /// 
    /// # Time Complexity
    /// 
    /// Takes *O*(1) time.
    #[inline(always)]
    pub unsafe fn push_unchecked(&mut self, value: T) {
        let len = self.len;
        debug_assert!(len < C);
        unsafe { self.as_mut_ptr().add(len).write(value); }
        self.len = len + 1;
    }

    /// Removes the last element of the bank and returns it, or None if it is empty.
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let mut bank = BankArr::<i32, 3>::from([1, 2, 3]);
    /// assert_eq!(bank.pop(), Some(3));
    /// ```
    /// 
    /// # Time Complexity
    /// 
    /// Takes *O*(1) time.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        match self.len == 0 {
            true => None,
            false => unsafe {
                self.len -= 1;
                core::hint::assert_unchecked(self.len < self.data.len());
                Some(self.as_ptr().add(self.len).read())
            }
        }
    }

    /// Inserts an element at position `index` within the bank, shifting all elements after it to the right.
    /// 
    /// # Panics
    /// 
    /// Panics if if `index > len` OR if `len == C`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let mut bank = BankArr::<i32, 3>::from([1, 3]);
    /// 
    /// bank.insert(1, 2);
    /// 
    /// assert_eq!(bank, [1, 2, 3]);
    /// ```
    /// 
    /// # Time Complexity
    /// 
    /// Takes *O*(`BankArr::len - index`) time. All items after the insertion 
    /// index must be shifted right. In the worst cast, all elements are 
    /// shifted when insertion index is 0.
    pub fn insert(&mut self, index: usize, element: T) -> bool {
        assert!(index <= self.len, "Index out of bounds");
        if self.len == C { return false }

        unsafe {
            let ptr = self.as_mut_ptr().add(index);
            ptr.copy_to(ptr.add(1), self.len - index);
            ptr.write(element);
        }
        self.len += 1;
        true
    }

    /// Removes and returns the element at position `index` within the bank, 
    /// shifting all elements after it to the left.
    /// 
    /// This function has, at worst, *O*(n) performance. If you don't need to
    /// preserve the order of elements, use [`swap_remove`](BankArr::swap_remove)
    /// instead.
    /// 
    /// # Panics
    /// 
    /// Panics if the `index` is out of bounds.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let mut bank = BankArr::<i32, 3>::from([1, 2, 3]);
    /// assert_eq!(bank.remove(1), 2);
    /// assert_eq!(bank, [1, 3]);
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "Index out of bounds");
        self.len -= 1;
        unsafe {
            let removed = self.as_mut_ptr().add(index).read();
            let ptr = self.as_mut_ptr().add(index);
            ptr.add(1).copy_to(ptr, self.len - index);
            removed
        }
    }

    /// Removes an element from the bank and returns it.
    /// 
    /// The removed element is replaced by the last element in the bank.  This
    /// doesn't preserve ordering of the remaining elements but **is** *O*(1).
    /// If you need to preserve ordering, use [`remove`](BankArr::remove).
    /// 
    /// # Panics
    /// 
    /// Panics if `index` is out of bounds.
    /// 
    /// # Examples 
    /// 
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let mut bank = BankArr::<i32, 5>::from([1, 2, 3, 4, 5]);
    /// assert_eq!(bank.swap_remove(2), 3);
    /// assert_eq!(bank, [1, 2, 5, 4]);
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "Index out of bounds");
        self.len -= 1;
        unsafe {
            let ptr = self.data.as_mut_ptr();
            ptr.add(index).replace(ptr.add(self.len).read()).assume_init()
        }

    }

    /// Removes all elements from the bank and returns a double-ended iterator over
    /// the elements.
    /// 
    /// If the iterator is dropped before being fully consumed, it drops the
    /// remaining elements.
    /// 
    /// The returned iterator keeps a mutable borrow on the bank to optimize its
    /// implementation.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let mut bank = BankArr::<i32, 3>::from([1, 2, 3]);
    /// let drained: Vec<_> = bank.drain(..).collect();
    /// 
    /// assert_eq!(drained, [1, 2, 3]);
    /// assert_eq!(bank.len(), 0);
    /// assert_eq!(bank, []);
    /// ```
    /// 
    pub fn drain<R>(&mut self, range: R) -> drain::Drain<'_, T, Self> 
    where 
        R: ops::RangeBounds<usize>,
    {
        let ptr: *const T = self.as_ptr();
        let len = self.len;
        let ops::Range { start, end } = drain::slice_range(range, ..len);

        unsafe {
            self.len = start;
            drain::Drain {
                tail_start: end,
                tail_len: len - end,
                iter: slice::from_raw_parts(ptr.add(start), end - start).iter(),
                bank: NonNull::new_unchecked(self)
            }
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
    /// use bankarr::BankArr;
    /// 
    /// let bank = BankArr::<u8, 3>::from([1, 2, 3]);
    /// io::sink().write(bank.as_slice()).unwrap();
    /// ```
    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        // We are tracking initialized values via len, ensuring the slice is not UB
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
    }


    /// Extracts a mutable slice containing the entire bank.
    /// 
    /// Equivalent to `&mut bank[..]`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use std::io::{self, Read};
    /// use bankarr::BankArr;
    /// 
    /// let mut bank = BankArr::<u8, 3>::from([0; 3]);
    /// io::repeat(0b101).read_exact(bank.as_mut_slice()).unwrap();
    /// ```
    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        // We are tracking initialized values via len, ensuring the slice is not UB
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    #[inline]
    fn truncate(&mut self, len: usize) {
        if len > self.len { return }

        unsafe {
            let rem = self.len - len;
            let s = ptr::slice_from_raw_parts_mut(self.as_mut_ptr().add(len), rem);
            self.len = len;
            s.drop_in_place();
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0);
    }

}

impl<T: PartialEq, const C: usize> BankArr<T, C> {


    /// Removes the item from the bank and returns true if the item existed,
    /// otherwise returns false.
    /// 
    /// Performs a [`swap_remove`](BankArr::swap_remove) on the value if found.
    /// Does *NOT* preserve ordering.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankArr;
    /// 
    /// let mut bank = BankArr::<i32, 4>::from([1, 2, 3, 4]);
    /// 
    /// assert!(bank.remove_item(&2));
    /// assert!(!bank.remove_item(&2));
    /// 
    /// assert_eq!(bank, [1, 4, 3]);
    /// ```
    ///
    #[inline]
    pub fn remove_item(&mut self, value: &T) -> bool {
        unsafe {
            let ptr: NonNull<T> = NonNull::new_unchecked(self.data.as_mut_ptr().cast());

            for index in 0usize..self.len {
                let cp_ptr = ptr.add(index);
                if cp_ptr.as_ref() == value {
                    self.len -= 1;
                    cp_ptr.replace(ptr.add(self.len).read());
                    return true
                }
            }
        }

        false
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    type B = BankArr<u32, 4>;

    #[test]
    fn remaining_capacity() {
        let mut bank = B::from([1, 2]);
        assert_eq!(bank.remaining_capacity(), 2);
        bank.push(3);
        assert_eq!(bank.remaining_capacity(), 1);
    }

    #[test]
    fn index() {
        let bank = B::from([1, 2, 3]);
        assert_eq!(bank[0], 1);
        assert_eq!(bank[2], 3);
    }

    #[test]
    fn index_mut() {
        let mut bank = B::from([1, 2, 3]);
        bank[0] = 7;
        assert_eq!(bank[0], 7);
    }

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
        assert_eq!(bank, [3, 5]);
    }

    #[test]
    fn swap_remove() {
        let mut bank: BankArr<String, 3> = BankArr::from(["aa".to_string(), "bb".to_string(), "cc".to_string()]);
        let removed = bank.swap_remove(0);

        assert_eq!(removed, "aa".to_string());
        assert_eq!(bank, ["cc".to_string(), "bb".to_string()]);
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
        assert_eq!(bank, [3, 4, 5, 6]);
    }

    #[test]
    #[should_panic]
    fn insert_out_of_bounds() {
        let mut bank = B::from([3, 4]);

        bank.insert(3, 0);
    }

    #[test]
    fn extend() {
        let mut bank = BankArr::<i32, 16>::from([1, 2]);
        bank.extend([3, 4, 5]);

        assert_eq!(bank, [1, 2, 3, 4, 5]);

        let mut bank = BankArr::<(), 16>::from([(), ()]);
        bank.extend([(); 4]);
        assert_eq!(bank, [(); 6]);
    }

    #[test]
    #[should_panic]
    fn extend_panics() {
        let mut bank = BankArr::<i32, 3>::from([1, 2]);
        bank.extend([3, 4, 5]);
    }

    #[test]
    #[should_panic]
    fn extend_zst_panics() {
        let mut bank = BankArr::<(), 3>::from([(), ()]);
        bank.extend([(), ()]);
    }

    #[test]
    fn drain() {
        let mut bank = B::from([3, 4, 5]);
        let drained = bank.drain(..)
            .into_iter().collect::<Vec<u32>>();

        assert_eq!(bank.len(), 0);
        assert_eq!(drained, vec![3, 4, 5]);

        let mut bank = B::from([3, 4]);
        let mut drain = bank.drain(..);
        assert_eq!(drain.next_back(), Some(4));
        assert_eq!(drain.next(), Some(3));
        assert_eq!(drain.next(), None);
    }

    #[test]
    fn drain_zst() {
        let mut bank = BankArr::<(), 2>::from([(), ()]);
        let mut drain = bank.drain(..);
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

        assert_eq!(bank, collected); 
    }

    #[test]
    fn iter_mut() {
        let mut bank = B::from([3, 4, 5]);
        let collected = bank.iter_mut()
            .map(|v| *v)
            .collect::<Vec<u32>>();

        assert_eq!(bank, collected); 
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
        assert_eq!(bank, ["dd".to_string(), "ff".to_string()])
    }

    #[test]
    fn clone() {
        let bank = BankArr::<_, 2>::from(["aa".to_string(), "bb".to_string()]);
        assert_eq!(bank, bank.clone());
    }

    #[test]
    fn to_vec() {
        let bank = BankArr::<i32, 4>::from([1, 2, 3, 4]);

        let vec: Vec<i32> = bank.into();
        assert_eq!(vec, [1, 2, 3, 4]);

    }

    #[test]
    fn partial_eq() {
        let bank = BankArr::<i32, 2>::from([1, 2]);
        let vec = vec![1, 2];
        assert_eq!(bank, [1, 2]);
        assert_eq!(bank, &[1, 2]);
        assert_eq!(bank, *[1, 2].as_slice());
        assert_eq!(bank, vec.as_slice());
        assert_eq!(bank, vec);
    }

    #[test]
    fn truncate() {
        let mut bank = BankArr::<i32, 3>::from([1, 2, 3]);
        bank.truncate(1);
        assert_eq!(bank, [1]);
        assert_eq!(bank.len(), 1);
    }

    #[test]
    fn clear() {
        let mut bank = BankArr::<i32, 3>::from([1, 2]);
        bank.clear();
        assert_eq!(bank, []);
        assert_eq!(bank.len(), 0);
    }

    #[test]
    fn remove_item() {
        let mut bank = BankArr::<i32, 3>::from([1, 2, 3]);
        assert!(bank.remove_item(&2));
        assert!(!bank.remove_item(&2));

        assert_eq!(bank.len(), 2);
        assert_eq!(bank, [1, 3]);

        let mut bank = BankArr::<String, 3>::from(["aa".to_string(), "bb".to_string(), "cc".to_string()]);

        assert!(bank.remove_item(&"aa".to_string()));
        assert!(!bank.remove_item(&"aa".to_string()));

        assert_eq!(bank.len(), 2);
        assert_eq!(bank, ["cc".to_string(), "bb".to_string()]);
    }
}