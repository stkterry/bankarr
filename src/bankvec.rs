
use core::slice;
use std::{mem::{self, ManuallyDrop}, ops::{self, Deref, DerefMut, Index, IndexMut}, ptr::{self, NonNull}, slice::SliceIndex};

mod allocation;
mod buffer_union;

use crate::{drain, errors::AllocErr};
use buffer_union::*;
use allocation::*;

/// A fixed-size contiguous growable array type with spillover.
/// 
/// [`push`](BankVec::push) / [`pop`](BankVec::pop) like semantics with a fixed-size
/// array up to capacity `C`, after which it moves to a heap-like vector.
/// 
/// 
/// # Examples
/// ```
/// use bankarr::BankVec;
/// 
/// let mut bank = BankVec::<i32, 2>::new();
/// bank.push(3);
/// bank.push(20);
/// 
/// assert!(!bank.on_heap());
/// bank.push(9); // Changes to a Vec under the hood.
/// assert!(bank.on_heap());
/// 
/// assert_eq!(bank.len(), 3);
/// assert_eq!(bank[0], 3);
/// 
/// assert_eq!(bank.pop(), Some(9)); // Drops back into a BankArr
/// assert_eq!(bank.len(), 2);
/// 
/// bank[0] = 19;
/// assert_eq!(bank[0], 19);
/// 
/// bank.extend([21, 22]);
/// for v in &bank {
///     println!("{v}");
/// }
/// 
/// assert_eq!(bank, [19, 20, 21, 22]); 
/// ```
/// 
/// # Indexing
/// 
/// `BankVec` allows access to values by index just as you'd get from a vec because
/// it implements the [`Index`] trait.
/// 
/// ```
/// use bankarr::BankVec;
/// 
/// let bank = BankVec::<i32, 3>::from([1, 2, 3]);
/// println!("{}", bank[1]); // prints `2`
/// ```
/// 
/// Indexing out of bounds will cause a panic.
/// ```should_panic
/// use bankarr::BankVec;
/// 
/// let bank = BankVec::<i32, 3>::from([1, 2, 3]);
/// println!("{}", bank[3]); // Panics!
/// ``` 
/// 
/// # Slicing
/// 
/// You can easily slice `BankVec`
/// ```
/// use bankarr::BankVec;
/// 
/// fn read_slice(slice: &[i32]) {
///     // ...
/// }
/// 
/// let bank = BankVec::<i32, 3>::from([1, 2, 3]);
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
/// As with a [`BankArr`], the underlying capacity is specified by its generic, `C`.
/// Unlike `BankArr` however, is that you may continue to push into the data structure
/// as much as you like.  The caveat is performance.
/// 
/// It takes *O*(`C`) time to move elements into a heap allocated `Vec`.  Moreover
/// you lose the performance of a stack allocated, fixed-size structure while the 
/// [`len`](BankVec::len) exceeds `C`. Dropping back into the fixed-size structure
/// also has the same *O*(`C`) cost!
/// 
/// `BankVec` carries a small performance overhead in order to manage two possible
/// configurations.  If you know your data won't exceed some fixed, maximum size,
/// prefer [`BankArr`] instead. Its performance is equivalent to that of an array `[T; C]`.
/// 
/// [`BankArr`]: crate::BankArr
pub struct BankVec<T, const C: usize> {
    buf: BufferUnion<T, C>,
    capacity: usize,
}

#[cfg(not(tarpaulin_include))]
impl<T: std::fmt::Debug, const C: usize> std::fmt::Debug for BankVec<T, C> 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        const VEC_FIELD: &'static str = "buf (Vec)";
        const ARR_FIELD: &'static str = "buf (Array)";

        let (field, capacity) = match self.on_heap() {
            true => (VEC_FIELD, self.capacity),
            false => (ARR_FIELD, C)
        };
        
        let name = std::fmt::format(format_args!("BankVec<T, {}>", C));
        f.debug_struct(&name)
            .field(field, &self.as_slice())
            .field("capacity", &capacity)
            .finish()
    
    }
}

impl <T, const C: usize> Deref for BankVec<T, C> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target { &self.as_slice() }
}

impl <T, const C: usize> DerefMut for BankVec<T, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { self.as_mut_slice() }
}

impl<T, const C: usize, I: SliceIndex<[T]>> Index<I> for BankVec<T, C> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output { 
        Index::index(&**self, index) }
}

impl<T, const C: usize, I: SliceIndex<[T]>> IndexMut<I> for BankVec<T, C> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output { IndexMut::index_mut(&mut **self, index) }
}

impl<'a, T, const C: usize> IntoIterator for &'a BankVec<T, C> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a, T, const C: usize> IntoIterator for &'a mut BankVec<T, C> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

impl<T: PartialEq, const C: usize> PartialEq for BankVec<T, C> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
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

impl<T: Clone, const C: usize> Clone for BankVec<T, C> {
    fn clone(&self) -> Self {
        use ptr::copy_nonoverlapping as cp;

        if self.on_heap() {
            let (ptr, len, _) = unsafe { self.heap() };
            let mut cloned = Self {
                buf: BufferUnion::heap_from(NonNull::dangling(), 0),
                capacity: 0
            };
            cloned.reserve(len);
            unsafe { cp(ptr, cloned.buf.heap.0.as_ptr(), len) }
            cloned.buf.heap.1 = len;

            cloned
        } else {
            let (ptr, len, _) = unsafe { self.stack() };
            let mut buf = BufferUnion::new_stack();
            unsafe { cp(ptr, buf.stack_ptr_nn().as_ptr(), len) }
            Self { buf, capacity: len }
        }
    }
}

impl<T, const C: usize> Extend<T> for BankVec<T, C> {

    /// Extends a collection with the contents of an iterator.  
    /// Will reallocate onto the heap if necessary.
    fn extend<I: IntoIterator<Item = T>>(&mut self, items: I) {

        let mut iter = items.into_iter();
        let (ptr, len, cap) = self.data_buf_mut();

        let ptr = ptr.as_ptr();
        let mut cp_len = *len;

        while cp_len < cap {
            if let Some(value) = iter.next() {
                unsafe { ptr.add(cp_len).write(value) }
                cp_len += 1;
            } else { break }
        }
        *len = cp_len;

        // This produces identical results to the while loop above
            //for idx in cp_len..cap {
            //    if let Some(value) = iter.next() {
            //        unsafe { ptr.add(idx).write(value) }
            //    } else {
            //        *len = idx;
            //        break;
            //    }
            //}
        //

        iter.for_each(|value| self.push(value))
    }
}


#[cfg(not(tarpaulin_include))] // Drain's drop implicitly tests this
impl<'a, T, const C: usize> drain::Drainable<'a, T> for BankVec<T, C> {
    fn drain_parts(&'a mut self) -> (NonNull<T>, &'a mut usize) {
        let (ptr, len, _) = self.data_buf_mut();
        (ptr, len)
    }
}

impl<T, const C: usize> From<Vec<T>> for BankVec<T, C> {

    /// Create a new instance from a vec.
    /// 
    /// The vec consumed may be smaller or larger than the specified bank size `C`.
    /// If the consumed vec is larger, then the it will be stored on the heap
    /// automatically, otherwise it will be created as the faster stack based array.
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let bank1 = BankVec::<i32, 3>::from(vec![1, 2]);
    /// assert!(!bank1.on_heap());
    /// 
    /// let bank2 = BankVec::<i32, 3>::from(vec![1, 2, 3, 4]);
    /// assert!(bank2.on_heap());
    /// ```
    /// 
    #[inline]
    fn from(mut vec: Vec<T>) -> Self {
        use ptr::copy_nonoverlapping as cp;

        let len = vec.len();
        if len <= C {
            let mut buf = BufferUnion::new_stack();
            unsafe { vec.set_len(0); }
            unsafe { cp(vec.as_ptr(), buf.stack_ptr_nn().as_ptr(), len); }

            Self { buf, capacity: len }
        } else {
            let (ptr, cap, len) = (vec.as_mut_ptr(), vec.capacity(), vec.len());
            mem::forget(vec);
            let ptr = NonNull::new(ptr).expect("Uh oh");

            Self {
                buf: BufferUnion::heap_from(ptr, len),
                capacity: cap,
            }
        }
    }
}

impl<T, const C: usize, const N: usize> From<[T; N]> for BankVec<T, C> {

    /// Create a new instance from an array.
    /// 
    /// The array consumed may be smaller or larger than the specified bank size `C`.
    /// If the consumed array is larger, then the it will be stored on the heap
    /// automatically, otherwise it will be created as the faster stack based array.
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let bank1 = BankVec::<i32, 3>::from([1, 2]);
    /// assert!(!bank1.on_heap());
    /// 
    /// let bank2 = BankVec::<i32, 3>::from([1, 2, 3, 4]);
    /// assert!(bank2.on_heap());
    /// ```
    fn from(arr: [T; N]) -> Self {

        let arr = ManuallyDrop::new(arr);
        let ptr = unsafe { NonNull::new_unchecked(arr.as_ptr().cast_mut()) };

        if N <= C {
            let mut buf = BufferUnion::new_stack();
            unsafe { ptr.copy_to_nonoverlapping(buf.stack_ptr_nn(), N);}
            Self { buf, capacity: N }
        } else {
            let mut bank = Self { buf: BufferUnion::new_heap(), capacity: 0, };
            bank.reserve(N);
            unsafe { ptr.copy_to_nonoverlapping(bank.buf.heap.0, N);}
            bank.buf.heap.1 = N;
            
            bank
        }

    }
}



impl<T, const C: usize> Drop for BankVec<T, C> {
    fn drop(&mut self) {
        match self.on_heap() {
            true => unsafe {
                let (ptr, &mut len, _) = self.heap_mut();
                drop(Vec::from_raw_parts(ptr.as_ptr(), len, self.capacity))
            },
            false => unsafe { ptr::drop_in_place(&mut self[..]); }
        }
    }
}

impl<T, const C: usize> BankVec<T, C> {

    #[cold]
    fn reserve_one_unchecked(&mut self) {
        debug_assert_eq!(self.len(), self.capacity());
        let new_cap = self.len()
            .checked_add(1)
            .and_then(usize::checked_next_power_of_two)
            .expect("allocation: capacity overflow");
        infallible(try_grow(self, new_cap));
    }


    /// Reserves the minimum capacity for at least `additional` more elements to be 
    /// inserted in the given BankVec. After calling reserve, capacity will be greater 
    /// than or equal to self.len() + additional. Does nothing if the capacity 
    /// is already sufficient.  If the new capacity would exceed `C` the data is 
    /// moved to the heap.  May allocate more than `additional`.
    /// 
    /// # Panics
    /// 
    /// Panics if the resulting capacity would exceed `usize::MAX`
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<i32, 3>::from([1, 2, 3]);
    /// assert_eq!(bank.capacity(), 3);
    /// bank.reserve(10);
    /// assert!(bank.capacity() >= 10);
    /// ```
    ///     
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        infallible(self.try_reserve(additional));
    }

    /// Reserves the minimum capacity for at least `additional` more elements to be 
    /// inserted in the given BankVec. This will not deliberately over-allocate 
    /// to speculatively avoid frequent allocations. After calling reserve_exact, 
    /// capacity will be greater than or equal to self.len() + additional. Does 
    /// nothing if the capacity is already sufficient.  If the new capacity would
    /// exceed `C` the data is moved to the heap.
    /// 
    /// # Panics
    /// 
    /// Panics if the resulting capacity would exceed `usize::MAX`
    /// 
    /// # Examples
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<i32, 3>::from([1, 2, 3]);
    /// assert_eq!(bank.capacity(), 3);
    /// bank.reserve_exact(10);
    /// assert_eq!(bank.capacity(), 13); // 3 + 10
    /// ```
    ///     
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        infallible(self.try_reserve_exact(additional))
    }

    #[inline]
    fn try_reserve(&mut self, additional: usize) -> Result<(), AllocErr> {
        let (_, &mut len, cap) = self.data_buf_mut();
        match cap - len >= additional {
            true => Ok(()),
            false => len.checked_add(additional)
                .and_then(usize::checked_next_power_of_two)
                .ok_or(AllocErr::Overflow)
                .and_then(|new_cap| try_grow(self, new_cap))
        }
    }

    #[inline]
    fn try_reserve_exact(&mut self, additional: usize) -> Result<(), AllocErr> {
        let (_, &mut len, cap) = self.data_buf_mut();
        match cap - len >= additional {
            true => Ok(()),
            false => len.checked_add(additional)
                .ok_or(AllocErr::Overflow)
                .and_then(|new_cap| try_grow(self, new_cap))
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
    pub const fn len(&self) -> usize {
        match self.on_heap() {
            true => unsafe { self.buf.heap.1 },
            false => self.capacity
        }
    }

    #[inline]
    pub const unsafe fn set_len(&mut self, length: usize) {
        match self.on_heap() {
            true => self.buf.heap.1 = length,
            false => self.capacity = length
        }
    }

    /// Returns true if the bank has exceeded its capacity and moved to the heap,
    /// otherwise false.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankVec;
    /// let bank = BankVec::<i32, 3>::from([0; 2]);
    /// assert!(!bank.on_heap());
    /// 
    /// let bank = BankVec::<i32, 3>::from([0; 5]);
    /// assert!(bank.on_heap());
    /// ```
    #[inline(always)]
    pub const fn on_heap(&self) -> bool { self.capacity > C }


    #[inline(always)]
    unsafe fn heap(&self) -> DataBuf<T> {
        unsafe { (self.buf.heap.0.as_ptr().cast_const(), self.buf.heap.1, self.capacity) }
    }

    #[inline(always)]
    const unsafe fn heap_mut<'a>(&'a mut self) -> DataBufMut<'a,T> {
        unsafe { (self.buf.heap.0, &mut self.buf.heap.1, self.capacity) }
    }

    #[inline(always)]
    unsafe fn stack(&self) -> DataBuf<T> {
        unsafe { (self.buf.stack.as_ptr().cast(), self.capacity, C) }
    }

    #[inline(always)]
    unsafe fn stack_mut<'a>(&'a mut self) -> DataBufMut<'a,T> {
        unsafe { (self.buf.stack_ptr_nn(), &mut self.capacity, C) }
    }

    #[inline]
    fn data_buf(&self) -> DataBuf<T> {
        match self.on_heap() {
            true => unsafe { self.heap() },
            false => unsafe { self.stack() }
        }
    }

    #[inline]
    pub(super) fn data_buf_mut<'a>(&'a mut self) -> DataBufMut<'a,T> {
        match self.on_heap() {
            true => unsafe { self.heap_mut() },
            false => unsafe { self.stack_mut() }
        }
    }


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
    /// 
    #[inline]
    pub const fn new() -> Self {
        assert!(
            mem::size_of::<[T; C]>() == C * mem::size_of::<T>()
                && mem::align_of::<[T; C]>() >= mem::align_of::<T>()
        );

        Self {
            buf: BufferUnion::new_stack(),
            capacity: 0
        }
    }


    /// Returns the number of elements the bank can hold without reallocating.
    /// 
    #[inline]
    pub fn capacity(&self) -> usize {
        if self.on_heap() { self.capacity } else { C }
        //self.data_buf().2 
    }


    /// Appends an element to the back of the collection.
    /// 
    /// If the resulting length would exceed `C`, the bank is moved to the heap.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<i32, 3>::from([1, 2]);
    /// bank.push(3);
    /// assert!(!bank.on_heap()); // Still a fixed size data structure
    /// bank.push(4); /// Length exceeds `C`
    /// assert!(bank.on_heap()); // Now a vec-like heap
    /// assert_eq!(bank, [1, 2, 3, 4]);
    /// ```
    /// 
    /// # Time Complexity
    /// 
    /// Takes *O*(1) time if the new bank length does not exceed, or has already 
    /// exceeded, `C`, otherwise *O*(`C` + 1) time is needed to move the data 
    /// into a heap.
    ///     
    #[inline]
    pub fn push(&mut self, value: T) {
        let (mut ptr, mut len, cap) = self.data_buf_mut();
        if *len == cap {
            self.reserve_one_unchecked();
            ptr = unsafe { self.buf.heap.0 };
            len = unsafe { &mut self.buf.heap.1 };
        }
        unsafe { ptr.add(*len).write(value) };
        *len += 1;
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
    /// data is moved to the heap.
    /// 
    pub fn insert(&mut self, index: usize, element: T) {
        // Most of this procedure for insert was copied from the SmallVec crate.
        // I really don't understand why but, it compiles down to slightly faster
        // machine code.
        let (mut ptr, mut len, cap) = self.data_buf_mut();
        if *len == cap {
            self.reserve_one_unchecked();
            ptr = unsafe { self.buf.heap.0 };
            len = unsafe { &mut self.buf.heap.1 };
        }
        let mut ptr = ptr.as_ptr();
        let cp_len = *len;

        if index > cp_len { panic!("index out of bounds"); }

        ptr = unsafe { ptr.add(index) };
        if index < cp_len {
            unsafe { ptr.copy_to(ptr.add(1), cp_len - index) }
        }
        *len = cp_len + 1;
        unsafe { ptr.write(element) };

    }

    /// Removes the last element of the bank and returns it, or None if it is empty.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bankarr::BankVec;
    /// 
    /// let mut bank = BankVec::<i32, 3>::from([1, 2, 3, 4]);
    /// assert_eq!(bank.pop(), Some(4));
    /// 
    /// ```
    /// 
    /// # Time Complexity
    /// 
    /// Takes *O*(1) time but this time will be faster or slower depending on 
    /// whether the capacity has exceeded `C`.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        let (ptr, len, _) = self.data_buf_mut();
        if *len == 0 { return None }
        *len -= 1;
        Some(unsafe { ptr.add(*len).read() })
    }

    /// Removes and returns the element at position `index` within the bank, 
    /// shifting all elements after it to the left.
    /// 
    /// This function has, at worst, *O*(n) performance. If you don't need to
    /// preserve the order of elements, use [`swap_remove`](BankVec::swap_remove)
    /// instead.
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
    /// 
    pub fn remove(&mut self, index: usize) -> T {
        let (ptr, len, _) = self.data_buf_mut();
        assert!(index < *len, "index out of bounds");
        *len -= 1;
        let ptr = unsafe { ptr.as_ptr().add(index) };
        let removed = unsafe { ptr.read() };
        unsafe { ptr.copy_from(ptr.add(1), *len - index) }
        removed
    }

    /// Removes an element from the bank and returns it.
    /// 
    /// The removed element is replaced by the last element in the bank.  This
    /// doesnt preserve ordering of the remaining elements but **is** *O*(1).
    /// If you need to preserve ordering, use [`remove`](BankVec::remove).
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
    ///     
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        let (ptr, len, _) = self.data_buf_mut();
        assert!(index < *len, "index out of bounds");
        *len -= 1;
        // Storing and reusing ptr.add(*len) doesn't improve performance
        unsafe { ptr.add(index).swap(ptr.add(*len)); };
        unsafe { ptr.add(*len).read() }
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
    /// 
    #[inline]
    pub fn as_slice(&self) -> &[T] { 
        let (ptr, len, _) = self.data_buf();
        unsafe { slice::from_raw_parts(ptr, len) }
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
        let (ptr, &mut len, _) = self.data_buf_mut();
        unsafe { slice::from_raw_parts_mut(ptr.as_ptr(), len)}
    }

    pub fn drain<R>(&mut self, range: R) -> drain::Drain<'_, T, Self> 
    where 
        R: ops::RangeBounds<usize>,
    {
        // This implementation was pulled from `Vec::drain`

        let (ptr, len, _) = self.data_buf_mut();

        let ptr = ptr.as_ptr();
        let cp_len = *len;
        let ops::Range { start, end } = drain::slice_range(range, ..cp_len);

        unsafe {
            *len = start;
            drain::Drain {
                tail_start: end,
                tail_len: cp_len - end,
                iter: slice::from_raw_parts(ptr.add(start), end - start).iter(),
                bank: NonNull::new_unchecked(self)
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::array;
    use super::*;

    type B = BankVec<u32, 3>;

    #[test]
    fn from_vec() {
        let bank = BankVec::<i32, 4>::from(vec![1, 2, 3, 4]);
        assert_eq!(bank, [1, 2, 3, 4]);

        let bank = BankVec::<i32, 4>::from(vec![1, 2, 3, 4, 5]);
        assert_eq!(bank, [1, 2, 3, 4, 5]);

    }

    #[test]
    fn from_arr() {
        let bank = BankVec::<i32, 4>::from([1, 2, 3, 4]);
        assert_eq!(bank, [1, 2, 3, 4]);

        let bank = BankVec::<i32, 4>::from([1, 2, 3, 4, 5]);
        assert_eq!(bank, [1, 2, 3, 4, 5]);
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
        assert!(!bank.on_heap());
        
        assert_eq!(bank[..1], [1]);
        assert_eq!(bank, [1, 2, 3]);
        
        bank.push(4);
        assert!(bank.on_heap());
        bank.push(5);
        assert_eq!(bank, [1, 2, 3, 4, 5]);
    }

    #[test]
    #[should_panic]
    fn insert_out_of_bounds() {
        let mut bank = B::from([3, 4, 5]);

        bank.insert(4, 0);
    }

    #[test]
    fn insert() {
        let mut bank = BankVec::<i32, 4>::from([1, 2, 4]);
        bank.insert(2, 3);
        assert_eq!(bank, [1, 2, 3, 4]);

        let mut bank = BankVec::<i32, 3>::from([1, 2, 4, 5]);
        bank.insert(2, 3);
        assert_eq!(bank, [1, 2, 3, 4, 5]);

    }

    #[test]
    fn pop() {
        let mut bank = B::from([3, 4, 5, 6]);
        
        assert!(bank.on_heap());
        assert_eq!(bank.pop(), Some(6));

        //assert!(!bank.on_heap());
        //assert_eq!(bank.pop(), Some(5))
    }

    #[test]
    fn remove() {
        let mut bank = B::from([3, 4, 5, 6]);

        assert!(bank.on_heap());
        let removed = bank.remove(1);
        assert_eq!(removed, 4);
        assert_eq!(bank, [3, 5, 6]);

        //assert!(!bank.on_heap());
        //let removed = bank.remove(1);
        //assert_eq!(removed, 5);
        //assert_eq!(bank, [3, 6]);
    }

    #[test]
    fn swap_remove() {
        let mut bank = BankVec::<String, 3>::from(["aa".to_string(), "bb".to_string(), "cc".to_string(), "dd".to_string()]);
        
        assert!(bank.on_heap());
        let removed = bank.swap_remove(0);
        assert_eq!(removed, "aa".to_string());

        let removed = bank.swap_remove(0);
        assert_eq!(removed, "dd".to_string());

        //assert!(!bank.on_heap());
        //let removed = bank.swap_remove(1);
        //assert_eq!(removed, "bb".to_string());

        //assert_eq!(bank, ["dd".to_string(), "cc".to_string()])
    }

    #[test]
    fn reserve_exact() {
        let mut bank = B::from([3, 4, 5]);
        assert_eq!(bank.capacity(), 3);
        bank.reserve_exact(1);
        assert_eq!(bank.capacity(), 4);
        bank.push(4);
        bank.reserve_exact(1);
        assert_eq!(bank.capacity(), 5);
    }

    #[test]
    fn extend() {
        let mut bank = BankVec::<i32, 4>::new();
        let arr: [i32; 8] = array::from_fn(|idx| idx as i32);
        bank.extend(arr.clone());

        assert_eq!(bank, arr);
    }

    #[test]
    fn iter() {
        let mut bank = BankVec::<&'static str, 3>::from(["a", "b", "c"]);
        assert!(!bank.on_heap());
        let mut iter = bank.iter();
        for s in ["a", "b", "c"] {
            assert_eq!(iter.next(), Some(s).as_ref());
        }
        assert_eq!(iter.next(), None);


        bank.push("d");
        assert!(bank.on_heap());
        let mut iter = bank.iter();
        for s in ["a", "b", "c", "d"] {
            assert_eq!(iter.next(), Some(s).as_ref());
        }
        assert_eq!(iter.next(), None);

        let mut bank = BankVec::<i32, 3>::from([1, 2, 3]);
        let r = &mut bank;
        for v in r { *v *= 2 }
        let r = &bank;
        let out = r.into_iter().map(|v| *v).collect::<Vec<_>>();
        assert_eq!(out, [2, 4, 6]);
    }

    #[test]
    fn iter_mut() {
        let mut bank = BankVec::<&'static str, 3>::from(["a", "b", "c"]);
        assert!(!bank.on_heap());
        let mut iter = bank.iter_mut();
        for s in ["a", "b", "c"] {
            assert_eq!(iter.next(), Some(s).as_mut());
        }
        assert_eq!(iter.next(), None);


        bank.push("d");
        assert!(bank.on_heap());
        let mut iter = bank.iter_mut();
        for s in ["a", "b", "c", "d"] {
            assert_eq!(iter.next(), Some(s).as_mut());
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn as_slice() {
        let mut bank = B::from([3, 4, 5]);
        assert!(!bank.on_heap());
        assert_eq!(bank.as_slice(), [3, 4, 5]);

        bank.push(6);
        assert!(bank.on_heap());
        assert_eq!(bank.as_slice(), [3, 4, 5, 6]);
    }

    #[test]
    fn as_slice_mut() {
        let mut bank = B::from([3, 4, 5]);
        assert!(!bank.on_heap());
        assert_eq!(bank.as_slice(), [3, 4, 5]);

        bank.push(6);
        assert!(bank.on_heap());
        assert_eq!(bank.as_mut_slice(), [3, 4, 5, 6]);
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
    fn drain() {
        let arr: [i32; 8] = array::from_fn(|idx| idx as i32);
        let mut bank = BankVec::<i32, 4>::from(arr.clone());

        let drained: Vec<i32> = bank.drain(..).collect();

        assert_eq!(arr, *drained);
        assert_eq!(bank.len(), 0);
        assert_eq!(bank, []);
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

    #[test]
    fn try_reserve() {
        let mut bank = BankVec::<i32, 3>::new();
        
        assert!(bank.try_reserve(1).is_ok());
        assert!(bank.try_reserve(4).is_ok());
    }

    #[test]
    fn try_reserve_exact() {
        let mut bank = BankVec::<i32, 3>::new();
        
        assert!(bank.try_reserve_exact(1).is_ok());
        assert!(bank.try_reserve_exact(4).is_ok());
    }

    #[test]
    fn set_len() {
        let mut bank = BankVec::<i32, 3>::from([1, 2, 3]);

        // This technically leaks memory but here it doesn't matter.
        unsafe { bank.set_len(1) };
        assert_eq!(bank.len(), 1);
        assert_eq!(bank, [1]);

        // Now again from the heap form
        let mut bank = BankVec::<i32, 3>::from([1, 2, 3, 4]);
        unsafe { bank.set_len(1) };
        assert_eq!(bank.len(), 1);
        assert_eq!(bank, [1]);

    }
}