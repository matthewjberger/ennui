use crate::data::{Handle, UiDrag};
use crate::queries::ui::{
    chrome_scale, element_under, handle_rects, rect_on_screen, siblings_of, ui_element,
};
use crate::resources::{Designing, Editor, Shell};
use crate::theme::{ORDER, PANEL, PIN, UI_RIM_ORDER, UI_ROUNDING};
use editor_choose::prelude::{CHOSEN_COLOR, OVER_COLOR};
use editor_document::prelude::{Author, Change, commit, edit_leaf, user_layer};
use ennui::later::{attach, set_if_new};
use ennui::prelude::{Edits, Entity, Later, Storage};
use ennui::reflect::prelude::{Reflect, Value};
use ennui::storage::get;
use ennui_document::prelude::Placed;
use ennui_scene::prelude::ChildOf;
use ennui_ui::prelude::{
    Dye, Float, Frame, Hidden, Hosted, Hosting, Hosts, Lay, Order, Panel, Pin, Rect, Span, Theme,
    afloat, frame, host_point, pointer_of, relay,
};
use nalgebra_glm::{Vec2, Vec4};

fn rim_frame<'theme>(look: &'theme Theme, color: Vec4, filled: bool) -> Frame<'theme> {
    let held = Frame::new(look)
        .wide(Span::Fixed(0.0))
        .tall(Span::Fixed(0.0))
        .border(look.line)
        .rim(Dye::Color(color))
        .round(0.0)
        .pad(0.0)
        .gap(0.0);
    match filled {
        true => held.fill(color),
        false => held.bare(),
    }
}

pub(crate) fn lay_rims(later: &mut Later, look: &Theme, shell: &Shell, designing: &mut Designing) {
    let Some(over) = shell.pin_sheet else {
        return;
    };
    let mut make = |color: Vec4, filled: bool| {
        let held = frame(later, rim_frame(look, color, filled));
        afloat(later, over, held);
        attach(later, held, (Hidden(true), Order(UI_RIM_ORDER)));
        held
    };
    designing.rims = [
        Some(make(OVER_COLOR, false)),
        Some(make(CHOSEN_COLOR, false)),
    ];
    designing.handles = [0; 3].map(|_| Some(make(CHOSEN_COLOR, true)));
}

fn place_rim(
    edits: &mut Edits,
    held: Entity,
    rect: Option<Rect>,
    hosts: &Hosts,
    sheet: Entity,
    storage: &Storage,
) {
    let Some(rect) = rect else {
        set_if_new(edits, held, Hidden(true));
        return;
    };
    let host = get::<Hosted>(storage, sheet).map(|hosted| hosted.0);
    let (corner, size) = match host {
        Some(host) => (
            host_point(
                hosts,
                host,
                rect.center + Vec2::new(-rect.size.x, rect.size.y) * 0.5,
            ),
            rect.size / chrome_scale(hosts, storage, sheet),
        ),
        None => (rect.center, rect.size),
    };
    relay(edits, held, move |panel| {
        panel.wide = Span::Fixed(size.x);
        panel.tall = Span::Fixed(size.y);
    });
    set_if_new(edits, held, Float(corner));
    set_if_new(edits, held, Hidden(false));
}

pub(crate) fn show_rims(
    (storage, edits): (&Storage, &mut Edits),
    (hosts, placed): (&Hosts, &Placed),
    (shell, editor): (&Shell, &Editor),
) {
    let Some(sheet) = shell.pin_sheet else {
        return;
    };
    let designing = &editor.designing;
    let on_screen = |id: &String| {
        ui_element(storage, placed, id).and_then(|entity| rect_on_screen(storage, hosts, entity))
    };
    let chosen = editor
        .book
        .chosen
        .first()
        .filter(|_| editor.book.chosen.len() == 1)
        .and_then(on_screen);
    let hovered = shell
        .hovered
        .as_ref()
        .filter(|id| !editor.book.chosen.contains(id))
        .and_then(on_screen);
    for (held, rect) in designing.rims.iter().zip([hovered, chosen]) {
        if let Some(held) = held {
            place_rim(edits, *held, rect, hosts, sheet, storage);
        }
    }
    let scale = chrome_scale(hosts, storage, sheet);
    let handles = chosen.map(|rect| handle_rects(rect, scale));
    for (place, held) in designing.handles.iter().enumerate() {
        if let Some(held) = held {
            let rect = handles.map(|rects| rects[place].1);
            place_rim(edits, *held, rect, hosts, sheet, storage);
        }
    }
}

pub(crate) fn handle_under(
    storage: &Storage,
    hosts: &Hosts,
    (placed, shell, editor): (&Placed, &Shell, &Editor),
    at: Vec2,
) -> Option<(Handle, Entity, String)> {
    let [id] = editor.book.chosen.as_slice() else {
        return None;
    };
    let entity = ui_element(storage, placed, id)?;
    let rect = rect_on_screen(storage, hosts, entity)?;
    let scale = shell
        .pin_sheet
        .map_or(1.0, |sheet| chrome_scale(hosts, storage, sheet));
    handle_rects(rect, scale)
        .into_iter()
        .find(|(_, held)| ennui_ui::queries::press::inside_of(at, held))
        .map(|(handle, _)| (handle, entity, id.clone()))
}

fn user_change(editor: &mut Editor, label: String) -> Change {
    editor.book.pending.take().unwrap_or_else(|| Change {
        label,
        author: Author::User,
        ..Change::default()
    })
}

pub(crate) fn write_ui_leaf(
    editor: &mut Editor,
    change: &mut Change,
    id: &str,
    (component, path): (&str, &str),
    value: Value,
) {
    let Some(layer) = user_layer(&editor.book) else {
        return;
    };
    edit_leaf(
        &mut editor.book,
        change,
        layer,
        id,
        component,
        path,
        Some(value),
    );
}

fn rounded(value: f32) -> f64 {
    f64::from((value * UI_ROUNDING).round() / UI_ROUNDING)
}

pub(crate) fn carry_element(editor: &mut Editor, drag: &UiDrag, delta: Vec2) {
    let mut change = user_change(editor, format!("move {}", drag.id));
    match (drag.handle, drag.nudge) {
        (Handle::Move, Some(nudge)) => {
            let value = Value::List(vec![
                Value::Number(rounded(nudge[0] + delta.x)),
                Value::Number(rounded(nudge[1] - delta.y)),
            ]);
            write_ui_leaf(editor, &mut change, &drag.id, (PIN, "nudge"), value);
        }
        (Handle::Move, None) => {}
        (handle, _) => {
            change.label = format!("size {}", drag.id);
            let wide = (drag.size.x + delta.x).max(0.0);
            let tall = (drag.size.y - delta.y).max(0.0);
            if handle != Handle::Tall {
                let value = Span::value_of(&Span::Fixed(rounded(wide) as f32));
                write_ui_leaf(editor, &mut change, &drag.id, (PANEL, "wide"), value);
            }
            if handle != Handle::Wide {
                let value = Span::value_of(&Span::Fixed(rounded(tall) as f32));
                write_ui_leaf(editor, &mut change, &drag.id, (PANEL, "tall"), value);
            }
        }
    }
    editor.book.pending = Some(change);
    editor.book.stale = true;
}

pub(crate) fn reorder_element(editor: &mut Editor, id: &str, target: &str, before: bool) -> bool {
    let (parent, mut siblings) = siblings_of(&editor.book.composed, id);
    let (other, _) = siblings_of(&editor.book.composed, target);
    if parent != other || !siblings.iter().any(|held| held == target) {
        return false;
    }
    siblings.retain(|held| held != id);
    let Some(place) = siblings.iter().position(|held| held == target) else {
        return false;
    };
    siblings.insert(place + usize::from(!before), String::from(id));
    let mut change = user_change(editor, format!("reorder {id}"));
    for (order, sibling) in siblings.iter().enumerate() {
        let value = Order::value_of(&Order(order as u32));
        write_ui_leaf(editor, &mut change, sibling, (ORDER, ""), value);
    }
    editor.book.pending = Some(change);
    true
}

pub(crate) fn finish_drag(editor: &mut Editor) {
    let Some(change) = editor.book.pending.take() else {
        return;
    };
    let said = format!("the user made the change \"{}\"", change.label);
    commit(&mut editor.book, change);
    editor.happened.push(said);
    editor.book.stale = true;
}

pub(crate) fn order_lines(
    document: &ennui_document::prelude::Document,
    id: &str,
    target: &str,
    before: bool,
) -> Vec<String> {
    let (parent, mut siblings) = siblings_of(document, target);
    siblings.retain(|held| held != id);
    let Some(place) = siblings.iter().position(|held| held == target) else {
        return Vec::new();
    };
    siblings.insert(place + usize::from(!before), String::from(id));
    let (own, _) = siblings_of(document, id);
    let mut lines = Vec::new();
    if own != parent {
        let under = parent.as_deref().unwrap_or("none");
        lines.push(format!("parent {id} {under}"));
    }
    lines.extend(
        siblings
            .iter()
            .enumerate()
            .map(|(order, sibling)| format!("set {sibling} {ORDER} {order}")),
    );
    lines
}

pub(crate) fn begin_drag(
    storage: &Storage,
    hosts: &Hosts,
    (placed, shell, editable): (&Placed, &Shell, &[&Hosting]),
    editor: &Editor,
    (at, press): (Vec2, [f32; 2]),
) -> Option<UiDrag> {
    let (handle, entity, id) = match handle_under(storage, hosts, (placed, shell, editor), at) {
        Some(found) => found,
        None => element_under(storage, editable, placed, editor)
            .map(|(entity, id)| (Handle::Move, entity, id))?,
    };
    let host = get::<Hosted>(storage, entity)?.0;
    let rect = get::<Rect>(storage, entity).copied().unwrap_or_default();
    Some(UiDrag {
        id,
        entity,
        host,
        handle,
        from: pointer_of(hosts, host),
        press,
        nudge: get::<Pin>(storage, entity).map(|pin| pin.nudge),
        size: rect.size,
        moved: false,
    })
}

pub(crate) fn reorder_at(
    storage: &Storage,
    editable: &[&Hosting],
    placed: &Placed,
    editor: &mut Editor,
    drag: &UiDrag,
) {
    let Some((target, target_id)) = element_under(storage, editable, placed, editor) else {
        return;
    };
    let parent_of = |entity| get::<ChildOf>(storage, entity).map(|of| of.0);
    if target == drag.entity || parent_of(target) != parent_of(drag.entity) {
        return;
    }
    let (Some(rect), Some(pointer)) = (
        get::<Rect>(storage, target),
        editable
            .iter()
            .find(|held| held.host == drag.host)
            .map(|held| held.pointer),
    ) else {
        return;
    };
    let flow = parent_of(target)
        .and_then(|parent| get::<Panel>(storage, parent))
        .map_or(Lay::Column, |panel| panel.flow);
    let before = match flow {
        Lay::Column => pointer.y > rect.center.y,
        Lay::Row => pointer.x < rect.center.x,
    };
    reorder_element(editor, &drag.id, &target_id, before);
}
