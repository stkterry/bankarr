[<picture><img src="https://badges.ws/crates/v/bankarr?color=f74d02&logo=rust" /></picture>](https://crates.io/crates/bankarr)
[<picture><img src="https://badges.ws/crates/docs/bankarr" /></picture>](https://docs.rs/bankarr/latest/bankarr/)
[<img src="https://badges.ws/maintenance/yes/2025" />](https://github.com/stkterry/bankarr)
[<img src="https://badges.ws/github/license/stkterry/bankarr" />](https://github.com/stkterry/bankarr/blob/main/LICENSE.md)
[![Coverage Status](https://coveralls.io/repos/github/stkterry/bankarr/badge.svg?branch=main)](https://coveralls.io/github/stkterry/bankarr?branch=main)
[![Miri](https://github.com/stkterry/bankarr/actions/workflows/miri.yaml/badge.svg)](https://github.com/stkterry/bankarr/actions/workflows/miri.yaml)

# Bankarr [<img src="https://badges.ws/badge/Rust-000000?logo=rust" />](https://www.rust-lang.org)

A pair of array-like vectors allowing storage on the stack.  `BankArr` is fixed-size
with a capacity that may not be exceeded, whereas `BankVec` can exceed its capacity,
moving to the heap.

This crate is very similar to both [arrayvec](https://crates.io/crates/arrayvec) and
[smallvec](https://crates.io/crates/smallvec).  Both are great and currently cover
more ground than this project.

Initial benchmarks suggest that this project is, in a few spots, marginally faster,
at least for my specific use-case and the machines tested on.  You can find the 
current benchmark results in the repository. All benchmarks are compared against
Rust's `Vec`.  Specifically you should be looking at `BankArr` vs `ArrayVec` and 
`BankVec` vs `SmallVec`, etc.


## Installation
Add the following to your Cargo.toml file:
```rust
[dependencies]
bankarr = "0.8.0"
```

## Examples 
```rust
use bankarr::{BankArr, BankVec};

let mut bank = BankArr::<i32, 5>::from([1, 2]);
bank.push(3);
assert_eq!(bank, [1, 2, 3]);
assert_eq!(bank.pop(), Some(3));

bank.extend([3, 4, 5]);

let removed = bank.swap_remove(0);
assert_eq!(removed, 1);
assert_eq!(bank, [5, 2, 3, 4]);

// BankVec has most of the same features but can exceed its capacity
let mut bank = BankVec::<i32, 5>::from([1, 2, 3, 4]);
assert!(!bank.on_heap());
bank.extend([5, 6, 7, 8]);
assert!(bank.on_heap());

assert_eq!(bank, [1, 2, 3, 4, 5, 6, 7, 8]);

```

Checkout the [docs](https://docs.rs/bankarr/latest/bankarr/) for more comprehensive examples.

## Real-World Peformance Benefits?

In most circumstances a stack based vector is unlikely to yield meaningful performance
gains.  There are some beneficial use-cases, however.  For instance, you can avoid 
pointer indirection in circumstances where you need a `Vec<Vec<T>>`, and can instead 
use `Vec<BankVec<T>>` or even better `Vec<BankArr<T>>` when you're sure you won't 
exceed capacity. In general you can get better cache-locality embedding either bank
into a struct that would otherwise store a vec, but that depends on how it's used.

It's important to keep in mind that `BankVec` is more of an escape hatch than anything,
as moving contents to the heap when its capacity is exceeded has significant performance
implications. Test your performance, etc.

#### Note
Again be sure to check out [arrayvec](https://crates.io/crates/arrayvec) and
[smallvec](https://crates.io/crates/smallvec) if this crate doesn't have what you're
looking for.  A lot of the code herein was inspired by or frankly pulled from 
those crates.  I've merely made a few bits faster for my own use-cases.
