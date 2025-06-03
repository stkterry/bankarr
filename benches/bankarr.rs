use std::{hint::black_box};

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

use bankarr::BankArr;
use smallvec::SmallVec;
use arrayvec::ArrayVec;


pub fn benchmark(c: &mut Criterion) {

    let mut group = c.benchmark_group("Bank Perf");
    group.sample_size(1000);
    group.bench_function(
        BenchmarkId::new("Bank", "push"),
        |b| b.iter_batched_ref(
            || BankArr::<u8, 16>::new(), 
            |bank| { black_box({ bank.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        BenchmarkId::new("Vec", "push"),
        |b| b.iter_batched_ref(
            || Vec::<u8>::with_capacity(16), 
            |vec| { black_box({ vec.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        BenchmarkId::new("SmallVec", "push"),
        |b| b.iter_batched_ref(
            || SmallVec::<[u8; 16]>::new(), 
            |vec| { black_box({ vec.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        BenchmarkId::new("ArrayVec", "push"),
        |b| b.iter_batched_ref(
            || ArrayVec::<u8, 16>::new(), 
            |vec| { black_box({ vec.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );


    group.bench_function(
        BenchmarkId::new("Bank", "pop"),
        |b| b.iter_batched_ref(
            || { BankArr::<u8, 16>::from([0, 1, 2, 3]) }, 
            |bank| black_box({let _ = bank.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        BenchmarkId::new("Vec", "pop"),
        |b| b.iter_batched_ref(
            || { let mut vec: Vec<u8> = vec![0, 1, 2, 3]; vec.reserve_exact(12); vec }, 
            |vec| black_box({ let _ = vec.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        BenchmarkId::new("SmallVec", "pop"),
        |b| b.iter_batched_ref(
            || SmallVec::<[u8; 16]>::from_vec(vec![0, 1, 2, 3]), 
            |vec| black_box({ let _ = vec.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        BenchmarkId::new("ArrayVec", "pop"),
        |b| b.iter_batched_ref(
            || { let mut vec = ArrayVec::<u8, 16>::new(); (0..4).for_each(|v| vec.push(v)); vec}, 
            |vec| black_box({ let _ = vec.pop(); }),
            BatchSize::SmallInput
        )
    );


    group.bench_function(
        BenchmarkId::new("Bank", "remove"),
        |b| b.iter_batched_ref(
            || { BankArr::<u8, 16>::from([0, 1, 2, 3]) }, 
            |bank| black_box({let _ = bank.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        BenchmarkId::new("Vec", "remove"),
        |b| b.iter_batched_ref(
            || { let mut vec: Vec<u8> = vec![0, 1, 2, 3]; vec.reserve_exact(12); vec }, 
            |vec| black_box({ let _ = vec.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        BenchmarkId::new("SmallVec", "remove"),
        |b| b.iter_batched_ref(
            || SmallVec::<[u8; 16]>::from_vec(vec![0, 1, 2, 3]), 
            |vec| black_box({ let _ = vec.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        BenchmarkId::new("ArrayVec", "remove"),
        |b| b.iter_batched_ref(
            || { let mut vec = ArrayVec::<u8, 16>::new(); (0..4).for_each(|v| vec.push(v)); vec }, 
            |vec| black_box({ let _ = vec.remove(1); }),
            BatchSize::SmallInput
        )
    );


    group.bench_function(
        BenchmarkId::new("Bank", "slice"),
        |b| b.iter_batched_ref(
            || BankArr::<u32, 16>::from(black_box([32; 8])), 
            |bank| black_box({
                let wut = &bank[..];
                wut[0];
            }), 
            BatchSize::SmallInput
        )
    );

    group.bench_function(
        "iter",
        |b| b.iter_batched_ref(
            || { BankArr::<u32, 16>::from(black_box([32; 8]))}, 
            |bank| black_box(for v in bank.iter() { black_box(v); }), 
            BatchSize::SmallInput
        )
    );

    group.finish();

}

criterion_group!(benches, benchmark);
criterion_main!(benches);