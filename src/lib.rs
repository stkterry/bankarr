//! 
//! Fixed-size arrays structs with vec-like semantics.
//! 
//! [`BankArr<T, C>`] is a fixed-size, array struct, storing items on the stack up to `C`.
//! 
//! [`BankVec<T, C>`] is a fixed-size as well, but can exceed `C`, reallocating onto the
//! heap when doing so.
//! 
//! 
//! # Performance
//! 
//! `BankArr` is about as fast as a stack-allocated array.  `BankVec` is generally
//! faster than vec as well, but only while its capacity is equal or less than its 
//! generic `C`.  Once `BankVec` has reallocated to the heap its generally on par
//! with a Vec, but in some cases slower.
//! 
//! # Time Complexity
//! 
//! You should prefer `BankArr` when you have an upper limit you know you wont exceed, 
//! and only considering `BankVec` when your data may occasionally spill over.  
//! 
//! In general `BankVec` will be *almost* as fast as `BankArr`, but has performance overhead for 
//! managing its variants but especially when tranforming into a heap allocation. Spilling over `C` requires
//! *O*(`C`) time complexity to move over to the heap.
//! 
//! # Similar Crates
//! 
//! This crate was inspired heavily from a few existing crate with similar intent,
//! namely [`SmallVec`](<https://crates.io/crates/smallvec>) and
//! [`ArrayVec`](<https://crates.io/crates/arrayvec>).
//! 
//! Comparing `BankArr` with `ArrayVec` and `BankVec` with `SmallVec`, performance
//! is generally equivalent, but in some cases this crate is favored.
//! 
//! 

mod bankarray;
mod bankvec;
mod drain;
pub(crate)mod errors;


pub use bankarray::BankArr;
pub use bankvec::BankVec;
