# Benchmarks

## Table of Contents

- [Benchmark Results](#benchmark-results)
    - [Bank Perf](#bank-perf)

## Benchmark Results

### Bank Perf

|              | `Bank`                  | `Vec`                          | `SmallVec`                     | `ArrayVec`                      |
|:-------------|:------------------------|:-------------------------------|:-------------------------------|:------------------------------- |
| **`push`**   | `1.73 ns` (✅ **1.00x**) | `1.96 ns` (❌ *1.13x slower*)   | `1.87 ns` (✅ **1.08x slower**) | `1.68 ns` (✅ **1.03x faster**)  |
| **`pop`**    | `0.62 ns` (✅ **1.00x**) | `0.53 ns` (✅ **1.18x faster**) | `0.71 ns` (❌ *1.14x slower*)   | `0.63 ns` (✅ **1.01x slower**)  |
| **`remove`** | `3.16 ns` (✅ **1.00x**) | `3.49 ns` (✅ **1.10x slower**) | `3.77 ns` (❌ *1.19x slower*)   | `3.41 ns` (✅ **1.08x slower**)  |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

