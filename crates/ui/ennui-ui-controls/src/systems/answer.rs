use crate::components::{Answer, Asked, Close, Deny, Grant};
use ennui::prelude::{Mut, Peek, each_mut};
use ennui_ui::prelude::{Click, Hidden, clicked};

pub(crate) fn read_answers(mut asked: Mut<(Asked, Hidden), (&Grant, &Deny)>, clicks: Peek<Click>) {
    each_mut(&mut asked, |_, (mut held, mut hidden), (grant, deny)| {
        if hidden.0 {
            return;
        }
        let wanted = match (clicked(&clicks, grant.0), clicked(&clicks, deny.0)) {
            (true, _) => Answer::Granted,
            (_, true) => Answer::Denied,
            _ => {
                if held.0 != Answer::Waiting {
                    *held = Asked(Answer::Waiting);
                }
                return;
            }
        };
        *held = Asked(wanted);
        *hidden = Hidden(true);
    });
}

pub(crate) fn shut_modals(mut shut: Mut<(Hidden,), (&Close,)>, clicks: Peek<Click>) {
    each_mut(&mut shut, |_, (mut hidden,), (close,)| {
        if !hidden.0 && clicked(&clicks, close.0) {
            *hidden = Hidden(true);
        }
    });
}
