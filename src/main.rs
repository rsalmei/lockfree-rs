use std::cell::UnsafeCell;
use std::iter::repeat;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let buffer = Arc::new(Buffer::<200>::new());
    let mut threads = vec![];
    for i in 1..10 {
        let buffer = buffer.clone();
        threads.push(thread::spawn(move || {
            for j in 0..10 {
                let data = repeat(i as u8).take(i).collect::<Vec<_>>();
                if buffer.push(&data) == 0 {
                    println!("{} filled in {}", i, j);
                    return;
                }
            }
            println!("{} ended", i);
        }));
    }

    for t in threads {
        t.join().unwrap();
    }
    println!("{:?}", unsafe { *buffer.buf.get() })
}

struct Buffer<const N: usize> {
    buf: UnsafeCell<[u8; N]>,
    next_index: AtomicUsize,
}

unsafe impl<const N: usize> Send for Buffer<N> {}
unsafe impl<const N: usize> Sync for Buffer<N> {}

impl<const N: usize> Buffer<N> {
    fn new() -> Self {
        Self {
            buf: [0; N].into(),
            next_index: Default::default(),
        }
    }

    fn push(&self, mut data: &[u8]) -> usize {
        let index = self.next_index.fetch_add(data.len(), Ordering::Relaxed);
        if index + data.len() > N {
            data = &data[..N - index];
            self.next_index.store(N, Ordering::Relaxed);
        }

        let ptr = self.buf.get() as *mut u8;
        for (i, &d) in (index..).zip(data.iter()) {
            unsafe {
                // SAFETY: the starting index is atomic, and we won't write out of data.len() range.
                *ptr.add(i) = d;
            }
        }

        data.len()
    }
}
