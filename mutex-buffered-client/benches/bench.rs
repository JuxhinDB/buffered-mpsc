use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use mutex_buffered_client::{mutex_actor, mutex_worker, serial_actor};
use tokio::runtime::Runtime;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;


const WORKERS: [i32; 10] = [1, 4, 8, 16, 32, 64, 128, 256, 512, 1024]; // Different worker counts to test

fn mutex_bench(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap(); // Create a new Tokio runtime
    let buffer_size = 1024;
    let sample_size = 100_000;

    for num_workers in WORKERS {
        c.bench_with_input(BenchmarkId::new("mutex_buffer_handling", num_workers), &num_workers, |b, &num_workers| {
            b.to_async(&runtime).iter(|| async {
                let buffer = Arc::new(Mutex::new(Vec::with_capacity(buffer_size)));

                // Spawn workers
                let worker_handles: Vec<_> = (0..num_workers)
                    .map(|_| {
                        let cloned_buffer = Arc::clone(&buffer);
                        tokio::spawn(async move {
                            mutex_worker(cloned_buffer, sample_size).await;
                        })
                    })
                    .collect();

                for handle in worker_handles {
                    handle.await.unwrap();
                }

                // Actor handling the buffer
                mutex_actor(Arc::clone(&buffer), sample_size).await;
            });
        });
    }
}

fn serial_bench(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap(); // Create a new Tokio runtime
    let sample_size = 100_000;


    for num_workers in WORKERS {
        c.bench_with_input(BenchmarkId::new("serial_buffer_handling", num_workers), &num_workers, |b, &num_workers| {
            b.to_async(&runtime).iter(|| async {
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

                // Spawn workers
                let worker_handles: Vec<_> = (0..num_workers)
                    .map(|_| {
                        let cloned_tx = tx.clone();
                        tokio::spawn(async move {
                            for _ in 0..sample_size {
                                cloned_tx.send(0).unwrap();
                            }
                        })
                    })
                    .collect();


                for handle in worker_handles {
                    handle.await.unwrap();
                }

                // Actor handling the buffer
                serial_actor(rx, sample_size).await;
            });
        });
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(5))
        .sample_size(20);
    targets = mutex_bench, serial_bench
}
criterion_main!(benches);

