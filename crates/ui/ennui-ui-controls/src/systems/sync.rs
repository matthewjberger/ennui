use crate::commands::build::{put_dropdown, put_field};
use crate::commands::show::show_tab;
use crate::commands::skin::offer_rows;
use crate::components::{
    Band, Button, Chose, Dropdown, Entry, Field, Ghost, Listing, Offered, Page, Room, Slid, Slide,
    Slider, Tab, Tabs, Turned, Wrote,
};
use crate::data::Ask;
use crate::queries::skin::{room_of, served_above, share_at, value_at, worn_above};
use crate::resources::Asks;
use ennui::later::set;
use ennui::prelude::{
    Entity, Glance, Later, Mut, Peek, Res, ResMut, View, each, each_mut, peek_mut,
};
use ennui_scene::prelude::ChildOf;
use ennui_screens::prelude::Served;
use ennui_text::prelude::Label;
use ennui_ui::prelude::{Click, Hidden, Hosted, Lit, Theme, Wearing, renew};

pub(crate) fn ask_buttons(
    buttons: View<(&Button, &Click, Option<&Hosted>)>,
    parents: Peek<ChildOf>,
    served: Peek<Served>,
    mut asks: ResMut<Asks>,
) {
    asks.list.clear();
    for (entity, (button, click, hosted)) in each(&buttons) {
        if !click.0 || button.ask.is_empty() {
            continue;
        }
        asks.list.push(Ask {
            name: button.ask.clone(),
            host: hosted.map(|held| held.0),
            instance: served_above(&parents, &served, entity),
        });
    }
}

pub(crate) fn sync_sliders(mut sliders: Mut<(Slider, Slide, Slid)>) {
    each_mut(&mut sliders, |_, (mut slider, mut slide, mut slid), ()| {
        if slide.0 != slid.0 {
            let snapped = Slider {
                value: value_at(&slider, slide.0),
                ..*slider
            };
            renew(&mut slider, snapped);
        }
        let share = share_at(&slider);
        renew(&mut slide, Slide(share));
        renew(&mut slid, Slid(share));
    });
}

pub(crate) fn sync_entries(
    mut entries: Mut<(Entry, Wrote, Room), (&Field, Option<&Ghost>)>,
    mut labels: Mut<(Label,)>,
    mut later: Later,
) {
    each_mut(
        &mut entries,
        |entity, (mut entry, mut wrote, mut room), (field, ghost)| {
            if field.0 != wrote.0 {
                let typed = Entry {
                    words: field.0.clone(),
                    ..entry.clone()
                };
                renew(&mut entry, typed);
                renew(&mut wrote, Wrote(field.0.clone()));
            } else if entry.words != wrote.0 {
                put_field(&mut later, entity, &entry.words);
                renew(&mut wrote, Wrote(entry.words.clone()));
            }
            let wanted = room_of(entry.most);
            if room.0 != wanted {
                *room = Room(wanted);
            }
            if let Some(ghost) = ghost
                && let Some((mut label,)) = peek_mut(&mut labels, ghost.0)
            {
                renew(&mut label, Label(entry.hint.clone()));
            }
        },
    );
}

pub(crate) fn sync_dropdowns(
    rows: Glance,
    drops: View<(&Dropdown, &Offered, &Chose, &Listing)>,
    look: Res<Theme>,
    wearing: Res<Wearing>,
    mut later: Later,
) {
    for (entity, (dropdown, offered, chose, listing)) in each(&drops) {
        if chose.0 != offered.chosen {
            set(
                &mut later,
                entity,
                Dropdown {
                    chosen: chose.0,
                    ..dropdown.clone()
                },
            );
            set(
                &mut later,
                entity,
                Offered {
                    chosen: chose.0,
                    options: offered.options.clone(),
                },
            );
            continue;
        }
        if dropdown.options == offered.options && dropdown.chosen == offered.chosen {
            continue;
        }
        if dropdown.options != offered.options {
            let worn = worn_above(&rows, (&look, &wearing), entity);
            offer_rows(
                &mut later,
                worn,
                entity,
                listing.0,
                &dropdown.options,
                dropdown.chosen,
            );
        }
        put_dropdown(&mut later, entity, Some(dropdown.chosen));
        set(
            &mut later,
            entity,
            Offered {
                options: dropdown.options.clone(),
                chosen: dropdown.chosen,
            },
        );
    }
}

pub(crate) fn sync_tabs(
    mut tabs: Mut<(Tabs, Turned, Chose)>,
    mut buttons: Mut<(Lit,), (&Tab, &Band)>,
    mut pages: Mut<(Hidden,), (&Page, &Band)>,
) {
    let mut shown: Vec<(Entity, usize)> = Vec::new();
    each_mut(
        &mut tabs,
        |holder, (mut tabs, mut turned, mut chose), ()| {
            if chose.0 != turned.0 {
                renew(&mut tabs, Tabs { chosen: chose.0 });
                renew(&mut turned, Turned(chose.0));
            } else if tabs.chosen != turned.0 {
                renew(&mut chose, Chose(tabs.chosen));
                renew(&mut turned, Turned(tabs.chosen));
                shown.push((holder, tabs.chosen));
            }
        },
    );
    for (holder, index) in shown {
        show_tab(&mut buttons, &mut pages, holder, index);
    }
}
