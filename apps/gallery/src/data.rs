use ennui_ui::prelude::Theme;
use gallery_widgets::prelude::Dated;

use ennui_ui_controls::prelude::{Marks, Note};
use ennui_ui_icons::prelude::icons;
use nalgebra_glm::Vec4;

pub(crate) const ROWS: usize = 24;
pub(crate) const GRID_ROWS: usize = 500;
pub(crate) const WHEN: Dated = Dated {
    year: 2026,
    month: 9,
    day: 20,
};

pub(crate) type Sample = (
    char,
    char,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

pub(crate) const SAMPLES: [Sample; 30] = [
    ('>', icons::FOLDER_OPEN, "Open a project", "", "CTRL O", ""),
    (
        '>',
        icons::FILE,
        "Save the document",
        "",
        "CTRL S",
        "SAVES A COPY",
    ),
    ('>', icons::IMAGES, "Export a PDF", "", "CTRL E", ""),
    ('>', icons::GRID, "Show the grid", "", "G", ""),
    ('>', icons::SQUARE, "Show the outlines", "", "B", ""),
    (
        '>',
        icons::IMAGE,
        "Take a picture",
        "",
        "F12",
        "OPENS THE FOLDER",
    ),
    ('>', icons::ARROW_LEFT, "Undo", "", "CTRL Z", ""),
    ('>', icons::ARROW_RIGHT, "Redo", "", "CTRL Y", ""),
    ('>', icons::PLAY, "Run the app", "", "F5", ""),
    ('>', icons::X, "Leave", "", "ALT F4", ""),
    (
        '#',
        icons::FILE,
        "home",
        "3 links",
        "scenes/ui/home.scene",
        "",
    ),
    (
        '#',
        icons::FILE,
        "settings",
        "1 link",
        "scenes/ui/settings.scene",
        "",
    ),
    (
        '#',
        icons::FILE,
        "pause",
        "no links",
        "scenes/ui/pause.scene",
        "",
    ),
    (
        '%',
        icons::PACKAGE,
        "menu button",
        "12 on this screen",
        "prefabs/menu_button",
        "PLACES ONE",
    ),
    (
        '%',
        icons::PACKAGE,
        "list row",
        "4 on this screen",
        "prefabs/list_row",
        "PLACES ONE",
    ),
    (
        '%',
        icons::PACKAGE,
        "toast card",
        "none here",
        "prefabs/toast_card",
        "PLACES ONE",
    ),
    (
        '+',
        icons::IMAGE,
        "logo.png",
        "picture",
        "assets/textures",
        "",
    ),
    (
        '+',
        icons::IMAGE,
        "button.png",
        "picture",
        "assets/textures",
        "",
    ),
    (
        '+',
        icons::IMAGE,
        "banner.png",
        "picture",
        "assets/textures",
        "",
    ),
    (
        '+',
        icons::IMAGE,
        "cover.jpg",
        "picture",
        "assets/textures",
        "",
    ),
    ('+', icons::TYPE, "display.ttf", "font", "assets/fonts", ""),
    ('+', icons::TYPE, "mono.ttf", "font", "assets/fonts", ""),
    (':', icons::SETTINGS, "Theme", "app setting", "midnight", ""),
    (':', icons::SETTINGS, "Scale", "app setting", "1.00", ""),
    (
        ':',
        icons::SETTINGS,
        "Language",
        "app setting",
        "english",
        "",
    ),
    (':', icons::SETTINGS, "Font", "app setting", "roboto", ""),
    (
        '@',
        icons::TYPE,
        "Title",
        "Text  Style",
        "title",
        "FOCUSES IT",
    ),
    (
        '@',
        icons::SQUARE,
        "Play",
        "Button  Style  Text",
        "play",
        "FOCUSES IT",
    ),
    (
        '@',
        icons::SLIDERS_HORIZONTAL,
        "Scale",
        "Slider  Bind",
        "scale",
        "FOCUSES IT",
    ),
    (
        '@',
        icons::LAYOUT_GRID,
        "Card",
        "Panel  Style",
        "card",
        "FOCUSES IT",
    ),
];
pub(crate) const NOTE_TEXT: &str =
    "Type here. The caret moves between lines with the arrows, and Enter makes a new line.";
pub(crate) const CRUMBS: [&str; 4] = ["home", "projects", "ennui", "docs"];
pub(crate) const STATS: [(&str, &str); 3] = [
    ("FILES", "1 284"),
    ("WORDS", "40 000"),
    ("SAVED", "2 min ago"),
];
pub(crate) const STOCK_NAMES: [&str; 8] = [
    "anvil", "bramble", "cinder", "dovetail", "elder", "fathom", "gable", "harrow",
];
pub(crate) const PARAGRAPH: &str = "A label with a wrap width breaks its own lines. The layout measures the block, not one long line, so the panel around it grows as tall as the text needs. Change the window width and the wrap stays where it was put, because the width is given, not taken from the parent.";

pub(crate) const MARKS: Marks = Marks {
    shut: icons::CHEVRON_RIGHT,
    open: icons::CHEVRON_DOWN,
};
pub(crate) const NOTES: [(&str, &str, Note); 4] = [
    ("PLAIN", "THE FILE WAS OPENED", Note::Plain),
    ("GOOD", "THE DOCUMENT WAS SAVED", Note::Good),
    ("WARN", "THE DISK IS ALMOST FULL", Note::Warn),
    ("BAD", "THE UPLOAD DID NOT FINISH", Note::Bad),
];
pub(crate) const QUALITY: [&str; 4] = ["LOW", "MEDIUM", "HIGH", "ULTRA"];
pub(crate) const MODES: [&str; 2] = ["FIRST", "SECOND"];
pub(crate) const SECTIONS: [&str; 3] = ["INBOX", "DRAFTS", "SENT"];
pub(crate) const SCREENS: [(&str, &str, char); 9] = [
    ("BUTTONS", "Toggles and choices", icons::TOGGLE_RIGHT),
    ("INPUTS", "Sliders and fields", icons::SLIDERS_HORIZONTAL),
    ("PICKERS", "Dropdowns and tabs", icons::LIST),
    ("CONTAINERS", "Lists and dialogs", icons::LAYOUT_GRID),
    ("DATA", "Grids and trees", icons::TABLE),
    ("LAYOUT", "Splits and menus", icons::GRID),
    ("TEXT", "Runs and long lists", icons::TYPE),
    ("PROPERTIES", "An inspector", icons::SLIDERS_VERTICAL),
    ("DOCK", "Panes you can move", icons::LAYOUT_GRID),
];

pub(crate) const PANES: [(&str, &str); 4] = [
    ("EDITOR", "The open document"),
    ("ASSETS", "Every file in the project"),
    ("CONSOLE", "What the app said"),
    ("PROFILER", "Where the frame went"),
];

pub(crate) const SHOWN_ICONS: [(char, &str); 53] = [
    (icons::HOUSE, "HOUSE"),
    (icons::SETTINGS, "SETTINGS"),
    (icons::SEARCH, "SEARCH"),
    (icons::X, "X"),
    (icons::MENU, "MENU"),
    (icons::PLUS, "PLUS"),
    (icons::MINUS, "MINUS"),
    (icons::TRASH, "TRASH"),
    (icons::REFRESH, "REFRESH"),
    (icons::SHUFFLE, "SHUFFLE"),
    (icons::FOLDER, "FOLDER"),
    (icons::FOLDER_OPEN, "FOLDER OPEN"),
    (icons::FILE, "FILE"),
    (icons::VIDEO, "VIDEO"),
    (icons::LIGHTBULB, "LIGHTBULB"),
    (icons::BOX, "BOX"),
    (icons::SLIDERS_HORIZONTAL, "SLIDERS HORIZONTAL"),
    (icons::EYE, "EYE"),
    (icons::EYE_OFF, "EYE OFF"),
    (icons::MAXIMIZE, "MAXIMIZE"),
    (icons::GRID, "GRID"),
    (icons::PALETTE, "PALETTE"),
    (icons::MOUNTAIN, "MOUNTAIN"),
    (icons::GLOBE, "GLOBE"),
    (icons::IMAGE, "IMAGE"),
    (icons::IMAGES, "IMAGES"),
    (icons::PAINTBRUSH, "PAINTBRUSH"),
    (icons::LAYOUT_GRID, "LAYOUT GRID"),
    (icons::ARROW_RIGHT, "ARROW RIGHT"),
    (icons::ARROW_LEFT, "ARROW LEFT"),
    (icons::CHEVRON_DOWN, "CHEVRON DOWN"),
    (icons::CHEVRON_UP, "CHEVRON UP"),
    (icons::CHEVRON_LEFT, "CHEVRON LEFT"),
    (icons::CHEVRON_RIGHT, "CHEVRON RIGHT"),
    (icons::PLAY, "PLAY"),
    (icons::PAUSE, "PAUSE"),
    (icons::SQUARE, "SQUARE"),
    (icons::CHECK, "CHECK"),
    (icons::CIRCLE_ALERT, "CIRCLE ALERT"),
    (icons::TRIANGLE_ALERT, "TRIANGLE ALERT"),
    (icons::INFO, "INFO"),
    (icons::GAUGE, "GAUGE"),
    (icons::TIMER, "TIMER"),
    (icons::COINS, "COINS"),
    (icons::LIST, "LIST"),
    (icons::NETWORK, "NETWORK"),
    (icons::TABLE, "TABLE"),
    (icons::SLIDERS_VERTICAL, "SLIDERS VERTICAL"),
    (icons::KEYBOARD, "KEYBOARD"),
    (icons::TYPE, "TYPE"),
    (icons::SHOPPING_BASKET, "SHOPPING BASKET"),
    (icons::TOGGLE_RIGHT, "TOGGLE RIGHT"),
    (icons::SPARKLES, "SPARKLES"),
];

pub(crate) const COLUMNS: [(&str, f32); 4] =
    [("NAME", 0.4), ("KIND", 0.2), ("SIZE", 0.2), ("STATE", 0.2)];

pub(crate) const STOCK: [(&str, &str, &str, &str); 8] = [
    ("anvil", "tool", "12", "held"),
    ("bramble", "plant", "3", "grown"),
    ("cinder", "ember", "1", "warm"),
    ("dovetail", "joint", "7", "cut"),
    ("elder", "tree", "40", "grown"),
    ("fathom", "depth", "6", "sunk"),
    ("gable", "roof", "22", "built"),
    ("harrow", "tool", "15", "held"),
];

pub(crate) const BRANCHES: [(&str, u32, bool, char); 7] = [
    ("project", 0, false, icons::FOLDER_OPEN),
    ("scenes", 1, false, icons::FOLDER_OPEN),
    ("home.scene", 2, true, icons::FILE),
    ("settings.scene", 2, true, icons::FILE),
    ("settings.scene", 1, true, icons::SETTINGS),
    ("assets", 1, false, icons::FOLDER_OPEN),
    ("logo.png", 2, true, icons::IMAGE),
];

pub(crate) const LONG_ROWS: usize = 2000;
pub(crate) const MENU_PICKS: [&str; 5] = [
    "OPEN\tCTRL O",
    "RENAME\tF2",
    "DUPLICATE\tCTRL D",
    "",
    "DELETE\tDEL",
];
pub(crate) const MENU_NAMES: [&str; 4] = ["OPENED", "RENAMED", "DUPLICATED", "DELETED"];

pub(crate) type Paint = fn(&mut Theme);

pub(crate) const THEMES: [(&str, Option<Paint>); 4] = [
    ("MIDNIGHT", None),
    ("SLATE", Some(slate)),
    ("EMBER", Some(ember)),
    ("DAWN", Some(dawn)),
];

fn slate(look: &mut Theme) {
    look.ground = Vec4::new(0.137, 0.153, 0.180, 1.0);
    look.panel = Vec4::new(0.106, 0.122, 0.145, 1.0);
    look.header = Vec4::new(0.161, 0.180, 0.212, 1.0);
    look.edge = Vec4::new(0.235, 0.259, 0.298, 1.0);
    look.accent = Vec4::new(0.404, 0.635, 0.816, 1.0);
    look.input = Vec4::new(0.086, 0.098, 0.118, 1.0);
}

fn ember(look: &mut Theme) {
    look.ground = Vec4::new(0.114, 0.098, 0.094, 1.0);
    look.panel = Vec4::new(0.086, 0.071, 0.067, 1.0);
    look.header = Vec4::new(0.153, 0.125, 0.114, 1.0);
    look.edge = Vec4::new(0.243, 0.196, 0.176, 1.0);
    look.accent = Vec4::new(0.937, 0.529, 0.243, 1.0);
    look.input = Vec4::new(0.071, 0.059, 0.055, 1.0);
}

fn dawn(look: &mut Theme) {
    look.ink = Vec4::new(0.118, 0.129, 0.153, 1.0);
    look.faint = Vec4::new(0.439, 0.467, 0.510, 1.0);
    look.ground = Vec4::new(0.937, 0.945, 0.957, 1.0);
    look.panel = Vec4::new(1.0, 1.0, 1.0, 1.0);
    look.header = Vec4::new(0.918, 0.929, 0.945, 1.0);
    look.edge = Vec4::new(0.812, 0.831, 0.859, 1.0);
    look.accent = Vec4::new(0.153, 0.388, 0.847, 1.0);
    look.input = Vec4::new(0.973, 0.976, 0.984, 1.0);
    look.shade = Vec4::new(0.0, 0.0, 0.0, 0.14);
    look.deep_shade = Vec4::new(0.0, 0.0, 0.0, 0.22);
    look.scrollbar = Vec4::new(0.0, 0.0, 0.0, 0.30);
}

pub(crate) const PROGRESS_NOTE: &str =
    "DRAG THE SLIDER TO SET A VALUE, SET SLIDING FOR A BAR THAT RUNS, OR CLEAR TO HIDE IT";
