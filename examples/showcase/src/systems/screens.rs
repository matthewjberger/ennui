use crate::data::Screen;
use crate::data::ask::Ask;
use crate::theme::SCREENS;
use ennui::prelude::{Res, ResMut};
use ennui_input::prelude::Actions;
use ennui_platform::prelude::Exit;
use ennui_screens::prelude::{Screens, close_screen, is_open, open_screen};
use ennui_state::prelude::State;
use ennui_ui_controls::prelude::{Asks, asked};

pub fn show(screen: Res<State<Screen>>, mut screens: ResMut<Screens>) {
    for (name, shown) in SCREENS {
        match (shown.contains(&screen.current), is_open(&screens, name)) {
            (true, false) => {
                open_screen(&mut screens, name, None);
            }
            (false, true) => close_screen(&mut screens, name),
            _ => {}
        }
    }
}

pub fn choose(
    asks: Res<Asks>,
    keys: Res<Actions<Ask>>,
    mut screen: ResMut<State<Screen>>,
    mut exit: ResMut<Exit>,
) {
    if asked(&asks, "profile") {
        screen.next = Some(Screen::Profile);
    }
    if asked(&asks, "settings") {
        screen.next = Some(Screen::Settings);
    }
    let back = asked(&asks, "back") || keys.pressed.contains(&Ask::Back);
    if back && screen.current != Screen::Title {
        screen.next = Some(Screen::Title);
    }
    if asked(&asks, "quit") {
        exit.0 = true;
    }
}
