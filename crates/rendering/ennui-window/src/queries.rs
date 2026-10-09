use ennui_platform::prelude::ScheduledCapture;
use std::path::PathBuf;

pub(crate) fn picture_at(schedule: &ScheduledCapture, frame: u64) -> Option<PathBuf> {
    let path = schedule.path.as_deref()?;
    let wanted = match schedule.frames.is_empty() {
        true => frame == 0,
        false => schedule.frames.contains(&frame),
    };
    if !wanted {
        return None;
    }
    if schedule.frames.len() <= 1 {
        return Some(path.to_path_buf());
    }
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let named = format!("{stem}-{frame}");
    Some(
        path.with_file_name(named)
            .with_extension(path.extension().unwrap_or_default()),
    )
}

pub(crate) fn last_picture(schedule: &ScheduledCapture, frame: u64) -> bool {
    !schedule.stay
        && schedule.path.is_some()
        && schedule.frames.iter().all(|wanted| *wanted <= frame)
}
