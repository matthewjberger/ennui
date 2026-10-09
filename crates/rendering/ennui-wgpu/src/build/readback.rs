use crate::wgpu;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

const FREE: u8 = 0;
const WAITING: u8 = 1;
const MAPPED: u8 = 2;
const FAILED: u8 = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Heard {
    Free,
    Waiting,
    Mapped,
    Failed,
}

pub struct Readback {
    pub buffer: wgpu::Buffer,
    state: Arc<AtomicU8>,
}

pub fn readback(device: &wgpu::Device, label: &str, size: u64) -> Readback {
    Readback {
        buffer: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        state: Arc::new(AtomicU8::new(FREE)),
    }
}

pub fn ask(readback: &Readback) {
    let state = readback.state.clone();
    state.store(WAITING, Ordering::Release);
    readback
        .buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |outcome| {
            let settled = match outcome {
                Ok(()) => MAPPED,
                Err(_) => FAILED,
            };
            state.store(settled, Ordering::Release);
        });
}

pub fn heard(readback: &Readback) -> Heard {
    match readback.state.load(Ordering::Acquire) {
        WAITING => Heard::Waiting,
        MAPPED => Heard::Mapped,
        FAILED => Heard::Failed,
        _ => Heard::Free,
    }
}

pub fn release(readback: &Readback) {
    if heard(readback) == Heard::Mapped {
        readback.buffer.unmap();
    }
    readback.state.store(FREE, Ordering::Release);
}
