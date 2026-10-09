use ennui::prelude::{Entity, Mut};
use ennui_text::prelude::Cut;
use ennui_ui::prelude::{Click, Hosted, Hover, Poke, Rect, Touch};

#[derive(Clone, Copy, PartialEq)]
pub enum Way {
    Up,
    Down,
    Left,
    Right,
}

pub(crate) type Reachable = (Vec<(Entity, Rect, Entity)>, Option<Entity>, Option<Entity>);

pub(crate) type Focusable<'world, 'row> = Mut<
    'world,
    (Poke,),
    (
        &'row Touch,
        &'row Rect,
        Option<&'row Click>,
        Option<&'row Hover>,
        Option<&'row Cut>,
        Option<&'row Hosted>,
    ),
>;
