use crate::components::Kept;
use crate::data::Way;
use crate::queries::{endpoint, first, keyed, neighbor};
use crate::resources::Walked;
use crate::theme::{FIRST_WAIT, NEXT_WAIT};
use ennui::later::set;
use ennui::prelude::{Entity, Later, Peek, peek};
use ennui_platform::prelude::{Claimed, Input, KeyCode, Time};
use ennui_ui::prelude::Rect;

pub(crate) fn steer(
    walked: &mut Walked,
    reachable: &[(Entity, Rect, Entity)],
    (chosen, turns): (Option<Entity>, bool),
    (input, time, claimed): (&Input, &Time, &Claimed),
) -> bool {
    if let Some(entity) = chosen {
        walked.at = Some(entity);
    }
    let asked = match walked.steered && claimed.keys.is_empty() {
        true => keyed(input),
        false => None,
    };
    let asked: Option<Way> = match asked {
        Some(way) if walked.held == Some(way) => {
            walked.waits -= time.since_last_frame;
            match walked.waits <= 0.0 {
                true => {
                    walked.waits = NEXT_WAIT;
                    Some(way)
                }
                false => None,
            }
        }
        Some(way) => {
            walked.held = Some(way);
            walked.waits = FIRST_WAIT;
            Some(way)
        }
        None => {
            walked.held = None;
            None
        }
    };
    walked.turned = asked.filter(|way| turns && matches!(way, Way::Left | Way::Right));
    let asked = asked.filter(|_| walked.turned.is_none());
    walked.stepped = asked.is_some();
    if let Some(way) = asked {
        let held = walked.at.and_then(|entity| {
            reachable
                .iter()
                .find(|(held, _, _)| *held == entity)
                .copied()
        });
        walked.at = match held {
            Some((entity, rect, _)) => {
                neighbor(reachable, entity, rect, way).or(endpoint(reachable, way))
            }
            None => first(reachable),
        };
    }
    walked.steered
        && input.pressed.contains(&KeyCode::Enter)
        && claimed.keys.is_empty()
        && walked.at.is_some()
}

pub(crate) fn switch_host(
    walked: &mut Walked,
    reachable: &[(Entity, Rect, Entity)],
    host: Option<Entity>,
    (kept, later): (&Peek<Kept>, &mut Later),
) {
    if walked.host == host {
        return;
    }
    let (was, at) = (walked.host, walked.at);
    if let Some(was) = was.filter(|_| at.is_some()) {
        set(later, was, Kept(at));
    }
    walked.host = host;
    let inside = |at: &Entity| reachable.iter().any(|(held, _, _)| held == at);
    walked.at = host
        .and_then(|host| peek(kept, host))
        .and_then(|held| held.0)
        .filter(inside)
        .or_else(|| host.and_then(|_| first(reachable)));
}
