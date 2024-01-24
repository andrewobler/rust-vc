use std::sync::{Arc, Mutex};

use dasp_ring_buffer::Bounded;
use dasp_sample::Sample;

pub type BufferHandle = Arc<Mutex<Buffer>>;

pub struct Buffer {
    buf: Bounded<Vec<u8>>,
}

impl Buffer {
    pub fn new(size_bound: usize) -> Self {
        Self {
            buf: Bounded::from(vec![Sample::EQUILIBRIUM; size_bound]),
        }
    }

    pub fn new_handle(size_bound: usize) -> BufferHandle {
        Arc::new(Mutex::new(Self::new(size_bound)))
    }

    pub fn push(&mut self, sample: u8) -> Option<u8> {
        self.buf.push(sample)
    }

    pub fn pop(&mut self) -> Option<u8> {
        self.buf.pop()
    }

    pub fn push_all<'a, T: Iterator<Item = &'a u8>>(&mut self, iterable: T) {
        self.push_all_with_transform(iterable, |sample| *sample);
    }

    pub fn push_all_with_transform<'a, S: 'a, T: Iterator<Item = &'a S>, F: Fn(&S) -> u8>(
        &mut self,
        iterable: T,
        transform: F,
    ) {
        for sample in iterable {
            self.push(transform(sample));
        }
    }

    pub fn pop_into<'a, T: Iterator<Item = &'a mut u8>>(&mut self, iterable: T) -> usize {
        self.pop_into_with_transform(iterable, |sample| *sample)
    }

    pub fn pop_into_with_transform<'a, S: 'a, T: Iterator<Item = &'a mut S>, F: Fn(&u8) -> S>(
        &mut self,
        iterable: T,
        transform: F,
    ) -> usize {
        let mut num_written = 0usize;
        for dst in iterable {
            if let Some(sample) = self.pop() {
                *dst = transform(&sample);
                num_written += 1;
            } else {
                break;
            }
        }

        num_written
    }
}
