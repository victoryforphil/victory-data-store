use divan::counter::{BytesCount, ItemsCount};
use tracing_subscriber::layer::SubscriberExt;
use tracing_tracy::client::ProfiledAllocator;
use victory_data_store::{primitives::serde::serialize::to_map, test_util::BigState};
#[global_allocator]
static GLOBAL: ProfiledAllocator<std::alloc::System> =
    ProfiledAllocator::new(std::alloc::System, 100);
fn main() {
    // Run registered benchmarks.

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
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
