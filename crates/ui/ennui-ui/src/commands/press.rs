use crate::commands::renew::renew;
use crate::components::{Click, Hover, Poke, Press, Scroll};
use crate::data::{Found, Scrolled, Touched};
use crate::queries::press::reaches;
use crate::resources::Laid;
use ennui::prelude::{Entity, each_mut};

pub(crate) fn mark_presses(
    touched: &mut Touched<'_, '_>,
    found: &[Found],
    laid: &Laid,
    (front, top): ((u32, usize), Option<usize>),
    began: bool,
    down: bool,
) {
    let mut index = 0;
    each_mut(
        touched,
        |entity, (mut hover, mut press, mut click, mut poke), _| {
            let place = index;
            index += 1;
            let Some(&(_, inside, held, seat, _, step)) =
                found.get(place).filter(|found| found.0 == entity)
            else {
                return;
            };
            let reached = inside && seat == front && reaches(laid, seat.1, step, top);
            let poked = poke.0;
            *hover = Hover(reached);
            *press = Press((reached && began) || (held && down));
            *click = Click(poked || (reached && held && !down));
            if poked {
                *poke = Poke(false);
            }
        },
    );
}

pub(crate) fn wheel(scrolled: &mut Scrolled<'_, '_>, target: Entity, step: f32) {
    each_mut(scrolled, |entity, (mut scroll,), (_, reach, _, _, _)| {
        let Some(reach) = reach.filter(|_| entity == target) else {
            return;
        };
        let wanted = Scroll((scroll.0 - step).clamp(0.0, reach.0.max(0.0)));
        renew(&mut scroll, wanted);
    });
}
