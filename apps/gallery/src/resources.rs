use ennui::prelude::Entity;
use ennui_platform::clap;
use ennui_ui_controls::prelude::Offer;

#[derive(clap::Args, Clone, Default)]
pub(crate) struct Arguments {
    #[arg(long, help = "Open the gallery on this tab", default_value_t = 0)]
    pub tab: usize,
    #[arg(long, help = "Open the context menu near the right edge at start")]
    pub menu: bool,
}

#[derive(Default)]
pub(crate) struct Shown {
    pub press: Option<Entity>,
    pub tick: Option<Entity>,
    pub switch: Option<Entity>,
    pub slide: Option<Entity>,
    pub name: Option<Entity>,
    pub quality: Option<Entity>,
    pub tabs: Option<Entity>,
    pub reading: Option<Entity>,
    pub ask: Option<Entity>,
    pub tell: Option<Entity>,
    pub raise_ask: Option<Entity>,
    pub raise_tell: Option<Entity>,
    pub count: usize,
    pub grid: Option<Entity>,
    pub menu_target: Option<Entity>,
    pub menu_said: Option<Entity>,
    pub menu_opened: bool,
    pub menu_asked: bool,
    pub deck: Option<Entity>,
    pub dock_tree: Option<Entity>,
    pub dock_pointer: Option<Entity>,
    pub long: Option<Entity>,
    pub theme: usize,
    pub theme_pick: Option<Entity>,
    pub preview: Option<usize>,
    pub canvas: Option<Entity>,
    pub palette: Option<Entity>,
    pub raise_palette: Option<Entity>,
    pub offered: Option<(Entity, usize)>,
    pub offers: Vec<Offer>,
    pub built: usize,
    pub icon_said: Option<Entity>,
    pub progress_slide: Option<Entity>,
    pub progress_sliding: Option<Entity>,
    pub progress_clear: Option<Entity>,
    pub progress_last: f32,
}

#[derive(Default)]
pub(crate) struct Sheet {
    pub held: Vec<[String; 4]>,
    pub sorted: Option<(usize, bool)>,
}
