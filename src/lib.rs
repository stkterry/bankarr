//! 
//! Fixed-size arrays structs with vec-like semantics.
//! 
//! [`BankArr<T, C>`] is a fixed-size, array struct, storing items on the stack up to `C`.
//! [`BankVec<T, C>`] contains a fixed-size array struct as well, but can exceed `C`, switching to
//! a heap allocated [`Vec`] when doing so. 
//! 
//! 
//! # Performance
//! 
//! `BankArr` has effectively equivalent performance of a fixed-size array, `[T; C]`, whereas `BankVec`
//! is slightly slower, but still faster than a heap allocation while its length is below `C`. Once `BankVec`
//! exceeds its array capacity, you can expect equivalent performance to a `Vec`.
//! 
//! # Time Complexity
//! 
//! You should prefer `BankArr` when you have an upper limit you know you wont exceed, 
//! and only considering `BankVec` when your data may occasionally spill over.  
//! 
//! In general `BankVec` will be *almost* as fast as `BankArr`, but has performance overhead for 
//! managing its variants but especially when tranforming into a heap allocation. Spilling over `C` requires
//! *O*(`C`) time complexity to move over to the heap, and the same cost is incurred again should the
//! length again fall bellow `C`.
//! 
//! # Similar Crates
//! 
//! 

mod bankarray;
mod bankvec;
pub(crate)mod errors;


pub use bankarray::BankArr;
pub use bankvec::BankVec;
