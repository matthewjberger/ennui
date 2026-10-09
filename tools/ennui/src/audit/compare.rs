use std::path::{Path, PathBuf};

#[derive(clap::Args)]
pub struct Compare {
    #[arg(help = "The first PNG")]
    pub(super) first: PathBuf,
    #[arg(help = "The second PNG")]
    pub(super) second: PathBuf,
}

pub(super) fn compared(first: &Path, second: &Path) -> Result<String, String> {
    let first = image::open(first)
        .map_err(|problem| format!("{}: {problem}", first.display()))?
        .into_rgba8();
    let second = image::open(second)
        .map_err(|problem| format!("{}: {problem}", second.display()))?
        .into_rgba8();
    if first.dimensions() != second.dimensions() {
        let (width, height) = first.dimensions();
        let (other_width, other_height) = second.dimensions();
        return Ok(format!(
            "shape ({width}, {height}) ({other_width}, {other_height})"
        ));
    }
    let (width, height) = first.dimensions();
    let mut over_32 = 0usize;
    let mut over_0 = 0usize;
    let mut changed: Option<(u32, u32, u32, u32)> = None;
    for (x, y, pixel) in first.enumerate_pixels() {
        let other = second.get_pixel(x, y);
        let difference = pixel
            .0
            .iter()
            .zip(other.0)
            .map(|(mine, theirs)| mine.abs_diff(theirs))
            .max()
            .unwrap_or(0);
        if difference > 0 {
            over_0 += 1;
        }
        if difference > 32 {
            over_32 += 1;
            changed = Some(match changed {
                None => (x, y, x, y),
                Some((left, top, right, bottom)) => {
                    (left.min(x), top.min(y), right.max(x), bottom.max(y))
                }
            });
        }
    }
    let where_changed = changed
        .map(|(left, top, right, bottom)| {
            format!(", changed box x {left}..{right} y {top}..{bottom}")
        })
        .unwrap_or_default();
    Ok(format!(
        "{over_32} over 32, {over_0} over 0, of {}{where_changed}",
        width as usize * height as usize
    ))
}
