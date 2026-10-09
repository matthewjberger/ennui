use crate::components::{Hosted, Style, Text};
use crate::data::{Dye, Mood, Resolved, Size};
use crate::resources::{Hosts, Theme};
use crate::theme::THEME_COMPONENT;
use ennui::prelude::{Entity, Peek, peek};
use ennui::reflect::prelude::Value;
use ennui_document::prelude::{Document, Placed, parent_of};
use nalgebra_glm::Vec4;
use std::collections::HashMap;

pub(crate) fn color_of(theme: &Theme, dye: &Dye) -> Option<Vec4> {
    Some(match dye {
        Dye::Keep => return None,
        Dye::Ink => theme.ink,
        Dye::Faint => theme.faint,
        Dye::Accented => theme.accented,
        Dye::Ground => theme.ground,
        Dye::Panel => theme.panel,
        Dye::Header => theme.header,
        Dye::Edge => theme.edge,
        Dye::Accent => theme.accent,
        Dye::Good => theme.good,
        Dye::Warn => theme.warn,
        Dye::Bad => theme.bad,
        Dye::Input => theme.input,
        Dye::Chosen => theme.chosen,
        Dye::Scrollbar => theme.scrollbar,
        Dye::Shade => theme.shade,
        Dye::DeepShade => theme.deep_shade,
        Dye::Named(name) => theme.named.iter().find(|held| held.name == *name)?.color,
        Dye::Color(color) => *color,
    })
}

pub(crate) fn size_of(theme: &Theme, size: Size) -> f32 {
    match size {
        Size::Text => theme.text,
        Size::Heading => theme.heading,
        Size::Caption => theme.caption,
        Size::Fixed(height) => height,
    }
}

pub(crate) fn theme_of<'held>(
    themes: &'held Peek<'held, Theme>,
    fallback: &'held Theme,
    named: Option<Entity>,
) -> &'held Theme {
    named
        .and_then(|entity| peek(themes, entity))
        .unwrap_or(fallback)
}

pub fn worn_theme<'held>(
    themes: &'held Peek<'held, Theme>,
    fallback: &'held Theme,
    hosts: &Hosts,
    hosted: Option<&Hosted>,
) -> &'held Theme {
    let named = hosted.and_then(|hosted| {
        hosts
            .list
            .iter()
            .find(|held| held.host == hosted.0)
            .and_then(|held| held.theme)
    });
    theme_of(themes, fallback, named)
}

pub(crate) fn themed_root(
    document: &Document,
    records: &HashMap<u32, Vec<(String, Value)>>,
    placed: &Placed,
) -> Option<Entity> {
    (0..document.rows.len())
        .filter(|place| parent_of(document, *place).is_none())
        .find(|place| {
            records.get(&document.rows[*place].id).is_some_and(|held| {
                held.iter()
                    .any(|(component, _)| component == THEME_COMPONENT)
            })
        })
        .and_then(|place| {
            let id = document.names.list.get(document.rows[place].id as usize)?;
            placed.entities.get(id).copied()
        })
}

pub(crate) fn dressed(
    theme: &Theme,
    style: &Style,
    text: Option<&Text>,
    (off, lit, focus): (bool, bool, bool),
) -> Resolved {
    let mut fill = color_of(theme, &style.fill);
    let mut edge = color_of(theme, &style.edge);
    let mut ink = text.and_then(|text| color_of(theme, &text.color));
    let moods: [(bool, &Mood, &Mood); 3] = [
        (off, &style.off, &theme.off),
        (lit, &style.lit, &theme.lit),
        (focus, &style.focus, &theme.focus),
    ];
    for (on, own, fallback) in moods {
        if !on {
            continue;
        }
        let pick = |mine: &Dye, theirs: &Dye| match mine {
            Dye::Keep => color_of(theme, theirs),
            held => color_of(theme, held),
        };
        fill = pick(&own.fill, &fallback.fill).or(fill);
        edge = pick(&own.edge, &fallback.edge).or(edge);
        ink = pick(&own.ink, &fallback.ink).or(ink);
    }
    let height = text.map(|text| size_of(theme, text.size));
    (fill, edge, ink, height)
}
