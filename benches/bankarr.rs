use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

use bankarr::{BankArr, BankVec};
use smallvec::SmallVec;
use arrayvec::ArrayVec;


pub fn benchmark(c: &mut Criterion) {

    let mut group = c.benchmark_group("push");
    group.sample_size(2000);
    group.bench_function(
        "BankArr",
        |b| b.iter_batched_ref(
            || BankArr::<u8, 16>::new(), 
            |bank| { black_box({ bank.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec - stack",
        |b| b.iter_batched_ref(
            || BankVec::<u8, 16>::new(), 
            |bank| { black_box({ bank.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec - heap",
        |b| b.iter_batched_ref(
            || { 
                let mut bank = BankVec::<u8, 14>::from([128; 15]);
                bank.reserve_exact(1);
                bank
            }, 
            |bank| { black_box({ bank.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "Vec",
        |b| b.iter_batched_ref(
            || Vec::<u8>::with_capacity(16), 
            |vec| { black_box({ vec.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "SmallVec",
        |b| b.iter_batched_ref(
            || SmallVec::<[u8; 16]>::new(), 
            |vec| { black_box({ vec.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "ArrayVec",
        |b| b.iter_batched_ref(
            || ArrayVec::<u8, 16>::new(), 
            |vec| { black_box({ vec.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.finish();


    let mut group = c.benchmark_group("pop");
    group.bench_function(
        "BankArr",
        |b| b.iter_batched_ref(
            || BankArr::<u8, 16>::from([0, 1, 2, 3]), 
            |bank| black_box({let _ = bank.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec - stack",
        |b| b.iter_batched_ref(
            || BankVec::<u8, 16>::from([0, 1, 2, 3]), 
            |bank| black_box({let _ = bank.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec - heap",
        |b| b.iter_batched_ref(
            || BankVec::<u8, 14>::from([128; 16]), 
            |bank| black_box({let _ = bank.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "Vec",
        |b| b.iter_batched_ref(
            || { let mut vec: Vec<u8> = vec![0, 1, 2, 3]; vec.reserve_exact(12); vec }, 
            |vec| black_box({ let _ = vec.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "SmallVec",
        |b| b.iter_batched_ref(
            || SmallVec::<[u8; 16]>::from_vec(vec![0, 1, 2, 3]), 
            |vec| black_box({ let _ = vec.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "ArrayVec",
        |b| b.iter_batched_ref(
            || { let mut vec = ArrayVec::<u8, 16>::new(); (0..4).for_each(|v| vec.push(v)); vec}, 
            |vec| black_box({ let _ = vec.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.finish();

    let mut group = c.benchmark_group("remove");
    group.bench_function(
        "BankArr",
        |b| b.iter_batched_ref(
            || { BankArr::<u8, 16>::from([0, 1, 2, 3]) }, 
            |bank| black_box({let _ = bank.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec - stack",
        |b| b.iter_batched_ref(
            || { BankVec::<u8, 16>::from([0, 1, 2, 3]) }, 
            |bank| black_box({let _ = bank.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec - heap",
        |b| b.iter_batched_ref(
            || BankVec::<u8, 14>::from([128; 16]), 
            |bank| black_box({let _ = bank.remove(14); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "Vec",
        |b| b.iter_batched_ref(
            || { let mut vec: Vec<u8> = vec![0, 1, 2, 3]; vec.reserve_exact(12); vec }, 
            |vec| black_box({ let _ = vec.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "SmallVec",
        |b| b.iter_batched_ref(
            || SmallVec::<[u8; 16]>::from_vec(vec![0, 1, 2, 3]), 
            |vec| black_box({ let _ = vec.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "ArrayVec",
        |b| b.iter_batched_ref(
            || { let mut vec = ArrayVec::<u8, 16>::new(); (0..4).for_each(|v| vec.push(v)); vec }, 
            |vec| black_box({ let _ = vec.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.finish();

    let mut group = c.benchmark_group("swap_remove");
        group.bench_function(
        "BankArr",
        |b| b.iter_batched_ref(
            || { BankArr::<u8, 16>::from([0, 1, 2, 3]) }, 
            |bank| black_box({let _ = bank.swap_remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec - stack",
        |b| b.iter_batched_ref(
            || { BankVec::<u8, 16>::from([0, 1, 2, 3]) }, 
            |bank| black_box({let _ = bank.swap_remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec - heap",
        |b| b.iter_batched_ref(
            || BankVec::<u8, 14>::from([128; 16]), 
            |bank| black_box({let _ = bank.swap_remove(14); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "Vec",
        |b| b.iter_batched_ref(
            || { let mut vec: Vec<u8> = vec![0, 1, 2, 3]; vec.reserve_exact(12); vec }, 
            |vec| black_box({ let _ = vec.swap_remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "SmallVec",
        |b| b.iter_batched_ref(
            || SmallVec::<[u8; 16]>::from_vec(vec![0, 1, 2, 3]), 
            |vec| black_box({ let _ = vec.swap_remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "ArrayVec",
        |b| b.iter_batched_ref(
            || { let mut vec = ArrayVec::<u8, 16>::new(); (0..4).for_each(|v| vec.push(v)); vec }, 
            |vec| black_box({ let _ = vec.swap_remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.finish();


    let mut group = c.benchmark_group("extend");
    group.bench_function(
        "BankArr",
        |b| b.iter_batched_ref(
            || { BankArr::<u8, 16>::new() }, 
            |bank| { bank.extend(black_box([8u8; 8])); },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec - stack",
        |b| b.iter_batched_ref(
            || BankVec::<u8, 16>::new(), 
            |bank| { bank.extend(black_box([8u8; 8])); },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec - heap",
        |b| b.iter_batched_ref(
            || BankVec::<u8, 7>::from([8u8; 8]),
            |bank| { bank.extend(black_box([8u8; 8])); },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "Vec",
        |b| b.iter_batched_ref(
            || Vec::<u8>::with_capacity(16), 
            |vec| { vec.extend(black_box([8u8; 8])); },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "SmallVec",
        |b| b.iter_batched_ref(
            || SmallVec::<[u8; 16]>::new(), 
            |vec| { vec.extend(black_box([8u8; 8])); },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "ArrayVec",
        |b| b.iter_batched_ref(
            || ArrayVec::<u8, 16>::new(), 
            |vec| { vec.extend(black_box([8u8; 8])); },
            BatchSize::SmallInput
        )
    );
    group.finish();

}

criterion_group!(benches, benchmark);
criterion_main!(benches);