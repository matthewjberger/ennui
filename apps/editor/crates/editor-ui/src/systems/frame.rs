use crate::queries::ui_scene;
use crate::resources::Designer;
use crate::theme::{FRAME_COLOR, FRAME_FILL, FRAME_INSET, FRAME_LINE, FRAME_TOP};
use editor_core::prelude::{Editor, Viewing};
use ennui::later::{attach, set_if_new};
use ennui::prelude::{Glance, Later, Res, ResMut, View, each};
use ennui::storage::get;
use ennui_ui::prelude::{Dye, Float, Frame, Hidden, Rect, Span, Theme, afloat, frame, relay};
use nalgebra_glm::Vec2;

pub(crate) fn frame_view(
    seen: Glance,
    viewing: View<(&Viewing,)>,
    mut later: Later,
    look: Res<Theme>,
    mut editor: ResMut<Editor>,
    mut designer: ResMut<Designer>,
) {
    let Some(room) = each(&viewing).next().map(|(entity, _)| entity) else {
        return;
    };
    let held = match designer.frame {
        Some(held) => held,
        None => {
            let held = frame(
                &mut later,
                Frame::new(&look)
                    .wide(Span::Fixed(0.0))
                    .tall(Span::Fixed(0.0))
                    .fill(FRAME_FILL)
                    .border(FRAME_LINE)
                    .rim(Dye::Color(FRAME_COLOR))
                    .round(0.0)
                    .pad(0.0)
                    .gap(0.0),
            );
            afloat(&mut later, room, held);
            attach(&mut later, held, (Hidden(true),));
            designer.frame = Some(held);
            return;
        }
    };
    let aspect = editor.designing.aspect.filter(|_| ui_scene(&editor));
    let (Some(aspect), Some(pane)) = (aspect, get::<Rect>(&seen, room)) else {
        editor.designing.frame = None;
        set_if_new(&mut later, held, Hidden(true));
        return;
    };
    let room_size = Vec2::new(
        (pane.size.x - FRAME_INSET * 2.0).max(1.0),
        (pane.size.y - FRAME_INSET * 2.0 - FRAME_TOP).max(1.0),
    );
    let ratio = aspect[0] / aspect[1];
    let size = match room_size.x / room_size.y > ratio {
        true => Vec2::new(room_size.y * ratio, room_size.y),
        false => Vec2::new(room_size.x, room_size.x / ratio),
    };
    let middle = Vec2::new(pane.center.x, pane.center.y - FRAME_TOP * 0.5);
    let corner = middle + Vec2::new(-size.x, size.y) * 0.5;
    relay(&mut later, held, move |panel| {
        panel.wide = Span::Fixed(size.x);
        panel.tall = Span::Fixed(size.y);
    });
    set_if_new(&mut later, held, Float(corner));
    set_if_new(&mut later, held, Hidden(false));
    editor.designing.frame = Some(held);
}
