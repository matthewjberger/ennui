use ennui::prelude::ResMut;
use ennui_bind::prelude::Binding;

pub(crate) fn samples_only(mut binding: ResMut<Binding>) {
    if binding.live {
        binding.live = false;
    }
}
