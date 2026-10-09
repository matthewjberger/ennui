use crate::components::{Edge, Fill, Worn};
use crate::data::Resolved;
use ennui::later::set;
use ennui::prelude::{Entity, Later};
use ennui_text::prelude::{Height, Ink};
use nalgebra_glm::Vec4;

fn picked<T: PartialEq + Copy>(
    current: Option<T>,
    kept: T,
    first: bool,
    resolved: Option<T>,
) -> (Option<T>, T) {
    let Some(resolved) = resolved else {
        return (None, kept);
    };
    match current {
        None => (Some(resolved), resolved),
        Some(_) if first => (None, resolved),
        Some(current) if current == kept && resolved != kept => (Some(resolved), resolved),
        Some(_) => (None, kept),
    }
}

pub(crate) fn wear(
    later: &mut Later,
    entity: Entity,
    kept: Option<Worn>,
    current: (Option<Vec4>, Option<Vec4>, Option<Vec4>, Option<f32>),
    resolved: Resolved,
) {
    let first = kept.is_none();
    let kept = kept.unwrap_or_default();
    let (fill, worn_fill) = picked(current.0, kept.fill, first, resolved.0);
    let (edge, worn_edge) = picked(current.1, kept.edge, first, resolved.1);
    let (ink, worn_ink) = picked(current.2, kept.ink, first, resolved.2);
    let (height, worn_height) = picked(current.3, kept.height, first, resolved.3);
    if let Some(fill) = fill {
        set(later, entity, Fill(fill));
    }
    if let Some(edge) = edge {
        set(later, entity, Edge(edge));
    }
    if let Some(ink) = ink {
        set(later, entity, Ink(ink));
    }
    if let Some(height) = height {
        set(later, entity, Height(height));
    }
    let worn = Worn {
        fill: worn_fill,
        edge: worn_edge,
        ink: worn_ink,
        height: worn_height,
    };
    if first || worn != kept {
        set(later, entity, worn);
    }
}
