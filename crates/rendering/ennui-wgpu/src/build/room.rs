pub fn uniform(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

const ROOM_AHEAD: usize = 1 << 18;

struct Moving {
    buffer: wgpu::Buffer,
    capacity: usize,
    copied: u64,
    cleared: u64,
}

pub struct Growing {
    pub buffer: wgpu::Buffer,
    label: &'static str,
    pub stride: u64,
    usage: wgpu::BufferUsages,
    pub(crate) capacity: usize,
    age: u64,
    moving: Option<Moving>,
}

#[derive(Default)]
pub struct Tied {
    pub group: Option<wgpu::BindGroup>,
    ages: Vec<u64>,
}

pub enum Part<'a> {
    Grown(&'a Growing),
    Held(wgpu::BindingResource<'a>),
}

pub fn new(
    device: &wgpu::Device,
    label: &'static str,
    stride: u64,
    capacity: usize,
    usage: wgpu::BufferUsages,
) -> Growing {
    Growing {
        buffer: made(device, label, stride * capacity as u64, usage),
        label,
        stride,
        usage,
        capacity,
        age: 0,
        moving: None,
    }
}

fn most_of(growing: &Growing, device: &wgpu::Device) -> usize {
    let limits = device.limits();
    let largest = match growing.usage.contains(wgpu::BufferUsages::STORAGE) {
        true => limits
            .max_buffer_size
            .min(limits.max_storage_buffer_binding_size),
        false => limits.max_buffer_size,
    };
    (largest / growing.stride) as usize
}

fn grown_capacity(growing: &Growing, device: &wgpu::Device, wanted: usize) -> usize {
    let most = most_of(growing, device);
    let small = wanted.next_power_of_two().min(wanted + ROOM_AHEAD);
    let grown = growing.capacity + growing.capacity / 2;
    small.max(grown).min(most).max(wanted)
}

pub fn room(growing: &mut Growing, device: &wgpu::Device, wanted: usize) -> bool {
    if wanted <= growing.capacity {
        return false;
    }
    growing.moving = None;
    growing.capacity = grown_capacity(growing, device, wanted);
    growing.buffer = made(
        device,
        growing.label,
        growing.stride * growing.capacity as u64,
        growing.usage,
    );
    growing.age += 1;
    true
}

pub(crate) fn copy_along(
    growing: &mut Growing,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    most: u64,
) -> bool {
    let Some(moving) = growing.moving.as_mut() else {
        return false;
    };
    let total = growing.buffer.size();
    let room = moving.buffer.size();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(growing.label),
    });
    let mut left = most;
    let length = (total - moving.copied).min(left);
    if length > 0 {
        encoder.copy_buffer_to_buffer(
            &growing.buffer,
            moving.copied,
            &moving.buffer,
            moving.copied,
            length,
        );
        moving.copied += length;
        left -= length;
    }
    let length = (room - moving.cleared).min(left) & !3;
    if length > 0 {
        encoder.clear_buffer(&moving.buffer, moving.cleared, Some(length));
        moving.cleared += length;
    }
    queue.submit([encoder.finish()]);
    if moving.copied < total || moving.cleared + 4 <= room {
        return false;
    }
    let Some(moved) = growing.moving.take() else {
        return false;
    };
    growing.buffer = moved.buffer;
    growing.capacity = moved.capacity;
    growing.age += 1;
    true
}

pub fn keep(
    growing: &mut Growing,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    wanted: usize,
) -> bool {
    let mut grown = false;
    if wanted > growing.capacity && growing.moving.is_some() {
        grown = copy_along(growing, device, queue, u64::MAX);
    }
    if wanted <= growing.capacity {
        let early = growing.capacity > ROOM_AHEAD && wanted * 5 > growing.capacity * 3;
        if early && growing.moving.is_none() {
            let capacity = grown_capacity(growing, device, growing.capacity + 1);
            if capacity > growing.capacity {
                growing.moving = Some(Moving {
                    buffer: made(
                        device,
                        growing.label,
                        growing.stride * capacity as u64,
                        growing.usage,
                    ),
                    capacity,
                    copied: 0,
                    cleared: growing.buffer.size(),
                });
            }
        }
        return grown;
    }
    let old = growing.buffer.clone();
    if !room(growing, device, wanted) {
        return grown;
    }
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(growing.label),
    });
    encoder.copy_buffer_to_buffer(&old, 0, &growing.buffer, 0, old.size());
    queue.submit([encoder.finish()]);
    true
}

pub fn fill<T: bytemuck::Pod>(
    growing: &mut Growing,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    items: &[T],
) -> bool {
    let grown = room(growing, device, items.len());
    write(growing, queue, 0, items);
    grown
}

pub fn write<T: bytemuck::Pod>(growing: &Growing, queue: &wgpu::Queue, at: usize, items: &[T]) {
    if items.is_empty() {
        return;
    }
    queue.write_buffer(
        &growing.buffer,
        at as u64 * growing.stride,
        bytemuck::cast_slice(items),
    );
    if let Some(moving) = growing.moving.as_ref() {
        queue.write_buffer(
            &moving.buffer,
            at as u64 * growing.stride,
            bytemuck::cast_slice(items),
        );
    }
}

pub fn tie(
    tied: &mut Tied,
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    parts: Vec<Part<'_>>,
) {
    let slotted = parts
        .into_iter()
        .enumerate()
        .map(|(slot, part)| (slot as u32, part))
        .collect();
    tie_at(tied, device, label, layout, slotted);
}

pub(crate) fn tie_at(
    tied: &mut Tied,
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    parts: Vec<(u32, Part<'_>)>,
) {
    let ages: Vec<u64> = parts
        .iter()
        .filter_map(|(_, part)| match part {
            Part::Grown(grown) => Some(grown.age),
            Part::Held(_) => None,
        })
        .collect();
    if tied.group.is_some() && tied.ages == ages {
        return;
    }
    let held: Vec<(u32, wgpu::BindingResource<'_>)> = parts
        .iter()
        .map(|(slot, part)| match part {
            Part::Grown(grown) => (*slot, grown.buffer.as_entire_binding()),
            Part::Held(held) => (*slot, held.clone()),
        })
        .collect();
    tied.group = Some(crate::build::binding_at(device, label, layout, held));
    tied.ages = ages;
}

pub fn made(
    device: &wgpu::Device,
    label: &'static str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(256),
        usage: usage | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    })
}
