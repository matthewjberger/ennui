use ennui_ui::prelude::Span;

#[derive(Default)]
pub struct Clock {
    pub since: f32,
    pub running: f32,
}

ennui::tuning! {
    #[derive(Clone, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "Where the moving parts of the screens are this frame")]
    pub struct Motion {
        #[reflect(about = "The nudge of parts that slide in from the left, in pixels")]
        left: [f32; 2] = [0.0, 0.0],
        #[reflect(about = "The nudge of parts that slide in from the right, in pixels")]
        right: [f32; 2] = [0.0, 0.0],
        #[reflect(about = "Opacity of the stamp, which lands a beat after the card")]
        stamp: f32 = 1.0,
        #[reflect(about = "Opacity of the line that pulses under the title")]
        pulse: f32 = 1.0,
        #[reflect(about = "The width of the first profile bar's fill")]
        speed: Span = Span::Fixed(0.0),
        #[reflect(about = "The width of the second profile bar's fill")]
        focus: Span = Span::Fixed(0.0),
        #[reflect(about = "The width of the third profile bar's fill")]
        flair: Span = Span::Fixed(0.0),
    }
}
