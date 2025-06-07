use std::{mem::{ManuallyDrop, MaybeUninit}, ptr::NonNull};



pub(super) type DataBuf<T> = (*const T, usize, usize);
pub(super) type DataBufMut<'a, T> = (NonNull<T>, &'a mut usize, usize);

pub(super) union BufferUnion<T, const C: usize> {
    pub(super) stack: ManuallyDrop<MaybeUninit<[T; C]>>,
    pub(super) heap: (NonNull<T>, usize),
}

unsafe impl<T: Send, const C: usize> Send for BufferUnion<T, C> {}
unsafe impl<T: Sync, const C: usize> Sync for BufferUnion<T, C> {}

impl<T, const C: usize> BufferUnion<T, C> {

    #[inline]
    pub(super) const fn new_stack() -> Self { 
        Self { stack: ManuallyDrop::new(MaybeUninit::uninit()) }
    }
    
    #[inline]
    pub(super) const fn new_heap(ptr: NonNull<T>, len: usize) -> Self {
        Self { heap: (ptr, len) }
    }

    #[inline]
    pub(super) unsafe fn stack_ptr_non_null(&mut self) -> NonNull<T> {
        unsafe {
            NonNull::new(self.stack.as_mut_ptr() as *mut T).unwrap_unchecked()
        }
    }
}

