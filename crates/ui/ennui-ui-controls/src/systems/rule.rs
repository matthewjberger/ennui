use crate::components::{Band, Ruled, Ruler};
use crate::theme::{RULE_LEAST, RULE_MOST};
use ennui::prelude::{Entity, Mut, Peek, Res, View, each, each_mut, peek, peek_mut};
use ennui_platform::prelude::Input;
use ennui_ui::prelude::{Hosted, Hosts, Panel, Press, Span, widen};

pub(crate) fn drag_rulers(
    rulers: View<(&Ruler, &Press)>,
    mut ruled: Mut<(Ruled,)>,
    mut cells: Mut<(Panel,), (&Band,)>,
    seats: Peek<Hosted>,
    hosts: Res<Hosts>,
    input: Res<Input>,
) {
    let motion = input.pointer_motion[0];
    if motion == 0.0 {
        return;
    }
    let held: Vec<Entity> = each(&rulers)
        .filter(|(_, (_, press))| press.0)
        .map(|(_, (ruler, _))| ruler.0)
        .collect();
    for ledger in held {
        let pixels = peek(&seats, ledger)
            .and_then(|seat| hosts.list.iter().find(|held| held.host == seat.0))
            .map_or(1.0, |held| held.pixels.max(f32::EPSILON));
        let across = motion / pixels;
        let Some((mut span,)) = peek_mut(&mut ruled, ledger) else {
            continue;
        };
        let wanted = (span.0 + across).clamp(RULE_LEAST, RULE_MOST);
        if wanted == span.0 {
            continue;
        }
        *span = Ruled(wanted);
        each_mut(&mut cells, |_, (mut panel,), (band,)| {
            if band.0 == ledger {
                widen(&mut panel, Span::Fixed(wanted));
            }
        });
    }
}
