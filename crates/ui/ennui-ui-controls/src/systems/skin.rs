use crate::commands::skin::{
    skin_button, skin_dropdown, skin_entry, skin_slider, skin_tabs, skin_toggle,
};
use crate::components::{Button, Dropdown, Entry, Skinned, Slider, Tabs, Toggle};
use crate::queries::skin::worn_above;
use ennui::prelude::{Glance, Later, Res, View, each};
use ennui_ui::prelude::{Theme, Wearing};

pub(crate) fn skin_presses(
    rows: Glance,
    buttons: View<(&Button, Option<&Skinned>)>,
    toggles: View<(&Toggle, Option<&Skinned>)>,
    look: Res<Theme>,
    wearing: Res<Wearing>,
    mut later: Later,
) {
    for (entity, (button, skinned)) in each(&buttons) {
        if skinned.is_none() {
            let worn = worn_above(&rows, (&look, &wearing), entity);
            skin_button(&mut later, &rows, worn, entity, &button.ask);
        }
    }
    for (entity, (toggle, skinned)) in each(&toggles) {
        if skinned.is_none() {
            let worn = worn_above(&rows, (&look, &wearing), entity);
            skin_toggle(&mut later, &rows, worn, entity, toggle.0);
        }
    }
}

pub(crate) fn skin_values(
    rows: Glance,
    sliders: View<(&Slider, Option<&Skinned>)>,
    entries: View<(&Entry, Option<&Skinned>)>,
    look: Res<Theme>,
    wearing: Res<Wearing>,
    mut later: Later,
) {
    for (entity, (slider, skinned)) in each(&sliders) {
        if skinned.is_none() {
            let worn = worn_above(&rows, (&look, &wearing), entity);
            skin_slider(&mut later, &rows, worn, entity, slider);
        }
    }
    for (entity, (entry, skinned)) in each(&entries) {
        if skinned.is_none() {
            let worn = worn_above(&rows, (&look, &wearing), entity);
            skin_entry(&mut later, &rows, worn, entity, entry);
        }
    }
}

pub(crate) fn skin_choices(
    rows: Glance,
    dropdowns: View<(&Dropdown, Option<&Skinned>)>,
    tabs: View<(&Tabs, Option<&Skinned>)>,
    look: Res<Theme>,
    wearing: Res<Wearing>,
    mut later: Later,
) {
    for (entity, (dropdown, skinned)) in each(&dropdowns) {
        if skinned.is_none() {
            let worn = worn_above(&rows, (&look, &wearing), entity);
            skin_dropdown(&mut later, &rows, worn, entity, dropdown);
        }
    }
    for (entity, (tabs, skinned)) in each(&tabs) {
        if skinned.is_none() {
            let worn = worn_above(&rows, (&look, &wearing), entity);
            skin_tabs(&mut later, &rows, worn, entity, tabs.chosen);
        }
    }
}
