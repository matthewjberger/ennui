pub mod commands;
pub mod components;
pub mod data;
pub mod plugin;
pub mod queries;
pub mod resources;
pub mod systems;
pub mod theme;

pub mod prelude {
    pub use crate::commands::build::{
        afloat, chrome, floating, frame, framed, image, ink, label, paint, panel, pinned, relay,
        restyle, screen, scroll, scroll_fill, separator, sheet, shielded, shortened, spacing,
        touched, under, wrapped,
    };
    pub use crate::commands::keep::{keyed, stream, sweep};
    pub use crate::commands::renew::{heighten, renew, renew_each, widen};
    pub use crate::components::{
        Centered, Click, Cursor, Edge, Effect, Fill, Float, Focus, Framed, Hidden, Host, Hosted,
        Hover, Inside, Lit, Off, Order, Panel, Picture, Pin, Poke, Press, Reach, Rect, Region,
        Scroll, Sharp, Step, Style, Tether, Text, Tone, Touch, Turn,
    };
    pub use crate::data::{
        Clicks, Dye, Frame, Hosting, Lay, Line, Mood, Mount, Named, Size, Sizing, Span, Window,
    };
    pub use crate::queries::ease::toward;
    pub use crate::queries::host::{
        fitted, host_point, pointer_in, pointer_of, screen_rect, staged,
    };
    pub use crate::queries::layout::{kids_of, thumb_of};
    pub use crate::queries::press::{clipped, inside_of, screen_from_pointer};
    pub use crate::queries::theme::worn_theme;
    pub use crate::queries::touch::{clicked, veiled};
    pub use crate::queries::window::{window, window_in};
    pub use crate::resources::{Hosts, Interface, Letterbox, Steering, Theme, Wearing};
}
