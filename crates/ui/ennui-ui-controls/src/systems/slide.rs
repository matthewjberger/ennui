use crate::components::{Grip, Knob, Rest, Slide, Slider};
use crate::data::Slides;
use crate::theme::{LEAST_FILL, SLIDER_STEPS};
use ennui::later::{remove, set};
use ennui::prelude::{Later, Mut, Peek, Res, View, each, each_mut, peek, peek_mut};
use ennui_ui::prelude::{Hosts, Panel, Rect, Span, pointer_in, renew, widen};
use ennui_ui_focus::prelude::{Walked, Way};

pub(crate) fn step_slides(walked: Res<Walked>, mut sliders: Mut<(Slider,)>) {
    let (Some(at), Some(way)) = (walked.at, walked.turned) else {
        return;
    };
    let Some((mut slider,)) = peek_mut(&mut sliders, at) else {
        return;
    };
    let step = match slider.step > 0.0 {
        true => slider.step,
        false => (slider.high - slider.low) / SLIDER_STEPS,
    };
    let signed = match way {
        Way::Left => -step,
        _ => step,
    };
    let held = *slider;
    let value = (held.value + signed).clamp(held.low.min(held.high), held.high.max(held.low));
    renew(&mut slider, Slider { value, ..held });
}

pub(crate) fn drag_slides(
    mut slides: Slides<'_>,
    rects: Peek<Rect>,
    hosts: Res<Hosts>,
    mut later: Later,
) {
    each_mut(
        &mut slides,
        |entity, (mut slide,), (track, press, hosted, grip)| {
            if !press.0 {
                if grip.is_some() {
                    remove::<Grip>(&mut later, entity);
                }
                return;
            }
            let Some(hosting) = hosts
                .list
                .iter()
                .find(|held| hosted.is_none_or(|hosted| held.host == hosted.0))
            else {
                return;
            };
            let at = pointer_in(&hosts, hosted);
            let x = at.x * hosting.scale + hosting.offset.x;
            let share = match grip {
                Some(grip) => grip.share + (x - grip.x) / grip.width,
                None => {
                    let Some(rect) = peek(&rects, track.0) else {
                        return;
                    };
                    let share = ((at.x - rect.center.x) / rect.size.x.max(LEAST_FILL) + 0.5)
                        .clamp(0.0, 1.0);
                    let width = (rect.size.x * hosting.scale).max(LEAST_FILL);
                    set(&mut later, entity, Grip { share, x, width });
                    share
                }
            };
            renew(&mut slide, Slide(share.clamp(0.0, 1.0)));
        },
    );
}
pub(crate) fn fill_slides(slides: View<(&Slide, &Knob, &Rest)>, mut panels: Mut<(Panel,)>) {
    for (_, (slide, knob, rest)) in each(&slides) {
        let share = slide.0.clamp(0.0, 1.0);
        if let Some((mut stamp,)) = peek_mut(&mut panels, knob.0) {
            widen(&mut stamp, Span::Fill(share.max(LEAST_FILL)));
        }
        if let Some((mut stamp,)) = peek_mut(&mut panels, rest.0) {
            widen(&mut stamp, Span::Fill((1.0 - share).max(LEAST_FILL)));
        }
    }
}
