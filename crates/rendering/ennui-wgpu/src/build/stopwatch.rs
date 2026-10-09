use crate::build::readback::{Heard, Readback, ask, heard, readback, release};
use crate::wgpu;
use std::collections::BTreeMap;
use std::sync::Mutex;

const MOST_SPANS: u32 = 128;

struct Reading {
    readback: Readback,
    names: Vec<&'static str>,
    copied: bool,
}

pub struct Stopwatch {
    set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    each_tick: f32,
    claimed: Mutex<Vec<&'static str>>,
    readings: [Mutex<Reading>; 2],
    turn: Mutex<usize>,
    sums: Mutex<BTreeMap<&'static str, (f64, u64, f64)>>,
    pub(crate) last: Mutex<f64>,
}

pub(crate) fn build(device: &wgpu::Device, queue: &wgpu::Queue) -> Stopwatch {
    let reading = |label| {
        Mutex::new(Reading {
            readback: readback(device, label, (MOST_SPANS as u64) * 16),
            names: Vec::new(),
            copied: false,
        })
    };
    Stopwatch {
        set: device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("stopwatch"),
            ty: wgpu::QueryType::Timestamp,
            count: MOST_SPANS * 2,
        }),
        resolve: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("stopwatch resolve"),
            size: (MOST_SPANS as u64) * 16,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }),
        each_tick: queue.get_timestamp_period(),
        claimed: Mutex::new(Vec::new()),
        readings: [reading("stopwatch read one"), reading("stopwatch read two")],
        turn: Mutex::new(0),
        sums: Mutex::new(BTreeMap::new()),
        last: Mutex::new(0.0),
    }
}

pub fn claim<'a>(
    stopwatch: impl Into<Option<&'a Stopwatch>>,
    name: &'static str,
) -> Option<wgpu::RenderPassTimestampWrites<'a>> {
    let stopwatch = stopwatch.into()?;
    let mut claimed = stopwatch.claimed.lock().ok()?;
    if claimed.len() >= MOST_SPANS as usize {
        return None;
    }
    let slot = claimed.len() as u32;
    claimed.push(name);
    Some(wgpu::RenderPassTimestampWrites {
        query_set: &stopwatch.set,
        beginning_of_pass_write_index: Some(slot * 2),
        end_of_pass_write_index: Some(slot * 2 + 1),
    })
}

pub(crate) fn resolve(stopwatch: &Stopwatch, encoder: &mut wgpu::CommandEncoder) {
    let Ok(names) = stopwatch.claimed.lock() else {
        return;
    };
    let claimed = names.len() as u32;
    if claimed == 0 {
        return;
    }
    encoder.resolve_query_set(&stopwatch.set, 0..claimed * 2, &stopwatch.resolve, 0);
    let turn = match stopwatch.turn.lock() {
        Ok(held) => *held,
        Err(_) => return,
    };
    if let Ok(mut reading) = stopwatch.readings[turn].lock()
        && heard(&reading.readback) == Heard::Free
    {
        reading.names = names.clone();
        reading.copied = true;
        encoder.copy_buffer_to_buffer(
            &stopwatch.resolve,
            0,
            &reading.readback.buffer,
            0,
            (claimed as u64) * 16,
        );
    }
}

pub(crate) fn hear(stopwatch: &Stopwatch) {
    for reading in stopwatch
        .readings
        .iter()
        .filter_map(|held| held.lock().ok())
    {
        match heard(&reading.readback) {
            Heard::Free | Heard::Waiting => continue,
            Heard::Failed => {
                release(&reading.readback);
                continue;
            }
            Heard::Mapped => {}
        }
        let ticks: Vec<u64> = {
            let held = reading.readback.buffer.slice(..).get_mapped_range();
            bytemuck::cast_slice(&held).to_vec()
        };
        release(&reading.readback);
        let mut framed: BTreeMap<&'static str, f64> = BTreeMap::new();
        for (slot, name) in reading.names.iter().enumerate() {
            let start = ticks.get(slot * 2).copied().unwrap_or(0);
            let end = ticks.get(slot * 2 + 1).copied().unwrap_or(0);
            if end > start {
                let spent = (end - start) as f64 * stopwatch.each_tick as f64 / 1_000_000.0;
                *framed.entry(name).or_insert(0.0) += spent;
            }
        }
        if let Ok(mut last) = stopwatch.last.lock() {
            *last = framed.values().sum();
        }
        if let Ok(mut sums) = stopwatch.sums.lock() {
            for (name, spent) in framed {
                let held = sums.entry(name).or_insert((0.0, 0, 0.0));
                held.0 += spent;
                held.1 += 1;
                held.2 = held.2.max(spent);
            }
        }
    }
    let Ok(mut turn) = stopwatch.turn.lock() else {
        return;
    };
    if let Ok(mut reading) = stopwatch.readings[*turn].lock()
        && reading.copied
        && heard(&reading.readback) == Heard::Free
    {
        reading.copied = false;
        ask(&reading.readback);
        *turn = (*turn + 1) % 2;
    }
}

pub(crate) fn open(stopwatch: &Stopwatch) {
    if let Ok(mut claimed) = stopwatch.claimed.lock() {
        claimed.clear();
    }
}

pub(crate) fn report(stopwatch: &Stopwatch) -> Vec<(&'static str, f64, u64, f64)> {
    let Ok(sums) = stopwatch.sums.lock() else {
        return Vec::new();
    };
    let mut held: Vec<(&'static str, f64, u64, f64)> = sums
        .iter()
        .map(|(name, (total, count, worst))| {
            (*name, total / (*count).max(1) as f64, *count, *worst)
        })
        .collect();
    held.sort_by(|left, right| right.1.total_cmp(&left.1).then(left.0.cmp(right.0)));
    held
}
