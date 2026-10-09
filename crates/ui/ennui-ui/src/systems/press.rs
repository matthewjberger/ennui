use crate::commands::press::{mark_presses, wheel};
use crate::data::{Filled, Overlaid, Scrolled, Touched};
use crate::queries::press::{
    filled_under, found_under, front_of, reaches, screen_from_pointer, wheeled_under,
};
use crate::queries::theme::theme_of;
use crate::resources::{Hosts, Laid, Theme};
use crate::theme::FAR_POINTER;
use ennui::prelude::{Peek, Res, ResMut};
use ennui_platform::prelude::{
    Claimed, CursorIcon, Input, MouseButton, Pointing, Viewport, set_claim,
};
use nalgebra_glm::Vec2;

pub(crate) fn read_presses(
    mut touched: Touched<'_, '_>,
    mut scrolled: Scrolled<'_, '_>,
    filled: Filled<'_, '_>,
    themes: Peek<Theme>,
    theme: Res<Theme>,
    laid: Res<Laid>,
    input: Res<Input>,
    viewport: Res<Viewport>,
    mut hosts: ResMut<Hosts>,
    mut claimed: ResMut<Claimed>,
    mut pointing: ResMut<Pointing>,
) {
    let at = screen_from_pointer(viewport.width, viewport.height, input.pointer);
    for held in hosts.list.iter_mut() {
        held.pointer = match held.scale > 0.0 {
            true => (at - held.offset) / held.scale,
            false => Vec2::repeat(FAR_POINTER),
        };
    }
    let down = input.buttons_held.contains(&MouseButton::Left);
    let began = input.buttons_pressed.contains(&MouseButton::Left);
    let found = found_under(&mut touched, &hosts);
    let (wheeled, covered) = wheeled_under(&mut scrolled, &hosts);
    let over = covered
        || filled_under(&filled, &hosts)
        || found
            .iter()
            .any(|(_, inside, held, _, _, _)| *inside || *held);
    set_claim::<Overlaid>(&mut claimed.pointer, over);
    let (front, top) = front_of(&found);
    let wanted = found
        .iter()
        .filter(|(_, inside, _, seat, _, step)| {
            *inside && *seat == front && reaches(&laid, seat.1, *step, top)
        })
        .max_by_key(|(_, _, _, _, _, step)| *step)
        .and_then(|(_, _, _, _, cursor, _)| *cursor);
    pointing.0 = wanted.unwrap_or(CursorIcon::Default);
    mark_presses(&mut touched, &found, &laid, (front, top), began, down);
    if input.scroll == 0.0 && input.swiped == 0.0 {
        return;
    }
    let Some((index, target)) = wheeled else {
        return;
    };
    let worn = theme_of(&themes, &theme, hosts.list[index].theme);
    let step =
        input.scroll * worn.scroll_step + input.swiped / hosts.list[index].scale.max(f32::EPSILON);
    wheel(&mut scrolled, target, step);
}
