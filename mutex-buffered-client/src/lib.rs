use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn mutex_worker(buf: Arc<Mutex<Vec<u8>>>, samples: u64) {
    let mut potato = 0;
    
    for _ in 0..samples {
        potato = (potato + 1) % 255;

        let mut buf = buf.lock().await;
        buf.push(potato);
        drop(buf);
    }
}

pub async fn mutex_actor(buf: Arc<Mutex<Vec<u8>>>, samples: u64) {
    let mut iters = 0;

    while iters < samples {
        let mut current = buf.lock().await;

        // Swap the buffer with an empty one.
        let mut buffer = Vec::with_capacity(current.capacity());
        std::mem::swap(&mut buffer, &mut *current);
        drop(current);

        // Instead of writing to a socket, we'll just sleep for a bit. We do
        // this to avoid including I/O in the benchmark.
        //tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        iters += 1;
    }
}

pub async fn serial_worker(tx: tokio::sync::mpsc::UnboundedSender<u8>, samples: u64) {
    let mut potato = 0;

    for _ in 0..samples {
        potato = (potato + 1) % 255;
        if tx.send(potato).is_err() {
            eprintln!("Receiver has dropped.");
            break;
        }
    }
}

pub async fn serial_actor(mut rx: tokio::sync::mpsc::UnboundedReceiver<u8>, samples: u64) {
    let mut iters = 0;

    while iters < samples {
        let _ = rx.recv().await;

        // Instead of writing to a socket, we'll just sleep for a bit. We do
        // this to avoid including I/O in the benchmark.
        //tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        iters += 1;
    }
}
