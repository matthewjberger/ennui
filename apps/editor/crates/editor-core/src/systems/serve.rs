use crate::commands::serve::{accept, open_editor, read_all, request_of};
use crate::resources::{Editor, Opening, Server};
use ennui::prelude::{Later, Res, ResMut};
use ennui::reflect::prelude::{Reflected, Settings};
use ennui_document::prelude::read_outsiders;
use ennui_document::prelude::{Level, Outsiders, Placed, Scenery};
use ennui_platform::prelude::Shelf;
use std::io::Write;

pub(crate) fn open(
    mut later: Later,
    mut editor: ResMut<Editor>,
    mut server: ResMut<Server>,
    opening: Res<Opening>,
    mut placed: ResMut<Placed>,
    registry: Res<Reflected>,
    mut outsiders: ResMut<Outsiders>,
    mut settings: ResMut<Settings>,
    mut level: ResMut<Level>,
    shelf: Res<Shelf>,
) {
    level.root.clone_from(&opening.root);
    if let Err(problem) = read_outsiders(&mut outsiders, &registry, &opening.root) {
        editor.problems.push(problem);
    }
    let mut scenery = Scenery {
        placed: &mut placed,
        registry: &registry,
        outsiders: &outsiders,
        settings: &mut settings,
        shelf: &shelf,
    };
    open_editor(
        &mut later,
        &mut scenery,
        (&mut editor, &mut server),
        &opening,
    );
}

pub(crate) fn serve(mut server: ResMut<Server>, mut editor: ResMut<Editor>) {
    for (id, text) in std::mem::take(&mut editor.answers) {
        let Some(place) = server.asked.iter().position(|asked| asked.id == id) else {
            continue;
        };
        let mut asked = server.asked.remove(place);
        let _ = asked.stream.set_nonblocking(false);
        let _ = asked.stream.write_all(text.as_bytes());
        let _ = asked.stream.flush();
    }
    accept(&mut server);
    for asked in server.asked.iter_mut().filter(|asked| !asked.read) {
        read_all(asked);
    }
    for asked in server
        .asked
        .iter_mut()
        .filter(|asked| asked.read && !asked.sent)
    {
        asked.sent = true;
        let request = request_of(asked);
        match request.lines.is_empty() {
            true => asked.failed = true,
            false => editor.requests.push_back(request),
        }
    }
    server.asked.retain(|asked| !asked.failed);
}
