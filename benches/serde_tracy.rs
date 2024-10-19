use std::collections::HashMap;
use std::sync::Arc;

use divan::counter::{BytesCount, ItemsCount};
use serde::Deserialize;
use tracing_tracy::client::ProfiledAllocator;
use tracy::alloc::GlobalAllocator;
use victory_data_store::database::Datastore;
use victory_data_store::primitives::serde::deserializer::PrimitiveDeserializer;
use victory_data_store::primitives::Primitives;
use victory_data_store::topics::TopicKey;
use victory_data_store::{primitives::serde::serialize::to_map, test_util::BigState};

use tracing_subscriber::layer::SubscriberExt;
use tracing_tracy::TracyLayer;
use victory_time_rs::Timepoint;

#[global_allocator]
static GLOBAL: ProfiledAllocator<std::alloc::System> =
    ProfiledAllocator::new(std::alloc::System, 200);

fn main() {
    // Run registered benchmarks.
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(TracyLayer::default()),
    )
    .expect("setup tracy layer");

    divan::main();
}

#[divan::bench]
fn bench_to_map_rate(bencher: divan::Bencher) {
    let len: usize = 10;

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
    let len: usize = 10;

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
    let len: usize = 10;

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
