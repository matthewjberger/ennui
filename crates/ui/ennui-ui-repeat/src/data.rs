use crate::components::{Prefab, Repeat};
use ennui::prelude::View;
use ennui_ui::prelude::Panel;

pub(crate) type Repeats<'world, 'row> =
    View<'world, (&'row Repeat, Option<&'row Prefab>, Option<&'row Panel>)>;
