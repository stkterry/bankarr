use std::{alloc::{self, Layout}, ptr::NonNull, alloc::{alloc, realloc}};

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
pub(super) unsafe fn deallocate<T>(ptr: NonNull<T>, cap: usize) {
    let layout = Layout::array::<T>(cap).unwrap();
    unsafe { alloc::dealloc(ptr.as_ptr() as *mut u8, layout) };
}

#[inline(always)]
pub(super) fn try_grow<T, const C: usize>(bank: &mut BankVec<T, C>, new_cap: usize) -> Result<(), AllocErr> {

    let (src, &mut len, cap) = bank.data_buf_mut();
    assert!(new_cap >= len);

    if new_cap <= C {
        if !bank.on_heap() { return Ok(()) }

        bank.buf = BufferUnion::new_stack();
        unsafe { src.copy_to_nonoverlapping(bank.buf.stack_ptr_nn(), len) }
        bank.capacity = new_cap;
        unsafe { deallocate(src, cap) };
    } else if new_cap != cap {
        let layout = Layout::array::<T>(new_cap).map_err(AllocErr::layout)?;
        debug_assert!(layout.size() > 0);

        let ptr = if !bank.on_heap() {
            let dst = NonNull::new(unsafe { alloc(layout) })
                .ok_or(AllocErr::alloc(layout))?.cast();
            unsafe { src.copy_to_nonoverlapping(dst, len) };
            
            dst
        } else {
            let prev_layout = Layout::array::<T>(cap).map_err(AllocErr::layout)?;
            let ptr = unsafe { realloc(src.as_ptr().cast(), prev_layout, layout.size()) };

            NonNull::new(ptr).ok_or(AllocErr::alloc(layout))?.cast()
        };

        bank.buf = BufferUnion::heap_from(ptr, len);
        bank.capacity = new_cap;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::panic;
    use super::*;

    #[test]
    fn infallible_() {
        let results: [Result<i32, AllocErr>; 3] = [
            Ok(3),
            Err(AllocErr::Layout),
            Err(AllocErr::Overflow),
            //Err(AllocErr::Alloc { layout: Layout::array::<i32>(4).unwrap() })
        ];

        let fallibles = results
            .into_iter()
            .map(|err| panic::catch_unwind(|| infallible(err) ))
            .map(|err| err.is_ok())
            .collect::<Vec<_>>();

        assert_eq!(fallibles, [true, false, false]);

        // Cant seem to properly capture the alloc error, it panics with 
        // or without #[should_panic]
        
    }

    #[test]
    fn deallocate_() {
        let mut vec = Vec::from([1i32, 2, 3]);
        let (ptr, cap, _) = (vec.as_mut_ptr(), vec.capacity(), vec.len());
        std::mem::forget(vec);
        let ptr = NonNull::new(ptr).expect("this should certainly work");

        unsafe { deallocate(ptr, cap) };
    }

    #[test]
    fn try_grow_() {
        
        let mut bank = BankVec::<i32, 4>::new();
        try_grow(&mut bank, 3).unwrap();
        assert_eq!(bank.capacity, 0);

        
        let mut bank = BankVec::<i32, 4>::from([1, 2, 3, 4, 5]);
        // capacity is the nearest power of 2 that is greater than the 
        // initial size of elements (5 in this case);
        assert!(bank.on_heap());
        assert_eq!(bank.capacity, 8); 

        bank.drain(3..); // drop len to less than the new capacity we want to reduce to.

        try_grow(&mut bank, 3).unwrap();
        assert_eq!(bank.capacity, 3);

    }

}