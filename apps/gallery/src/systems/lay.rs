use crate::commands::lay::build_all;
use crate::commands::make;
use crate::data;
use crate::resources::{Arguments, Shown};
use ennui::prelude::{Later, Res, ResMut};
use ennui_render::resources::Images;
use ennui_ui::prelude::Theme;

pub(crate) fn open_gallery(
    mut later: Later,
    look: Res<Theme>,
    mut shown: ResMut<Shown>,
    mut images: ResMut<Images>,
    asked: Res<Arguments>,
) {
    let badge = make::icon(&mut images);
    let opening = asked.tab.min(data::SCREENS.len() - 1);
    shown.menu_asked = asked.menu;
    shown.theme = 0;
    shown.built = 0;
    build_all(&mut later, &look, &mut shown, badge, opening);
}
