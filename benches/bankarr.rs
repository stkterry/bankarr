use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

use bankarr::{BankArr, BankVec};
use smallvec::SmallVec;
use arrayvec::ArrayVec;


pub fn benchmark(c: &mut Criterion) {

    let mut group = c.benchmark_group("push");
    group.sample_size(2000);
    group.bench_function(
        "Vec",
        |b| b.iter_batched_ref(
            || Vec::<i32>::with_capacity(16), 
            |vec| { black_box({ vec.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankArr",
        |b| b.iter_batched_ref(
            || BankArr::<i32, 16>::new(), 
            |bank| { black_box({ bank.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec",
        |b| b.iter_batched_ref(
            || BankVec::<i32, 16>::new(), 
            |bank| { black_box({ bank.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "SmallVec",
        |b| b.iter_batched_ref(
            || SmallVec::<[i32; 16]>::new(), 
            |vec| { black_box({ vec.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "ArrayVec",
        |b| b.iter_batched_ref(
            || ArrayVec::<i32, 16>::new(), 
            |vec| { black_box({ vec.push(black_box(128)); }) },
            BatchSize::SmallInput
        )
    );
    group.finish();


    let mut group = c.benchmark_group("insert");
    group.sample_size(2000);
    group.bench_function(
        "Vec",
        |b| b.iter_batched_ref(
            || Vec::<i32>::from([1, 2, 4, 5]), 
            |vec| black_box({ vec.insert(2, 3); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankArr",
        |b| b.iter_batched_ref(
            || BankArr::<i32, 16>::from([1, 2, 4, 5]), 
            |bank| black_box({ bank.insert(2, 3); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec",
        |b| b.iter_batched_ref(
            || BankVec::<i32, 16>::from([1, 2, 4, 5]), 
            |bank| black_box({ bank.insert(2, 3); }),
            BatchSize::SmallInput
        )
    );

    group.bench_function(
        "SmallVec",
        |b| b.iter_batched_ref(
            || SmallVec::<[i32; 16]>::from_vec(vec![1, 2, 3, 4]), 
            |vec| black_box({ vec.insert(2, 3); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "ArrayVec",
        |b| b.iter_batched_ref(
            || { let mut vec = ArrayVec::<i32, 16>::new(); (1..5).for_each(|v| vec.push(v)); vec}, 
            |vec| black_box({ vec.insert(2, 3); }),
            BatchSize::SmallInput
        )
    );
    group.finish();


    let mut group = c.benchmark_group("pop");
    group.sample_size(2000);
    group.bench_function(
        "Vec",
        |b| b.iter_batched_ref(
            || { let mut vec: Vec<i32> = vec![0, 1, 2, 3]; vec.reserve_exact(12); vec }, 
            |vec| black_box({ let _ = vec.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankArr",
        |b| b.iter_batched_ref(
            || BankArr::<i32, 16>::from([0, 1, 2, 3]), 
            |bank| black_box({let _ = bank.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec",
        |b| b.iter_batched_ref(
            || BankVec::<i32, 16>::from([0, 1, 2, 3]), 
            |bank| black_box({let _ = bank.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "SmallVec",
        |b| b.iter_batched_ref(
            || SmallVec::<[i32; 16]>::from_vec(vec![0, 1, 2, 3]), 
            |vec| black_box({ let _ = vec.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "ArrayVec",
        |b| b.iter_batched_ref(
            || { let mut vec = ArrayVec::<i32, 16>::new(); (0..4).for_each(|v| vec.push(v)); vec}, 
            |vec| black_box({ let _ = vec.pop(); }),
            BatchSize::SmallInput
        )
    );
    group.finish();

    let mut group = c.benchmark_group("remove");
    group.sample_size(2000);
    group.bench_function(
        "Vec",
        |b| b.iter_batched_ref(
            || { let mut vec: Vec<i32> = vec![0, 1, 2, 3]; vec.reserve_exact(12); vec }, 
            |vec| black_box({ let _ = vec.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankArr",
        |b| b.iter_batched_ref(
            || { BankArr::<i32, 16>::from([0, 1, 2, 3]) }, 
            |bank| black_box({let _ = bank.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec",
        |b| b.iter_batched_ref(
            || { BankVec::<i32, 16>::from([0, 1, 2, 3]) }, 
            |bank| black_box({let _ = bank.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "SmallVec",
        |b| b.iter_batched_ref(
            || SmallVec::<[i32; 16]>::from_vec(vec![0, 1, 2, 3]), 
            |vec| black_box({ let _ = vec.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "ArrayVec",
        |b| b.iter_batched_ref(
            || { let mut vec = ArrayVec::<i32, 16>::new(); (0..4).for_each(|v| vec.push(v)); vec }, 
            |vec| black_box({ let _ = vec.remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.finish();

    let mut group = c.benchmark_group("swap_remove");
    group.sample_size(2000);
    group.bench_function(
        "Vec",
        |b| b.iter_batched_ref(
            || {let mut vec = Vec::<i32>::from([0, 1, 2, 3]); vec.reserve_exact(12); vec}, 
            |vec| black_box({ let _ = vec.swap_remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankArr",
        |b| b.iter_batched_ref(
            || { BankArr::<i32, 16>::from([0, 1, 2, 3]) }, 
            |bank| black_box({let _ = bank.swap_remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec",
        |b| b.iter_batched_ref(
            || { BankVec::<i32, 16>::from([0, 1, 2, 3]) }, 
            |bank| black_box({let _ = bank.swap_remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "SmallVec",
        |b| b.iter_batched_ref(
            || SmallVec::<[i32; 16]>::from_vec(vec![0, 1, 2, 3]), 
            |vec| black_box({ let _ = vec.swap_remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "ArrayVec",
        |b| b.iter_batched_ref(
            || { let mut vec = ArrayVec::<i32, 16>::new(); (0..4).for_each(|v| vec.push(v)); vec }, 
            |vec| black_box({ let _ = vec.swap_remove(1); }),
            BatchSize::SmallInput
        )
    );
    group.finish();


    let mut group = c.benchmark_group("extend");
    group.sample_size(2000);
    group.bench_function(
        "Vec",
        |b| b.iter_batched_ref(
            || Vec::<i32>::with_capacity(16), 
            |vec| { vec.extend(black_box([8i32; 8])); },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankArr",
        |b| b.iter_batched_ref(
            || { BankArr::<i32, 16>::new() }, 
            |bank| { bank.extend(black_box([8i32; 8])); },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec",
        |b| b.iter_batched_ref(
            || { BankVec::<i32, 16>::new() }, 
            |bank| { bank.extend(black_box([8i32; 8])); },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "SmallVec",
        |b| b.iter_batched_ref(
            || SmallVec::<[i32; 16]>::new(), 
            |vec| { vec.extend(black_box([8i32; 8])); },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "ArrayVec",
        |b| b.iter_batched_ref(
            || ArrayVec::<i32, 16>::new(), 
            |vec| { vec.extend(black_box([8i32; 8])); },
            BatchSize::SmallInput
        )
    );
    group.finish();

    let mut group = c.benchmark_group("heap-realloc");
    group.sample_size(2000);
    group.bench_function(
        "Vec",
        |b| b.iter_batched_ref(
            || Vec::<i32>::from([8; 8]), 
            |vec| { vec.push(128); },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "BankVec",
        |b| b.iter_batched_ref(
            || BankVec::<i32, 8>::from([8; 8]), 
            |bank| { bank.push(128); },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "SmallVec",
        |b| b.iter_batched_ref(
            || SmallVec::<[i32; 8]>::from([8; 8]), 
            |vec| { vec.push(128); },
            BatchSize::SmallInput
        )
    );
    group.finish();

}

criterion_group!(benches, benchmark);
criterion_main!(benches);