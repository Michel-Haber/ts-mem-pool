#![feature(test)]
#![allow(unused_doc_comments)]

extern crate test;
extern crate a_memory_pool;

use test::Bencher;
use test::black_box;
use a_memory_pool::memory_pool::MemoryPool;

const DATA_SIZE: usize = 500_000;
type DataType = u64;

#[bench]
fn unconstrained_allocation_bench(b: &mut Bencher) {
    b.iter(|| {
        for _ in 0..10 {
            black_box(Vec::<DataType>::with_capacity(DATA_SIZE));
        }
    })
}

#[bench]
fn unconstrained_memory_pool_bench(b: &mut Bencher) {
    let mem = MemoryPool::create_with(10, 20, Box::new(|| {
        Vec::<DataType>::with_capacity(DATA_SIZE)
    }));

    b.iter(|| {
        for _ in 0..10 {
            black_box(mem.get());
        }
    })
}
