use std::collections::HashMap;
use std::sync::Arc;

use divan::counter::{BytesCount, ItemsCount};
use divan::AllocProfiler;
use serde::Deserialize;
use victory_data_store::database::Datastore;
use victory_data_store::primitives::serde::deserializer::PrimitiveDeserializer;
use victory_data_store::primitives::Primitives;
use victory_data_store::topics::TopicKey;
use victory_data_store::{primitives::serde::serialize::to_map, test_util::BigState};
use victory_time_rs::Timepoint;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();
fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn bench_to_map_rate(bencher: divan::Bencher) {
    let len: usize = 500;

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

#[divan::bench]
fn bench_from_map_rate(bencher: divan::Bencher) {
    let len: usize = 500;

    bencher
        .with_inputs(|| -> Vec<HashMap<Arc<TopicKey>, Primitives>> {
            vec![to_map(&BigState::new()).unwrap(); len]
        })
        .input_counter(|s: &Vec<HashMap<Arc<TopicKey>, Primitives>>| {
            // Changes based on input.
            BytesCount::of_iter(s.iter())
        })
        .input_counter(|s: &Vec<HashMap<Arc<TopicKey>, Primitives>>| {
            // Changes based on input.
            ItemsCount::of_iter(s.iter())
        })
        .bench_refs(|s: &mut Vec<HashMap<Arc<TopicKey>, Primitives>>| {
            s.iter()
                .map(|m| {
                    let mut deserializer = PrimitiveDeserializer::new(m);
                    let bs: BigState = Deserialize::deserialize(&mut deserializer).unwrap();
                    bs
                })
                .collect::<Vec<BigState>>()
        });
}

#[divan::bench]
fn bench_add_struct_rate(bencher: divan::Bencher) {
    let len: usize = 100;

    let topic_key = TopicKey::from_str("big_state");
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
        .bench_refs(|s: &mut Vec<BigState>| {
            let mut datastore = Datastore::new();
            for state in s.iter() {
                datastore
                    .add_struct(&topic_key, Timepoint::now(), state)
                    .unwrap();
            }
        });
}
