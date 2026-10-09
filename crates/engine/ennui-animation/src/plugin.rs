use crate::components::{Channel, Play, Sequence, Tween};
use crate::data::{Animated, Ended, Marked};
use crate::systems::{sequence, tween};
use ennui::app::component;
use ennui::events::add;
use ennui::prelude::{App, Stage, Step, before, grouped, on};
use ennui::resources::hold;
use ennui_document::prelude::Placed;

pub fn resources(app: &mut App) {
    hold::<Placed>(&mut app.resources);
    add::<Marked>(app);
    add::<Ended>(app);
    component::<Sequence>(app);
    component::<Channel>(app);
    component::<Play>(app);
    component::<Tween>(app);
}

pub fn systems() -> Vec<Step> {
    vec![
        grouped(Animated, on(Stage::Update, tween::ease_tweens)),
        grouped(Animated, on(Stage::Update, sequence::play_sequences)),
        before(tween::ease_tweens, sequence::play_sequences),
    ]
}
