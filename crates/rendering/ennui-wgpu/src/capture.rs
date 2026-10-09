use std::path::Path;
use std::sync::mpsc::channel;

const ROW_ALIGNMENT: u32 = 256;

pub(crate) fn write_picture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    format: wgpu::TextureFormat,
    white: f32,
    path: &Path,
) {
    let width = texture.width();
    let height = texture.height();
    let wide = format == wgpu::TextureFormat::Rgba16Float;
    let tight = width * if wide { 8 } else { 4 };
    let padded = tight.div_ceil(ROW_ALIGNMENT) * ROW_ALIGNMENT;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("capture readback"),
        size: u64::from(padded) * u64::from(height),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("capture copy"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    let (sender, receiver) = channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |outcome| {
            sender.send(outcome.is_ok()).ok();
        });
    let mapped = loop {
        if let Ok(outcome) = receiver.try_recv() {
            break outcome;
        }
        if device.poll(wgpu::PollType::Poll).is_err() {
            log::error!("the device stopped before the picture came back");
            return;
        }
    };
    if !mapped {
        log::error!("the picture for {} did not map", path.display());
        return;
    }

    let swap = matches!(
        format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    );
    let view = readback.slice(..).get_mapped_range();
    let mut pixels = Vec::with_capacity((width * 4 * height) as usize);
    for row in 0..height {
        let start = (row * padded) as usize;
        if wide {
            for pixel in view[start..start + tight as usize].as_chunks::<8>().0 {
                pixels.extend([
                    papered(pixel[0], pixel[1], white),
                    papered(pixel[2], pixel[3], white),
                    papered(pixel[4], pixel[5], white),
                    255,
                ]);
            }
            continue;
        }
        for pixel in view[start..start + tight as usize].as_chunks::<4>().0 {
            if swap {
                pixels.extend([pixel[2], pixel[1], pixel[0], 255]);
            } else {
                pixels.extend([pixel[0], pixel[1], pixel[2], 255]);
            }
        }
    }
    drop(view);
    readback.unmap();

    let made = std::fs::File::create(path);
    let Ok(file) = made else {
        log::error!("{} did not open for the picture", path.display());
        return;
    };
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    match encoder.write_header() {
        Ok(mut writer) => {
            if let Err(cause) = writer.write_image_data(&pixels) {
                log::error!("{} did not take the picture: {cause}", path.display());
            }
        }
        Err(cause) => log::error!("{} did not take a header: {cause}", path.display()),
    }
}

fn papered(low: u8, high: u8, white: f32) -> u8 {
    let linear = (widened(u16::from_le_bytes([low, high])) / white.max(1.0e-4)).clamp(0.0, 1.0);
    let encoded = match linear <= 0.003_130_8 {
        true => linear * 12.92,
        false => 1.055 * linear.powf(1.0 / 2.4) - 0.055,
    };
    (encoded * 255.0 + 0.5) as u8
}

fn widened(half: u16) -> f32 {
    let sign = if half & 0x8000 == 0 { 1.0 } else { -1.0 };
    let exponent = i32::from((half >> 10) & 0x1f);
    let fraction = f32::from(half & 0x3ff);
    match exponent {
        0 => sign * fraction * 2.0f32.powi(-24),
        31 => sign * f32::INFINITY,
        _ => sign * (1.0 + fraction / 1024.0) * 2.0f32.powi(exponent - 15),
    }
}
