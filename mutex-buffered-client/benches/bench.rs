use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mutex_buffered_client::{mutex_actor, mutex_worker, serial_actor, serial_worker};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::task::JoinSet;

const SAMPLE_SIZE: usize = 1000; // Number of samples to send
const WORKERS: [i32; 10] = [1, 4, 8, 16, 32, 64, 128, 256, 512, 1024]; // Different worker counts to test

fn mutex_bench(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap(); // Create a new Tokio runtime
    let buffer_size = 1024;

    for num_workers in WORKERS {
        c.bench_with_input(
            BenchmarkId::new("mutex_buffer_handling", num_workers),
            &num_workers,
            |b, &num_workers| {
                b.to_async(&runtime).iter(|| async {
                    let buffer = Arc::new(Mutex::new(Vec::with_capacity(buffer_size)));
                    let mut join_set = JoinSet::new();

                    // Spawn workers
                    for _ in 0..num_workers {
                        join_set.spawn(mutex_worker(Arc::clone(&buffer), SAMPLE_SIZE));
                    }

                    // Actor handling the buffer
                    mutex_actor(Arc::clone(&buffer), SAMPLE_SIZE).await;

                    join_set.shutdown().await;
                });
            },
        );
    }
}

fn serial_bench(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap(); // Create a new Tokio runtime

    for num_workers in WORKERS {
        c.bench_with_input(
            BenchmarkId::new("serial_buffer_handling", num_workers),
            &num_workers,
            |b, &num_workers| {
                b.to_async(&runtime).iter(|| async {
                    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                    let mut join_set = JoinSet::new();

                    // Actor handling the buffer, note we extend the sample size to 
                    // allow for the actor to consume all the samples of all workers.
                    join_set.spawn(serial_actor(rx, SAMPLE_SIZE * (num_workers as usize)));

                    // Spawn workers
                    for _ in 0..num_workers {
                        join_set.spawn(serial_worker(tx.clone(), SAMPLE_SIZE));
                    }

                    join_set.shutdown().await;
                });
            },
        );
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .sample_size(20);
    targets = mutex_bench, serial_bench
}
criterion_main!(benches);
