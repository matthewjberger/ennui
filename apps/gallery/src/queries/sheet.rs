use crate::data::{GRID_ROWS, STOCK};

pub(crate) fn stock() -> Vec<[String; 4]> {
    (0..GRID_ROWS)
        .map(|place| {
            let (name, kind, size, state) = STOCK[place % STOCK.len()];
            [
                format!("{name} {place}"),
                String::from(kind),
                format!("{}", size.parse::<usize>().unwrap_or(0) + place),
                String::from(state),
            ]
        })
        .collect()
}
