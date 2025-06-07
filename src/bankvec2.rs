

use core::slice;
use std::{alloc::{alloc, realloc, Layout, LayoutError}, mem::{self, ManuallyDrop, MaybeUninit}, ops::{Deref, DerefMut, Index, IndexMut}, ptr::{self, NonNull}, slice::SliceIndex};

mod allocation;
mod buffer_union;
use allocation::*;

use crate::errors::AllocErr;
use buffer_union::*;


pub struct BankVec<T, const C: usize> {
    buf: BufferUnion<T, C>,
    capacity: usize,
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
        debug_assert_eq!(self.len(), self.capacity);
        let new_cap = self.len()
            .checked_add(1)
            .and_then(usize::checked_next_power_of_two)
            .expect(&AllocErr::Overflow.to_string());
        infallible(self.try_grow(new_cap));
    }

    fn try_grow(&mut self, new_cap: usize) -> Result<(), AllocErr> {

        use ptr::copy_nonoverlapping as cp;

        let (ptr, &mut len, cap) = self.data_buf_mut();
        assert!(new_cap >= len);

        if new_cap <= C {
            if !self.on_heap() { return Ok(()) }
            
            self.buf = BufferUnion::new_stack();
            unsafe { cp(ptr.as_ptr(), self.buf.stack_ptr_non_null().as_ptr(), len) }
            self.capacity = new_cap;
            unsafe { deallocate(ptr, cap) };
        } else if new_cap != cap {
            let layout = Layout::array::<T>(new_cap).map_err(AllocErr::layout)?;
            debug_assert!(layout.size() > 0);

            let ptr = if !self.on_heap() {
                let dst = NonNull::new(unsafe { alloc(layout) })
                    .ok_or(AllocErr::alloc(layout))?.cast();
                unsafe { cp(ptr.as_ptr(), dst.as_ptr(), len) };
                
                dst
            } else {
                let prev_layout = Layout::array::<T>(cap).map_err(AllocErr::layout)?;
                let ptr = unsafe { realloc(ptr.as_ptr().cast(), prev_layout, layout.size()) };

                NonNull::new(ptr)
                    .ok_or(AllocErr::alloc(layout))?
                    .cast()
            };

            self.buf = BufferUnion::new_heap(ptr, len);
            self.capacity = new_cap;
        }

        Ok(())
    }
    
    #[inline]
    const fn len(&self) -> usize {
        match self.on_heap() {
            true => unsafe { self.buf.heap.1 },
            false => self.capacity
        }
    }

    #[inline(always)]
    const fn on_heap(&self) -> bool { self.capacity > C }

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
        unsafe { (self.buf.stack_ptr_non_null(), &mut self.capacity, C) }
    }

    #[inline]
    fn data_buf(&self) -> DataBuf<T> {
        match self.on_heap() {
            true => unsafe { self.heap() },
            false => unsafe { self.stack() }
        }
    }

    #[inline]
    fn data_buf_mut<'a>(&'a mut self) -> DataBufMut<'a,T> {
        match self.on_heap() {
            true => unsafe { self.heap_mut() },
            false => unsafe { self.stack_mut() }
        }
    }

    #[inline]
    pub const fn new() -> Self {

        assert!(
            mem::size_of::<[T; C]>() == C * mem::size_of::<T>()
                && mem::align_of::<[T; C]>() >= mem::align_of::<T>()
        );

        Self {
            capacity: 0,
            buf: BufferUnion { 
                stack: ManuallyDrop::new(MaybeUninit::uninit())
            },
        }
    }

    pub fn push(&mut self, value: T) {
        let (mut ptr, mut len, cap) = self.data_buf_mut();
        if *len == cap {
            self.reserve_one_unchecked();
            ptr = unsafe { self.buf.heap.0 };
            len = unsafe { &mut self.buf.heap.1 };
        }
        unsafe { ptr::write(ptr.as_ptr().add(*len), value) };
        //unsafe { ptr.add(*len).write(value) };
        *len += 1;
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] { 
        let (ptr, len, _) = self.data_buf();
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] { 
        let (ptr, &mut len, _) = self.data_buf_mut();
        unsafe { slice::from_raw_parts_mut(ptr.as_ptr(), len)}
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        let mut bank = BankVec::<i32, 4>::new();

        bank.push(1);
        bank.push(2);
        bank.push(3);
        bank.push(4);
        bank.push(5);

        assert_eq!(bank[..], [1, 2, 3, 4, 5]);
    }
}