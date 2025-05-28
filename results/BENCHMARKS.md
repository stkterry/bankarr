# Benchmarks

## Table of Contents

- [Benchmark Results](#benchmark-results)
    - [Bank Perf](#bank-perf)

## Benchmark Results

### Bank Perf

|              | `Bank`                  | `Vec`                          | `SmallVec`                     | `ArrayVec`                      |
|:-------------|:------------------------|:-------------------------------|:-------------------------------|:------------------------------- |
| **`push`**   | `1.73 ns` (✅ **1.00x**) | `1.95 ns` (❌ *1.13x slower*)   | `1.87 ns` (✅ **1.08x slower**) | `1.68 ns` (✅ **1.03x faster**)  |
| **`pop`**    | `0.52 ns` (✅ **1.00x**) | `0.52 ns` (✅ **1.01x slower**) | `0.70 ns` (❌ *1.34x slower*)   | `0.50 ns` (✅ **1.03x faster**)  |
| **`remove`** | `3.11 ns` (✅ **1.00x**) | `3.43 ns` (✅ **1.10x slower**) | `3.41 ns` (✅ **1.10x slower**) | `3.40 ns` (✅ **1.09x slower**)  |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

