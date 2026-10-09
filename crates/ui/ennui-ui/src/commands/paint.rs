use crate::components::{
    Edge, Effect, Fill, Framed, Lettered, Picture, Rect, Region, Sharp, Shield, Style, Sunk, Tone,
    Turn, Warm,
};
use crate::data::{HostLaid, SLICES};
use crate::queries::paint::{raise, reader};
use crate::queries::theme::color_of;
use crate::resources::Theme;
use ennui::prelude::{Entity, Storage};
use ennui::storage::{get, has};
use ennui_quads::prelude::{NO_PICTURE, Quad};
use ennui_text::data::Wording;
use ennui_text::prelude::{
    Align, Aligned, Cut, Deep, Family, Glyphs, Height, Ink, Label, Rim, Soft, Trim, Wrap,
    paint_wording, trimmed,
};
use nalgebra_glm::Vec4;

pub(crate) fn close_frames(
    rows: &Storage,
    held: &HostLaid,
    open: &mut Vec<(usize, Entity, Rect, Framed)>,
    step: usize,
    quads: &mut Vec<Quad>,
) {
    while let Some(&(end, entity, rect, framed)) = open.last()
        && end < step
    {
        open.pop();
        let deepest = get::<Deep>(rows, held.order[end]).map_or(0.0, |deep| deep.0);
        frame_the_edges(rows, entity, rect, framed, deepest + 0.5, quads);
    }
}

pub(crate) fn fill(
    rows: &Storage,
    entity: Entity,
    rect: Rect,
    theme: &Theme,
    quads: &mut Vec<Quad>,
) -> Option<(Vec4, f32)> {
    let style = get::<Style>(rows, entity);
    let tone = get::<Tone>(rows, entity).copied().unwrap_or_default();
    let edged = tone.edge.is_some()
        || (style.is_some_and(|style| style.border > 0.0) && has::<Edge>(rows, entity));
    let fill = get::<Fill>(rows, entity)
        .map(|held| held.0)
        .or_else(|| edged.then(Vec4::zeros))?;
    let lively = f32::from(!has::<Shield>(rows, entity));
    let warm = get::<Warm>(rows, entity).map_or(0.0, |held| held.0) * lively;
    let sunk = get::<Sunk>(rows, entity).map_or(0.0, |held| held.0) * lively;
    let (hover, press) = match style {
        Some(style) => (&style.hover, &style.press),
        None => (&theme.hover, &theme.press),
    };
    let lift = warm * hover.lift.or(theme.hover.lift).unwrap_or(0.0)
        + sunk * press.lift.or(theme.press.lift).unwrap_or(0.0);
    let swell = 1.0
        + warm * hover.grow.or(theme.hover.grow).unwrap_or(0.0)
        + sunk * press.grow.or(theme.press.grow).unwrap_or(0.0);
    let opacity = style.map_or(1.0, |style| style.opacity);
    let mut color = tone.fill.unwrap_or(fill);
    for (mood, plain, share) in [(hover, &theme.hover, warm), (press, &theme.press, sunk)] {
        if let Some(wanted) = color_of(theme, &mood.fill).or_else(|| color_of(theme, &plain.fill)) {
            color = nalgebra_glm::lerp(&color, &wanted, share.clamp(0.0, 1.0));
        }
    }
    let color = raise(
        Vec4::new(color.x, color.y, color.z, color.w * opacity),
        lift,
    );
    let edge = tone
        .edge
        .or_else(|| get::<Edge>(rows, entity).map(|held| held.0))
        .map_or([0.0; 4], |held| [held.x, held.y, held.z, held.w * opacity]);
    let shadow = style
        .and_then(|style| color_of(theme, &style.shadow))
        .map_or([0.0; 4], |held| [held.x, held.y, held.z, held.w * opacity]);
    let inset = get::<Framed>(rows, entity)
        .filter(|framed| framed.shaped)
        .map_or(0.0, |framed| {
            framed
                .corner
                .min(rect.size.x * 0.5 * swell)
                .min(rect.size.y * 0.5 * swell)
        });
    quads.push(Quad {
        center: [rect.center.x, rect.center.y],
        size: [
            rect.size.x * 0.5 * swell - inset,
            rect.size.y * 0.5 * swell - inset,
        ],
        color,
        edge,
        shape: [
            style.map_or(0.0, |style| style.round),
            style.map_or(0.0, |style| style.border),
            0.0,
        ],
        clip: get::<Cut>(rows, entity).map_or([0.0; 4], |held| held.0),
        picture: [
            get::<Picture>(rows, entity).map_or(NO_PICTURE, |held| held.0.0 as u32),
            reader(get::<Sharp>(rows, entity).is_some()),
            0,
            0,
        ],
        uv: get::<Region>(rows, entity).copied().unwrap_or_default().0,
        shadow,
        blur: style.map_or([0.0; 4], |style| style.cast),
        effect: {
            let mut effect = get::<Effect>(rows, entity).map_or([0.0; 4], |held| held.0);
            effect[3] = get::<Turn>(rows, entity).map_or(0.0, |held| held.0);
            effect
        },
        depth: get::<Deep>(rows, entity).map_or(0.0, |deep| deep.0),
    });
    Some((color.into(), swell))
}

pub(crate) fn letter(rows: &Storage, glyphs: &mut Glyphs, entity: Entity, quads: &mut Vec<Quad>) {
    let (Some(label), Some(anchor), Some(height), Some(ink)) = (
        get::<Label>(rows, entity),
        get::<Lettered>(rows, entity),
        get::<Height>(rows, entity),
        get::<Ink>(rows, entity),
    ) else {
        return;
    };
    let family = get::<Family>(rows, entity).map_or(String::new(), |held| held.0.clone());
    let text = match get::<Trim>(rows, entity).map_or(0.0, |held| held.0) {
        room if room > 0.0 => trimmed(glyphs, &family, &label.0, height.0, room),
        _ => label.0.clone(),
    };
    paint_wording(
        quads,
        glyphs,
        Wording {
            text,
            family,
            anchor: [anchor.0.x, anchor.0.y],
            height: height.0,
            ink: [ink.0.x, ink.0.y, ink.0.z, ink.0.w],
            clip: get::<Cut>(rows, entity).map_or([0.0; 4], |held| held.0),
            depth: get::<Deep>(rows, entity).map_or(0.0, |held| held.0),
            room: get::<Wrap>(rows, entity).map_or(0.0, |held| held.0),
            rim: get::<Rim>(rows, entity).copied().unwrap_or_default(),
            soft: get::<Soft>(rows, entity).map_or(0.0, |held| held.0),
            align: get::<Aligned>(rows, entity).map_or(Align::Start, |held| held.0),
        },
    );
}

pub(crate) fn frame_the_edges(
    rows: &Storage,
    entity: Entity,
    rect: Rect,
    framed: Framed,
    depth: f32,
    quads: &mut Vec<Quad>,
) {
    let band = 1.0 / SLICES;
    let corner = framed.corner.min(rect.size.x * 0.5).min(rect.size.y * 0.5);
    let left = rect.center.x - rect.size.x * 0.5;
    let top = rect.center.y + rect.size.y * 0.5;
    let spans = [
        (0.0, corner),
        (corner, rect.size.x - corner * 2.0),
        (rect.size.x - corner, corner),
    ];
    let rises = [
        (0.0, corner),
        (corner, rect.size.y - corner * 2.0),
        (rect.size.y - corner, corner),
    ];
    let clip = get::<Cut>(rows, entity).map_or([0.0; 4], |held| held.0);
    for (row, (down, tall)) in rises.iter().enumerate() {
        for (column, (across, wide)) in spans.iter().enumerate() {
            if row == 1 && column == 1 {
                continue;
            }
            quads.push(Quad {
                center: [left + across + wide * 0.5, top - down - tall * 0.5],
                size: [wide * 0.5, tall * 0.5],
                color: framed.tint.into(),
                clip,
                picture: [framed.picture.0 as u32, reader(framed.sharp), 0, 0],
                uv: [
                    column as f32 * band,
                    row as f32 * band,
                    (column + 1) as f32 * band,
                    (row + 1) as f32 * band,
                ],
                depth,
                ..Quad::default()
            });
        }
    }
}

pub(crate) fn paint_host(
    rows: &Storage,
    glyphs: &mut Glyphs,
    held: &HostLaid,
    theme: &Theme,
) -> Vec<Quad> {
    let mut made: Vec<Quad> = Vec::new();
    let mut open: Vec<(usize, Entity, Rect, Framed)> = Vec::new();
    for (step, entity) in held.order.iter().copied().enumerate() {
        close_frames(rows, held, &mut open, step, &mut made);
        let Some(rect) = get::<Rect>(rows, entity).copied() else {
            continue;
        };
        let shade = fill(rows, entity, rect, theme, &mut made);
        let framed = get::<Framed>(rows, entity).copied();
        if let Some(framed) = framed.filter(|framed| framed.shaped)
            && let Some((tint, swell)) = shade
        {
            let depth = get::<Deep>(rows, entity).map_or(0.0, |deep| deep.0);
            let grown = Rect {
                size: rect.size * swell,
                ..rect
            };
            frame_the_edges(
                rows,
                entity,
                grown,
                Framed { tint, ..framed },
                depth,
                &mut made,
            );
        }
        letter(rows, glyphs, entity, &mut made);
        if let Some(framed) = framed.filter(|framed| !framed.shaped) {
            open.push((held.ends[step], entity, rect, framed));
        }
    }
    close_frames(rows, held, &mut open, held.order.len(), &mut made);
    made
}
