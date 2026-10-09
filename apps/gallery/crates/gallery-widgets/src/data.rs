use crate::components::{Dated, Grips, Parts, Ranged, Rank, Shifted};
use ennui::prelude::{Mut, View};
use ennui_ui::prelude::{Click, Press, Rect};

use ennui_ui_controls::prelude::{Band, Knob};

pub(crate) const MONTHS: [&str; 12] = [
    "JANUARY",
    "FEBRUARY",
    "MARCH",
    "APRIL",
    "MAY",
    "JUNE",
    "JULY",
    "AUGUST",
    "SEPTEMBER",
    "OCTOBER",
    "NOVEMBER",
    "DECEMBER",
];

pub(crate) const DAYS: [&str; 7] = ["M", "T", "W", "T", "F", "S", "S"];

pub(crate) type Ranges<'world> = Mut<
    'world,
    (Ranged,),
    (
        &'static Rect,
        &'static Press,
        Option<&'static Parts>,
        Option<&'static Grips>,
    ),
>;

pub(crate) type Shifts<'world> = View<'world, (&'static Shifted, &'static Band, &'static Click)>;

pub(crate) type Picks<'world> = View<'world, (&'static Rank, &'static Band, &'static Click)>;

pub(crate) type Cards<'world> = Mut<'world, (Dated,), (&'static Knob,)>;
