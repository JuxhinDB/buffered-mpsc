use std::sync::Arc;
use std::sync::Mutex;

pub async fn mutex_worker(buf: Arc<Mutex<Vec<u8>>>, samples: usize) {
    let mut potato = 0;

    for _ in 0..samples {
        potato = (potato + 1) % 255;

        let mut buf = buf.lock().unwrap();
        buf.push(potato);
        drop(buf);
    }
}

pub async fn mutex_actor(buf: Arc<Mutex<Vec<u8>>>, samples: usize) {
    let mut iters = 0;

    // NOTE(jdb): We only allocate two buffers, whenever the consumer
    // consumes, it swaps the worker buffer with the current buffer
    // that is empty, and the current buffer is then consumed/cleared.
    let mut buffer = Vec::with_capacity(samples);

    while iters < samples {
        let mut current = buf.lock().unwrap();

        // Swap the buffer with an empty one.
        std::mem::swap(&mut buffer, &mut *current);
        drop(current);

        // Consume the buffer
        buffer.clear();

        iters += 1;
    }
}

pub async fn serial_worker(tx: tokio::sync::mpsc::UnboundedSender<u8>, samples: usize) {
    let mut potato = 0;

    for _ in 0..samples {
        potato = (potato + 1) % 255;
        if tx.send(potato).is_err() {
            eprintln!("Receiver has dropped.");
            break;
        }
    }
}

pub async fn serial_actor(mut rx: tokio::sync::mpsc::UnboundedReceiver<u8>, samples: usize) {
    let mut iters = 0;

    while iters < samples {
        let _ = rx.recv().await;

        iters += 1;
    }
}
