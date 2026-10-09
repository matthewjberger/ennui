use crate::data::{Dye, Lay, Line, Mood, Mount, Size, Sizing, Span};
use ennui::prelude::Entity;
use ennui_render::data::TextureId;
use ennui_text::prelude::Align;
use nalgebra_glm::{Vec2, Vec4};

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub center: Vec2,
    pub size: Vec2,
}

ennui::tuning! {
    #[derive(Clone, Copy, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "The root of a user interface tree: where it shows, how it scales, its theme and its layer")]
    pub struct Host {
        #[reflect(about = "Screen fills the window, Inside fills another element's rect")]
        mount: Mount = Mount::Screen,
        #[reflect(about = "The element whose rect an Inside host fills")]
        inside: Option<Entity> = None,
        #[reflect(about = "Pixel keeps reference pixels at the display density and shrinks to fit; Wide, Tall and Short match the reference size on that side")]
        sizing: Sizing = Sizing::Pixel,
        #[reflect(about = "The reference size in pixels that the sizing rule matches")]
        reference: Vec2 = Vec2::new(1280.0, 720.0),
        #[reflect(about = "The entity whose Theme this tree uses, none takes the Theme resource")]
        theme: Option<Entity> = None,
        #[reflect(about = "Hosts with a higher layer draw and take the pointer over lower ones")]
        layer: u32 = 0,
        #[reflect(about = "Keeps keyboard focus inside this host while it shows")]
        modal: bool = false,
        #[reflect(about = "Follows the user's interface scale setting; editor chrome turns this off")]
        scaled: bool = true,
    }
}

ennui::tuning! {
    #[derive(Clone, Copy, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "The layout of an element: its size, how its children flow, padding and gaps, in reference pixels")]
    pub struct Panel {
        #[reflect(about = "Width: Fixed pixels, Fill a share of the free room, or Hug the content")]
        wide: Span = Span::Hug,
        #[reflect(about = "Height: Fixed pixels, Fill a share of the free room, or Hug the content")]
        tall: Span = Span::Hug,
        #[reflect(about = "Children stack in a Column or a Row")]
        flow: Lay = Lay::Column,
        #[reflect(about = "Where the children sit along the flow")]
        along: Line = Line::Middle,
        #[reflect(about = "Where the children sit across the flow")]
        across: Line = Line::Middle,
        #[reflect(range(0.0, 200.0))]
        #[reflect(about = "Room between the edge and the children, in pixels")]
        pad: f32 = 0.0,
        #[reflect(range(0.0, 200.0))]
        #[reflect(about = "Room between the children, in pixels")]
        gap: f32 = 0.0,
        #[reflect(about = "Text inside wraps to the width it gets")]
        wraps: bool = false,
        #[reflect(about = "Text inside is cut with an ellipsis to the width it gets")]
        shrinks: bool = false,
        #[reflect(about = "Children are clipped to this element's rect")]
        clips: bool = false,
        #[reflect(about = "The children scroll with the wheel and a thumb, and are clipped")]
        scrolls: bool = false,
    }
}

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(
    about = "Pins a child at a share of its parent's rect, nudged by pixels, with its own pivot"
)]
pub struct Pin {
    #[reflect(about = "Where on the parent, 0 to 1 from the top left")]
    pub at: [f32; 2],
    #[reflect(about = "Pixels to move from that spot")]
    pub nudge: [f32; 2],
    #[reflect(about = "Which point of this element sits there, 0 to 1 from the top left")]
    pub pivot: [f32; 2],
}

ennui::tuning! {
    #[derive(Clone, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "The look of an element: fill, edge, shadow, picture, and how each state changes them")]
    pub struct Style {
        #[reflect(about = "Fill color, Keep draws no fill")]
        fill: Dye = Dye::Keep,
        #[reflect(about = "Edge color, drawn when border is above zero")]
        edge: Dye = Dye::Edge,
        #[reflect(about = "Text color of a label, Keep leaves it")]
        ink: Dye = Dye::Keep,
        #[reflect(range(0.0, 20.0))]
        #[reflect(about = "Border width in pixels")]
        border: f32 = 0.0,
        #[reflect(range(0.0, 200.0))]
        #[reflect(about = "Corner radius in pixels")]
        round: f32 = 0.0,
        #[reflect(about = "Shadow color, Keep draws no shadow")]
        shadow: Dye = Dye::Keep,
        #[reflect(about = "Shadow offset x and y, softness and spread, in pixels")]
        cast: [f32; 4] = [0.0; 4],
        #[reflect(range(0.0, 1.0))]
        #[reflect(about = "Multiplies the alpha of the fill, edge and picture")]
        opacity: f32 = 1.0,
        #[reflect(path("png", "jpg", "jpeg"))]
        #[reflect(about = "A picture file drawn in the fill")]
        picture: String = String::new(),
        #[reflect(range(0.0, 200.0))]
        #[reflect(about = "Nine-slice corner size in pixels; above zero the picture frames the edges")]
        corner: f32 = 0.0,
        #[reflect(about = "The nine-slice picture takes the fill color and grows with the element, so the picture shapes the fill")]
        shaped: bool = false,
        #[reflect(about = "Changes while the pointer is over the element")]
        hover: Mood = Mood::default(),
        #[reflect(about = "Changes while the element is pressed")]
        press: Mood = Mood::default(),
        #[reflect(about = "Changes while the element has keyboard focus")]
        focus: Mood = Mood::default(),
        #[reflect(about = "Changes while the element is off")]
        off: Mood = Mood::default(),
        #[reflect(about = "Changes while the element is lit as chosen")]
        lit: Mood = Mood::default(),
    }
}

ennui::tuning! {
    #[derive(Clone, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "Words drawn in an element, with their size, color, font and alignment")]
    pub struct Text {
        #[reflect(about = "The words")]
        words: String = String::new(),
        #[reflect(about = "Text, Heading or Caption from the theme, or Fixed pixels")]
        size: Size = Size::Text,
        #[reflect(about = "Text color, Keep leaves the theme ink")]
        color: Dye = Dye::Ink,
        #[reflect(path("ttf", "otf"))]
        #[reflect(about = "A font file, empty takes the theme font")]
        font: String = String::new(),
        #[reflect(about = "Where lines sit in a wrapped block")]
        align: Align = Align::Start,
        #[reflect(range(0.0, 8.0))]
        #[reflect(about = "Outline width around the letters")]
        outline: f32 = 0.0,
        #[reflect(about = "Outline color")]
        outline_color: Dye = Dye::Keep,
        #[reflect(range(0.0, 8.0))]
        #[reflect(about = "How soft the letter edges are")]
        softness: f32 = 0.0,
    }
}

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "Hides this element and everything below it")]
pub struct Hidden(pub bool);

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "Order among siblings; higher draws later")]
pub struct Order(pub u32);

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "Draws the picture with nearest sampling")]
pub struct Sharp;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Hosted(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Kids(pub u32);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Step(pub Option<usize>);

#[derive(Clone, Default)]
pub struct Keyed {
    pub held: std::collections::HashMap<u64, Entity>,
    pub seen: Vec<u64>,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Fill(pub Vec4);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Edge(pub Vec4);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Picture(pub TextureId);

#[derive(Clone, Copy, PartialEq)]
pub struct Region(pub [f32; 4]);

impl Default for Region {
    fn default() -> Self {
        Self([0.0, 0.0, 1.0, 1.0])
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Framed {
    pub picture: TextureId,
    pub corner: f32,
    pub tint: Vec4,
    pub sharp: bool,
    pub shaped: bool,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Scroll(pub f32);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Reach(pub f32);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Touch;

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Shield;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Hover(pub bool);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Press(pub bool);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Click(pub bool);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Focus(pub bool);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Off(pub bool);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Float(pub Vec2);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Inside;

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Anchored(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Tether(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Thumb(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Poke(pub bool);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Warm(pub f32);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Sunk(pub f32);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Effect(pub [f32; 4]);

#[derive(Clone, Copy, Default, PartialEq, ennui::Reflect)]
#[reflect(
    about = "Turns the fill, edge and picture of this element around its middle, for slanted tape, stamps and cuts; text stays level"
)]
pub struct Turn(#[reflect(about = "The angle, in radians, counterclockwise")] pub f32);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Centered;

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Worn {
    pub fill: Vec4,
    pub edge: Vec4,
    pub ink: Vec4,
    pub height: f32,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Lettered(pub Vec2);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Lit(pub bool);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Tone {
    pub fill: Option<Vec4>,
    pub edge: Option<Vec4>,
}

impl Tone {
    pub fn filled(fill: Vec4) -> Self {
        Self {
            fill: Some(fill),
            edge: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Cursor(pub ennui_platform::prelude::CursorIcon);
