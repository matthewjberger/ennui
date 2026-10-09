pub struct Claiming;

#[derive(Clone)]
pub enum Act {
    Key(winit::keyboard::KeyCode),
    Click([f32; 2]),
    Point([f32; 2]),
    Type(String),
    Wheel(f32),
    Drag {
        button: winit::event::MouseButton,
        from: [f32; 2],
        to: [f32; 2],
    },
}

#[derive(Clone)]
pub struct Deed {
    pub act: Act,
    pub from: u64,
    pub to: u64,
}

#[derive(clap::Args, Default, Clone)]
pub struct NoArguments {}

#[derive(clap::Parser)]
pub(crate) struct Line<Extra: clap::Args> {
    #[arg(
        long,
        help = "Write a picture of each capture frame to this path, then close"
    )]
    pub capture: Option<std::path::PathBuf>,

    #[arg(
        long,
        value_delimiter = ',',
        default_value = "30",
        help = "The frames pictures are taken on, as 60,120,300; with more than one, each picture's name ends in its frame"
    )]
    pub capture_frame: Vec<u64>,

    #[arg(
        long,
        value_name = "SECONDS",
        help = "Move time on by this much each frame, so each run shows the same moments"
    )]
    pub step: Option<f32>,

    #[arg(
        long,
        help = "Start, write the component list for the editor, then close"
    )]
    pub describe: bool,

    #[arg(long, help = "Open the window at this size, written as WIDTHxHEIGHT")]
    pub size: Option<Span>,

    #[arg(
        long,
        help = "Close when standard input ends, so the program that started this one stops it by closing the pipe"
    )]
    pub leash: bool,

    #[arg(
        long,
        help = "Present in scRGB for an HDR display when the surface offers it"
    )]
    pub hdr: bool,

    #[arg(
        long,
        value_name = "FRAMES",
        help = "Draw this many frames, say what each one cost, then close"
    )]
    pub frames: Option<u64>,

    #[arg(
        long,
        value_name = "LINE",
        help = "Run this command line once the app starts, more than once to run several"
    )]
    pub run: Vec<String>,

    #[arg(
        long,
        value_name = "KEY@FRAME",
        value_parser = crate::queries::script::pressing,
        help = "Press a key such as KeyJ on a frame, or hold it over FROM-TO frames, more than once for several"
    )]
    pub press: Vec<Deed>,

    #[arg(
        long,
        value_name = "X,Y@FRAME",
        value_parser = crate::queries::script::clicking,
        help = "Move the pointer to X,Y and click the left button on a frame, or hold it over FROM-TO frames"
    )]
    pub click: Vec<Deed>,

    #[arg(
        long,
        value_name = "X,Y@FRAME",
        value_parser = crate::queries::script::pointing,
        help = "Move the pointer to X,Y on a frame"
    )]
    pub point: Vec<Deed>,

    #[arg(
        long = "type",
        value_name = "TEXT@FRAME",
        value_parser = crate::queries::script::typing,
        help = "Type TEXT into whatever reads typed text on a frame, more than once for several"
    )]
    pub typing: Vec<Deed>,

    #[arg(
        long,
        value_name = "STEPS@FRAME",
        value_parser = crate::queries::script::wheeling,
        help = "Turn the mouse wheel by STEPS on a frame, or on each of FROM-TO frames; positive turns away from you"
    )]
    pub wheel: Vec<Deed>,

    #[arg(
        long,
        value_name = "BUTTON:X,Y>X,Y@FROM-TO",
        value_parser = crate::queries::script::dragging,
        help = "Hold Left, Right or Middle and move the pointer in a line from the first X,Y to the second over FROM-TO frames"
    )]
    pub drag: Vec<Deed>,

    #[command(flatten)]
    pub extra: Extra,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Span {
    pub width: u32,
    pub height: u32,
}

impl std::str::FromStr for Span {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (across, down) = text
            .split_once('x')
            .ok_or_else(|| format!("a size reads as WIDTHxHEIGHT, not {text}"))?;
        Ok(Self {
            width: across.parse().map_err(|held| format!("{held}"))?,
            height: down.parse().map_err(|held| format!("{held}"))?,
        })
    }
}
