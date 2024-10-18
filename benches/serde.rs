use divan::counter::{BytesCount, CharsCount, ItemsCount};
use divan::AllocProfiler;
use std::collections::*;
use victory_data_store::{primitives::serde::serialize::to_map, test_util::BigState};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();
fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn bench_to_map_rate(bencher: divan::Bencher) {
    let len: usize = 100;

    bencher
        .with_inputs(|| -> Vec<BigState> { vec![BigState::new(); len] })
        .input_counter(|s: &Vec<BigState>| {
            // Changes based on input.
            BytesCount::of_iter(s.iter())
        })
        .input_counter(|s: &Vec<BigState>| {
            // Changes based on input.
            ItemsCount::of_iter(s.iter())
        })
        .bench_refs(|s: &mut Vec<BigState>| to_map(s).unwrap());
}
