use crate::data::Image;

ennui::tuning! {
    #[derive(Clone, Copy, PartialEq, ennui::Reflect)]
    #[reflect(about = "Sets HDR screen output and its brightness levels")]
    pub struct Display {
        #[reflect(about = "Sends HDR output to the screen")]
        wide: bool = false,
        #[reflect(about = "Brightness of plain white in HDR mode, in nits")]
        paper: f32 = 200.0,
        #[reflect(about = "Brightest the screen shows in HDR mode, in nits")]
        peak: f32 = 1000.0,
    }
}

#[derive(Default)]
pub struct Images {
    pub list: Vec<Image>,
    pub dirty: Vec<usize>,
    pub free: Vec<usize>,
    pub freed: Vec<usize>,
}

#[derive(Default)]
pub struct GpuClock {
    pub wanted: bool,
    pub spent: f32,
}
