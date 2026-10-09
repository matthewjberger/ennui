use crate::components::{
    Anchored, Cursor, Float, Hidden, Host, Kids, Order, Panel, Picture, Pin, Rect, Scroll, Shield,
    Style, Text, Thumb, Touch,
};
use crate::data::{Dye, Frame, Line, Mood, Size, Span};
use crate::resources::Theme;
use ennui::later::{attach, change, set, spawn};
use ennui::prelude::{Edits, Entity, Later};
use ennui::storage::{get, query};
use ennui_platform::prelude::CursorIcon;
use ennui_render::data::TextureId;
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::Wrap;
use nalgebra_glm::{Vec2, Vec4};

pub fn framed(held: &Frame<'_>) -> (Rect, Panel, Style) {
    (
        Rect::default(),
        Panel {
            wide: held.wide,
            tall: held.tall,
            flow: held.flow,
            along: held.along,
            across: held.across,
            pad: held.pad,
            gap: held.gap,
            ..Panel::default()
        },
        Style {
            fill: held.fill.clone(),
            edge: held.edge.clone(),
            border: held.border,
            round: held.round,
            shadow: held.shadow.clone(),
            cast: held.cast,
            lit: Mood {
                fill: held.on.clone(),
                ..Mood::default()
            },
            ..Style::default()
        },
    )
}

pub fn frame(later: &mut Later, held: Frame<'_>) -> Entity {
    spawn(later, framed(&held))
}

pub fn under(edits: &mut Edits, parent: Entity, child: Entity) -> Entity {
    change(edits, move |storage| {
        let order = get::<Kids>(&*storage, parent).map_or_else(
            || {
                query::<(&ChildOf, Option<&Order>)>(&*storage)
                    .filter(|(_, (of, _))| of.0 == parent)
                    .map(|(_, (_, order))| order.map_or(0, |held| held.0) + 1)
                    .max()
                    .unwrap_or(0)
            },
            |held| held.0,
        );
        ennui::storage::set(&mut *storage, parent, Kids(order + 1));
        ennui::storage::set(&mut *storage, child, ChildOf(parent));
        ennui::storage::set(&mut *storage, child, Order(order));
    });
    child
}

pub fn screen(later: &mut Later, theme: &Theme) -> Entity {
    sheet(later, theme, 0)
}

pub fn sheet(later: &mut Later, theme: &Theme, layer: u32) -> Entity {
    let entity = frame(
        later,
        Frame::new(theme)
            .wide(Span::Fill(1.0))
            .tall(Span::Fill(1.0))
            .bare()
            .pad(0.0),
    );
    set(
        later,
        entity,
        Host {
            layer,
            ..Host::default()
        },
    );
    entity
}

pub fn chrome(later: &mut Later, theme: &Theme, layer: u32) -> Entity {
    let entity = sheet(later, theme, layer);
    set(
        later,
        entity,
        Host {
            layer,
            scaled: false,
            ..Host::default()
        },
    );
    entity
}

pub fn panel(later: &mut Later, parent: Entity, held: Frame<'_>) -> Entity {
    let entity = frame(later, held);
    under(later, parent, entity);
    entity
}

pub fn pinned(later: &mut Later, parent: Entity, held: Frame<'_>, pin: Pin) -> Entity {
    let entity = panel(later, parent, held);
    attach(later, entity, (Float(Vec2::zeros()), pin));
    entity
}

pub fn label(later: &mut Later, theme: &Theme, parent: Entity, text: &str, height: f32) -> Entity {
    let entity = frame(later, Frame::new(theme).bare().pad(0.0).gap(0.0));
    set(
        later,
        entity,
        Text {
            words: String::from(text),
            size: Size::Fixed(height),
            ..Text::default()
        },
    );
    under(later, parent, entity);
    entity
}

pub fn wrapped(
    later: &mut Later,
    theme: &Theme,
    parent: Entity,
    text: &str,
    height: f32,
    room: f32,
) -> Entity {
    let entity = label(later, theme, parent, text, height);
    set(later, entity, Wrap(room));
    entity
}

pub fn shortened(
    later: &mut Later,
    theme: &Theme,
    parent: Entity,
    text: &str,
    height: f32,
) -> Entity {
    let entity = label(later, theme, parent, text, height);
    relay(later, entity, |panel| panel.shrinks = true);
    entity
}

pub fn image(
    later: &mut Later,
    theme: &Theme,
    parent: Entity,
    picture: TextureId,
    span: Span,
) -> Entity {
    let entity = frame(
        later,
        Frame::new(theme)
            .wide(span)
            .tall(span)
            .fill(Vec4::new(1.0, 1.0, 1.0, 1.0))
            .pad(0.0),
    );
    set(later, entity, Picture(picture));
    under(later, parent, entity);
    entity
}

pub fn scroll(later: &mut Later, theme: &Theme, parent: Entity, held: Frame<'_>) -> Entity {
    let entity = panel(later, parent, held);
    relay(later, entity, |panel| panel.scrolls = true);
    set(later, entity, Scroll(0.0));
    add_thumb(later, theme, entity);
    entity
}

pub(crate) fn add_thumb(later: &mut Later, theme: &Theme, entity: Entity) {
    let thumb = frame(
        later,
        Frame::new(theme)
            .wide(Span::Fixed(theme.bar_wide))
            .tall(Span::Fixed(theme.bar_wide))
            .fill(Dye::Scrollbar)
            .round(theme.bar_wide * 0.5)
            .pad(0.0),
    );
    under(later, entity, thumb);
    set(later, thumb, Order(u32::MAX));
    set(later, thumb, Float(Vec2::zeros()));
    set(later, entity, Thumb(thumb));
    touched(later, thumb);
}

pub fn separator(later: &mut Later, theme: &Theme, parent: Entity) -> Entity {
    panel(
        later,
        parent,
        Frame::new(theme)
            .wide(Span::Fill(1.0))
            .tall(Span::Fixed(theme.line))
            .fill(Dye::Edge)
            .round(0.0)
            .pad(0.0)
            .gap(0.0),
    )
}

pub fn spacing(later: &mut Later, theme: &Theme, parent: Entity, room: f32) -> Entity {
    panel(
        later,
        parent,
        Frame::new(theme)
            .wide(Span::Fixed(room))
            .tall(Span::Fixed(room))
            .bare()
            .pad(0.0)
            .gap(0.0),
    )
}

pub fn scroll_fill(later: &mut Later, theme: &Theme, parent: Entity) -> Entity {
    scroll(
        later,
        theme,
        parent,
        Frame::new(theme)
            .wide(Span::Fill(1.0))
            .tall(Span::Fill(1.0))
            .along(Line::Start)
            .bare()
            .pad(0.0),
    )
}

pub fn afloat(edits: &mut Edits, parent: Entity, entity: Entity) -> Entity {
    under(edits, parent, entity);
    set(edits, entity, Float(Vec2::zeros()));
    entity
}

pub fn floating(edits: &mut Edits, parent: Entity, entity: Entity, below: Entity) -> Entity {
    under(edits, parent, entity);
    attach(
        edits,
        entity,
        (Float(Vec2::zeros()), Anchored(below), Hidden(true)),
    );
    entity
}

pub fn touched(edits: &mut Edits, entity: Entity) -> Entity {
    attach(edits, entity, (Touch, Cursor(CursorIcon::Pointer)));
    entity
}

pub fn shielded(edits: &mut Edits, entity: Entity) -> Entity {
    attach(edits, entity, (Touch, Shield));
    entity
}

pub fn paint(edits: &mut Edits, entity: Entity, off: Dye, on: Dye) {
    change(edits, move |storage| {
        if let Some(mut style) = ennui::storage::get_mut::<Style>(&mut *storage, entity) {
            style.fill = off;
            style.lit.fill = on;
        }
    });
}

pub fn ink(edits: &mut Edits, entity: Entity, off: Dye, on: Dye) {
    change(edits, move |storage| {
        if let Some(mut text) = ennui::storage::get_mut::<Text>(&mut *storage, entity) {
            text.color = off;
        }
        if let Some(mut style) = ennui::storage::get_mut::<Style>(&mut *storage, entity) {
            style.lit.ink = on;
        }
    });
}

pub fn restyle(
    edits: &mut Edits,
    entity: Entity,
    change_style: impl FnOnce(&mut Style) + Send + 'static,
) {
    change(edits, move |storage| {
        let Some(mut style) = ennui::storage::get_mut::<Style>(&mut *storage, entity) else {
            return;
        };
        let mut wanted = style.clone();
        change_style(&mut wanted);
        if wanted != *style {
            *style = wanted;
        }
    });
}

pub fn relay(
    edits: &mut Edits,
    entity: Entity,
    change_panel: impl FnOnce(&mut Panel) + Send + 'static,
) {
    change(edits, move |storage| {
        let Some(mut panel) = ennui::storage::get_mut::<Panel>(&mut *storage, entity) else {
            return;
        };
        let mut wanted = *panel;
        change_panel(&mut wanted);
        if wanted != *panel {
            *panel = wanted;
        }
    });
}
