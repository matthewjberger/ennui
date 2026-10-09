use crate::components::{Hosted, Text};
use crate::queries::theme::{color_of, worn_theme};
use crate::resources::{Fonts, Hosts, Said, Theme};
use ennui::later::{set, set_if_new};
use ennui::prelude::{Glance, Later, Peek, Res, ResMut};
use ennui::storage::{changed_since, get, tick, touched_since};
use ennui_text::prelude::{Aligned, Family, Label, Rim, Soft};

pub(crate) fn write_texts(
    rows: Glance,
    themes: Peek<Theme>,
    theme: Res<Theme>,
    hosts: Res<Hosts>,
    fonts: Res<Fonts>,
    mut said: ResMut<Said>,
    mut later: Later,
) {
    let since = said.tick;
    said.tick = tick(&rows);
    if !touched_since::<Text>(&rows, since) {
        return;
    }
    for entity in changed_since::<Text>(&rows, since) {
        let Some(text) = get::<Text>(&rows, entity) else {
            continue;
        };
        set_if_new(&mut later, entity, Label(text.words.clone()));
        set_if_new(&mut later, entity, Aligned(text.align));
        set_if_new(&mut later, entity, Soft(text.softness));
        let worn = worn_theme(&themes, &theme, &hosts, get::<Hosted>(&rows, entity));
        set_if_new(
            &mut later,
            entity,
            Rim {
                width: text.outline,
                color: color_of(worn, &text.outline_color).unwrap_or_default(),
            },
        );
        let path = match text.font.is_empty() {
            true => &worn.font,
            false => &text.font,
        };
        let family = fonts
            .held
            .get(path)
            .and_then(|held| held.clone())
            .unwrap_or_default();
        set(&mut later, entity, Family(family));
    }
}
