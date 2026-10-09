use crate::commands::{drain, follow, settle};
use crate::resources::{Changes, Watch, Watching};
use ennui::prelude::{Res, ResMut, Steady};

pub(crate) fn gather(
    steady: Res<Steady>,
    watch: Res<Watch>,
    mut watching: ResMut<Watching>,
    mut changes: ResMut<Changes>,
) {
    changes.frame += 1;
    if steady.0 {
        return;
    }
    if watching.seen != watch.folders {
        watching.seen.clone_from(&watch.folders);
        follow(&mut watching, &mut changes, &watch.folders);
    }
    drain(&mut watching, &mut changes);
    settle(&mut watching, &mut changes);
}
