use std::{alloc::{self, Layout}, mem, ptr::{self, NonNull}, alloc::{alloc, realloc}};

use crate::errors::AllocErr;
use super::{
    BankVec,
    BufferUnion,
};

#[inline]
pub(super) fn infallible<T>(result: Result<T, AllocErr>) -> T {
    match result {
        Ok(x) => x,
        Err(AllocErr::Layout) => panic!("invalid parameters to Layout::from_size_align"),
        Err(AllocErr::Overflow) => panic!("capacity overflow"),
        Err(AllocErr::Alloc { layout }) => alloc::handle_alloc_error(layout),
    }
}

#[inline]
pub(super) fn layout_array<T>(n: usize) -> Result<Layout, AllocErr> {

    let size = mem::size_of::<T>()
        .checked_mul(n)
        .ok_or(AllocErr::Overflow)?;

    let align = mem::align_of::<T>();

    Layout::from_size_align(size, align).map_err(|_| AllocErr::Overflow)
}

pub(super) unsafe fn deallocate<T>(ptr: NonNull<T>, cap: usize) {
    let layout = Layout::array::<T>(cap).unwrap();
    unsafe { alloc::dealloc(ptr.as_ptr() as *mut u8, layout) };
}