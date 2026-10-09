use crate::components::Dated;
use crate::data::MONTHS;

pub(crate) fn month_title(held: Dated) -> String {
    let name = MONTHS
        .get(held.month.saturating_sub(1) as usize)
        .copied()
        .unwrap_or_default();
    format!("{name} {}", held.year)
}

pub(crate) fn day_at(held: Dated, place: usize) -> Option<u32> {
    let first = first_weekday(held.year, held.month) as usize;
    let count = days_in(held.year, held.month) as usize;
    let day = place.checked_sub(first)? + 1;
    (day <= count).then_some(day as u32)
}

fn days_in(year: i32, month: u32) -> u32 {
    match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn first_weekday(year: i32, month: u32) -> u32 {
    let (year, month) = match month <= 2 {
        true => (year - 1, month + 12),
        false => (year, month),
    };
    let century = year / 100;
    let held = year % 100;
    let weekday = (1 + (13 * (month as i32 + 1)) / 5 + held + held / 4 + century / 4 + 5 * century)
        .rem_euclid(7);
    ((weekday + 5) % 7) as u32
}

pub(crate) fn stepped(year: i32, month: u32, way: i32) -> (i32, u32) {
    let held = month as i32 - 1 + way;
    (year + held.div_euclid(12), held.rem_euclid(12) as u32 + 1)
}
