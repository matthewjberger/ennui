use ennui::prelude::Entity;

#[derive(Clone, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "A button; a click or a focus poke adds its ask to the Asks resource")]
pub struct Button {
    #[reflect(about = "The name app code reads from Asks when this button is clicked")]
    pub ask: String,
}

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "A switch or checkbox; a click flips it")]
pub struct Toggle(#[reflect(about = "On")] pub bool);

ennui::tuning! {
    #[derive(Clone, Copy, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "A slider; dragging the track moves value between low and high")]
    pub struct Slider {
        #[reflect(about = "The value, kept between low and high")]
        value: f32 = 0.0,
        #[reflect(about = "The value at the left end")]
        low: f32 = 0.0,
        #[reflect(about = "The value at the right end")]
        high: f32 = 1.0,
        #[reflect(range(0.0, 1000.0))]
        #[reflect(about = "Values snap to multiples of this above low when it is above zero")]
        step: f32 = 0.0,
    }
}

ennui::tuning! {
    #[derive(Clone, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "A text entry; typing while focused changes words")]
    pub struct Entry {
        #[reflect(about = "The typed words")]
        words: String = String::new(),
        #[reflect(about = "Faint words shown while empty")]
        hint: String = String::new(),
        #[reflect(about = "The most letters allowed, 0 for no limit")]
        most: u32 = 0,
    }
}

ennui::tuning! {
    #[derive(Clone, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "A dropdown; a click opens the list of options and picking one sets chosen")]
    pub struct Dropdown {
        #[reflect(about = "The options shown in the list")]
        options: Vec<String> = Vec::new(),
        #[reflect(about = "The index of the chosen option")]
        chosen: usize = 0,
    }
}

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(
    about = "Tabs; the child named bar holds one button per page and the other children are the pages"
)]
pub struct Tabs {
    #[reflect(about = "The index of the shown page")]
    pub chosen: usize,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Skinned;

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Slid(pub f32);

#[derive(Clone, Default, PartialEq)]
pub(crate) struct Wrote(pub String);

#[derive(Clone, Default, PartialEq)]
pub(crate) struct Offered {
    pub options: Vec<String>,
    pub chosen: usize,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Turned(pub usize);

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Answer {
    #[default]
    Waiting,
    Granted,
    Denied,
}

#[derive(Clone, Copy, Default)]
pub struct Asked(pub Answer);

#[derive(Clone, Copy, Default)]
pub(crate) struct Grant(pub Entity);

#[derive(Clone, Copy, Default)]
pub(crate) struct Deny(pub Entity);

#[derive(Clone, Copy, Default)]
pub struct Close(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Slide(pub f32);

#[derive(Clone, Copy, Default)]
pub(crate) struct Rest(pub Entity);

#[derive(Clone, Copy, Default)]
pub(crate) struct Track(pub Entity);

#[derive(Clone, Copy, Default)]
pub(crate) struct Grip {
    pub share: f32,
    pub x: f32,
    pub width: f32,
}

#[derive(Clone, Copy, Default)]
pub struct Knob(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Followed {
    pub at: usize,
    pub length: usize,
}

#[derive(Clone, Default, PartialEq)]
pub struct Field(pub String);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Entered(pub bool);

#[derive(Clone, Copy, Default)]
pub struct Room(pub usize);

#[derive(Clone, Copy, Default)]
pub struct Drop(pub bool);

#[derive(Clone, Copy, Default)]
pub(crate) struct Listing(pub Entity);

#[derive(Clone, Copy, Default)]
pub struct Pick(pub usize);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Chose(pub usize);

#[derive(Clone, Copy, Default)]
pub struct Band(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Marked(pub bool);

#[derive(Clone, Copy, Default)]
pub struct Tab(pub usize);

#[derive(Clone, Copy, Default)]
pub(crate) struct Page(pub usize);

#[derive(Clone, Copy, Default)]
pub(crate) struct Leaf(pub Entity);

#[derive(Clone, Copy, Default)]
pub struct Fold(pub bool);

#[derive(Clone, Copy, Default)]
pub(crate) struct Tip(pub Entity);

#[derive(Clone, Default)]
pub struct Hint(pub String);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Scrub {
    pub value: f32,
    pub step: f32,
    pub low: f32,
    pub high: f32,
    pub moved: f32,
    pub typing: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Menu(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Opened(pub bool);

#[derive(Clone, Copy, Default)]
pub(crate) struct Rung(pub u32);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Arrow(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Folder(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Swatch(pub Entity);

#[derive(Clone, Copy, PartialEq)]
pub struct Channel(pub usize);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Wheel {
    pub hue: f32,
    pub saturation: f32,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Dot(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Swing(pub f32);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Caret(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Ghost(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Swath(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Toast {
    pub age: f32,
    pub life: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Marks {
    pub shut: char,
    pub open: char,
}

impl Default for Marks {
    fn default() -> Self {
        Self {
            shut: crate::theme::MARK_SHUT,
            open: crate::theme::MARK_OPEN,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Ruled(pub f32);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Ruler(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Hunted(pub Entity);

#[derive(Clone, Default, PartialEq)]
pub(crate) struct Offers {
    pub rows: Vec<crate::data::Offer>,
    pub made: u64,
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct Scopes(pub Vec<(char, String)>);

#[derive(Clone, Default, PartialEq)]
pub(crate) struct Sought {
    pub text: String,
    pub made: u64,
    pub recent: Vec<u64>,
    pub lines: Vec<crate::data::Ranked>,
    pub lit: usize,
    pub held: Option<u64>,
    pub open: bool,
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct Recent(pub Vec<u64>);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Ran(pub Option<(u64, bool)>);

#[derive(Clone, Default, PartialEq)]
pub(crate) struct Deck {
    pub card: Entity,
    pub list: Entity,
    pub marks: Vec<Entity>,
    pub foot: Entity,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Slot(pub usize);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Mixed(pub crate::data::Blend);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Dial(pub Entity);

#[derive(Clone, Default)]
pub struct Sketch(pub Vec<ennui_quads::prelude::Quad>);

#[derive(Clone, Default, PartialEq)]
pub struct Plot {
    pub series: Vec<crate::data::Series>,
    pub low: [f32; 2],
    pub high: [f32; 2],
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Plotted(pub u64);

#[derive(Clone, Copy, PartialEq)]
pub struct Glide {
    pub shown: bool,
    pub progress: f32,
    pub rest: [f32; 2],
    pub away: [f32; 2],
    pub time: f32,
    pub ease: crate::data::Ease,
}

impl Glide {
    pub const fn entering(side: crate::data::Side, reach: f32) -> Self {
        Self {
            shown: false,
            progress: 0.0,
            rest: [0.0, 0.0],
            away: match side {
                crate::data::Side::Top => [0.0, -reach],
                crate::data::Side::Bottom => [0.0, reach],
                crate::data::Side::Left => [-reach, 0.0],
                crate::data::Side::Right => [reach, 0.0],
            },
            time: crate::theme::GLIDE_TIME,
            ease: crate::data::Ease::Back(crate::theme::GLIDE_BACK),
        }
    }
}

ennui::setters! {
    Glide {
        shown: bool,
        progress: f32,
        rest: [f32; 2],
        away: [f32; 2],
        time: f32,
        ease: crate::data::Ease,
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct Meter {
    pub share: f32,
    pub tint: Option<nalgebra_glm::Vec4>,
    pub text: String,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Legend(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Tinted(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Titled(pub Entity);
