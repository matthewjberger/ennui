use crate::resources::{FileOpened, Files};
use ennui::events::send;
use ennui::prelude::{Events, Res, ResMut};

pub(crate) fn deliver_files(files: Res<Files>, mut opened: ResMut<Events<FileOpened>>) {
    let Ok(heard) = files.heard.lock() else {
        return;
    };
    for held in heard.try_iter() {
        send(&mut opened, held);
    }
}
