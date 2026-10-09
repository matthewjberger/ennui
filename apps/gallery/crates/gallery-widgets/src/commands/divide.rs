use ennui::prelude::{Entity, Mut, peek_mut};
use ennui_ui::prelude::{Panel, Span, heighten, widen};

pub(crate) fn span_of(panels: &mut Mut<(Panel,)>, entity: Entity, down: bool) -> Option<Span> {
    peek_mut(panels, entity).map(|(panel,)| match down {
        true => panel.tall,
        false => panel.wide,
    })
}

pub(crate) fn set_span(panels: &mut Mut<(Panel,)>, entity: Entity, down: bool, share: f32) {
    let Some((mut panel,)) = peek_mut(panels, entity) else {
        return;
    };
    match down {
        true => heighten(&mut panel, Span::Fill(share)),
        false => widen(&mut panel, Span::Fill(share)),
    }
}
