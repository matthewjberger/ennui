use crate::components::{Host, Hosted, Rect};
use crate::data::{Hosting, Mount, Sizing};
use crate::resources::{Hosts, Interface, Letterbox, Wearing};
use crate::theme::{FAR_POINTER, HOST_REFERENCE};
use ennui::prelude::{Entity, Storage};
use ennui::storage::get;
use ennui_platform::prelude::Viewport;
use ennui_quads::prelude::Quad;
use nalgebra_glm::Vec2;

pub fn fitted(viewport: &Viewport) -> f32 {
    pixels_per_reference(
        Sizing::Pixel,
        HOST_REFERENCE,
        Vec2::new(viewport.width.max(1) as f32, viewport.height.max(1) as f32),
        viewport.density,
    )
}

fn pixels_per_reference(sizing: Sizing, reference: Vec2, pixels: Vec2, density: f32) -> f32 {
    let reference = nalgebra_glm::max2(&reference, &Vec2::repeat(1.0));
    match sizing {
        Sizing::Pixel => {
            density
                * (pixels.x / (reference.x * density))
                    .min(pixels.y / (reference.y * density))
                    .min(1.0)
        }
        Sizing::Wide => pixels.x / reference.x,
        Sizing::Tall => pixels.y / reference.y,
        Sizing::Short => (pixels.x / reference.x).min(pixels.y / reference.y),
    }
}

fn window_rect_of(
    hosts: &[Hosting],
    previous: &Hosts,
    rows: &Storage,
    target: Entity,
) -> Option<Rect> {
    let rect = *get::<Rect>(rows, target)?;
    let host = get::<Hosted>(rows, target)?.0;
    let held = hosts
        .iter()
        .chain(previous.list.iter())
        .find(|held| held.host == host)?;
    Some(Rect {
        center: held.offset + rect.center * held.scale,
        size: rect.size * held.scale,
    })
}

pub(crate) fn hosting_of(
    rows: &Storage,
    (hosts, previous): (&[Hosting], &Hosts),
    (viewport, interface, letterbox, wearing): (&Viewport, &Interface, &Letterbox, &Wearing),
    (entity, host): (Entity, Host),
) -> Hosting {
    let aspect = viewport.width.max(1) as f32 / viewport.height.max(1) as f32;
    let tall = viewport.height.max(1) as f32;
    let whole = Rect {
        center: Vec2::zeros(),
        size: Vec2::new(2.0 * aspect, 2.0),
    };
    let window = match host.mount {
        Mount::Screen => match letterbox.inside.filter(|_| host.scaled) {
            Some(target) => window_rect_of(hosts, previous, rows, target),
            None => Some(whole),
        },
        Mount::Inside => host
            .inside
            .and_then(|target| window_rect_of(hosts, previous, rows, target)),
    };
    let grown = match host.scaled {
        true => interface.scale.max(f32::EPSILON),
        false => 1.0,
    };
    let (size, pixels, scale, offset) = match window {
        Some(rect) => {
            let pixels = rect.size * tall * 0.5;
            let per = pixels_per_reference(host.sizing, host.reference, pixels, viewport.density)
                .max(f32::EPSILON)
                * grown;
            (pixels / per, per, 2.0 / tall * per, rect.center)
        }
        None => (host.reference / grown, 0.0, 0.0, Vec2::zeros()),
    };
    Hosting {
        host: entity,
        mount: host.mount,
        layer: host.layer,
        modal: host.modal,
        theme: host.theme.or(wearing.theme.filter(|_| host.scaled)),
        size,
        pixels,
        scale,
        offset,
        pointer: Vec2::repeat(FAR_POINTER),
    }
}

pub fn pointer_of(hosts: &Hosts, host: Entity) -> Vec2 {
    hosts
        .list
        .iter()
        .find(|held| held.host == host)
        .map_or(Vec2::repeat(FAR_POINTER), |held| held.pointer)
}

pub fn pointer_in(hosts: &Hosts, hosted: Option<&Hosted>) -> Vec2 {
    match hosted {
        Some(hosted) => pointer_of(hosts, hosted.0),
        None => hosts
            .list
            .first()
            .map_or(Vec2::repeat(FAR_POINTER), |held| held.pointer),
    }
}

pub fn host_point(hosts: &Hosts, host: Entity, at: Vec2) -> Vec2 {
    hosts
        .list
        .iter()
        .find(|held| held.host == host && held.scale > 0.0)
        .map_or(at, |held| (at - held.offset) / held.scale)
}

pub fn screen_rect(hosts: &Hosts, host: Entity, rect: Rect) -> Rect {
    hosts
        .list
        .iter()
        .find(|held| held.host == host)
        .map_or(rect, |held| Rect {
            center: held.offset + rect.center * held.scale,
            size: rect.size * held.scale,
        })
}

fn fenced(clip: [f32; 4], bounds: [f32; 4]) -> [f32; 4] {
    if clip[2] <= 0.0 {
        return bounds;
    }
    let low = [
        (clip[0] - clip[2]).max(bounds[0] - bounds[2]),
        (clip[1] - clip[3]).max(bounds[1] - bounds[3]),
    ];
    let high = [
        (clip[0] + clip[2]).min(bounds[0] + bounds[2]).max(low[0]),
        (clip[1] + clip[3]).min(bounds[1] + bounds[3]).max(low[1]),
    ];
    [
        (low[0] + high[0]) * 0.5,
        (low[1] + high[1]) * 0.5,
        ((high[0] - low[0]) * 0.5).max(f32::EPSILON),
        (high[1] - low[1]) * 0.5,
    ]
}

pub fn staged(hosting: &Hosting, quad: Quad) -> Quad {
    let scale = hosting.scale;
    let shift = |at: [f32; 2]| {
        [
            hosting.offset.x + at[0] * scale,
            hosting.offset.y + at[1] * scale,
        ]
    };
    let lettered = quad.shape[2] > 0.5;
    Quad {
        center: shift(quad.center),
        size: [quad.size[0] * scale, quad.size[1] * scale],
        shape: [quad.shape[0] * scale, quad.shape[1] * scale, quad.shape[2]],
        clip: {
            let clip = match quad.clip[2] > 0.0 {
                true => {
                    let [x, y] = shift([quad.clip[0], quad.clip[1]]);
                    [x, y, quad.clip[2] * scale, quad.clip[3] * scale]
                }
                false => quad.clip,
            };
            match hosting.mount {
                Mount::Screen => fenced(
                    clip,
                    [
                        hosting.offset.x,
                        hosting.offset.y,
                        hosting.size.x * scale * 0.5,
                        hosting.size.y * scale * 0.5,
                    ],
                ),
                _ => clip,
            }
        },
        blur: match lettered {
            true => quad.blur,
            false => quad.blur.map(|held| held * scale),
        },
        ..quad
    }
}
