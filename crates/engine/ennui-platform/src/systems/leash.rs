use crate::resources::{Exit, Leash, Told};
use ennui::prelude::{Res, ResMut};

pub(crate) fn follow_the_leash(leash: Res<Leash>, mut told: ResMut<Told>, mut exit: ResMut<Exit>) {
    told.0 = leash
        .lines
        .as_ref()
        .and_then(|lines| lines.lock().ok().map(|lines| lines.try_iter().collect()))
        .unwrap_or_default();
    if leash
        .reading
        .as_ref()
        .is_some_and(|reading| reading.is_finished())
    {
        exit.0 = true;
    }
}
