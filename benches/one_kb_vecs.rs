#![feature(allocator_api)]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use pool::PoolAllocator;


fn one_kb_vecs(alloc_dealloc_step: (usize, usize), pool_size_bytes: usize) {
    const BLOCK_SZ_BYTES: usize = 1024;

    let (allocs, deallocs) = alloc_dealloc_step;
    let pool = PoolAllocator::new(BLOCK_SZ_BYTES, pool_size_bytes / BLOCK_SZ_BYTES);

    let num_vectors = pool_size_bytes / 1024;

    // Let's keep the allocated vectors in a vector as well.
    let mut allocated_vectors = vec![];
    allocated_vectors.reserve(allocs);

    for _ in 0..num_vectors {
        if allocated_vectors.len() == allocs {
            // Dealloc last deallocs number of vectors
            for _ in 0..deallocs {
                allocated_vectors.pop();
            }
        }

        for _ in 0..num_vectors {
            let vec: Vec<u8, &PoolAllocator> = Vec::new_in(&pool);
            black_box(&vec);

            allocated_vectors.push(vec);
            black_box(&allocated_vectors);
        }
    }
}


fn one_kb_vecs_default_allocator(alloc_dealloc_step: (usize, usize)) {
    const BLOCK_SZ_BYTES: usize = 1024;

    let (allocs, deallocs) = alloc_dealloc_step;

    let num_vectors = 1024;

    // Let's keep the allocated vectors in a vector as well.
    let mut allocated_vectors = vec![];
    allocated_vectors.reserve(allocs);

    for _ in 0..num_vectors {
        if allocated_vectors.len() == allocs {
            // Dealloc last deallocs number of vectors
            for _ in 0..deallocs {
                allocated_vectors.pop();
            }
        }

        for _ in 0..num_vectors {
            let vec: Vec<u8> = Vec::new();
            black_box(&vec);

            allocated_vectors.push(vec);
            black_box(&allocated_vectors);
        }
    }
}


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("one kb vecs (5,2)", |b| b.iter(|| one_kb_vecs((5, 2), 1024)));

    c.bench_function("one kb vecs (5,2) default allocator", |b| b.iter(|| one_kb_vecs_default_allocator((5, 2))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
