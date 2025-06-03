use std::{hint::black_box};

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

use bankarr::{BankArr, BankVec};



pub fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bank Comparision");
    group.sample_size(1000);

    group.bench_function(
        BenchmarkId::new("BankVec", "push-arr"), 
        |b| b.iter_batched_ref(
            || BankVec::<u8, 16>::new(),
            |bank| black_box({ bank.push(black_box(128)); }), 
            BatchSize::SmallInput
        )
    );
    
    group.bench_function(
        BenchmarkId::new("BankVec", "push-overflow"),
        |b| b.iter_batched_ref(
            || BankVec::<u8, 16>::from([1; 16]),
            |bank| black_box({ bank.push(black_box(128)); }), 
            BatchSize::SmallInput
        )
    );

    group.bench_function(
        BenchmarkId::new("BankVec", "push-vec"),
        |b| b.iter_batched_ref(
            || {
                let mut bank = BankVec::<u8, 16>::from([1; 17]);
                bank.reserve_exact(1);
                bank
            },
            |bank| black_box({ bank.push(black_box(128)); }), 
            BatchSize::SmallInput
        )
    );

    group.bench_function(
        BenchmarkId::new("BankArr", "push"), 
        |b| b.iter_batched_ref(
            || BankArr::<u8, 16>::new(),
            |bank| black_box({ bank.push(black_box(128)); }), 
            BatchSize::SmallInput
        )
    );

    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);