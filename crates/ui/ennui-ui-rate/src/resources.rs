use ennui::prelude::Entity;
use ennui_ui::prelude::Line;

use nalgebra_glm::Vec4;

ennui::tuning! {
    #[derive(Clone, Copy)]
    pub struct Rate {
        shown: bool = true,
        frames: bool = true,
        frame_time: bool = false,
        gpu: bool = true,
        along: Line = Line::End,
        across: Line = Line::End,
        margin: f32 = 20.0,
        width: f32 = 220.0,
        good: f32 = 110.0,
        fair: f32 = 55.0,
        good_color: Vec4 = Vec4::new(0.4, 1.0, 0.5, 1.0),
        fair_color: Vec4 = Vec4::new(1.0, 0.85, 0.3, 1.0),
        poor_color: Vec4 = Vec4::new(1.0, 0.36, 0.3, 1.0),
        card: Option<Entity> = None,
        label: Option<Entity> = None,
    }
}
