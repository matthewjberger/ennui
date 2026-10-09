use crate::resources::Progress;

pub fn set_progress(progress: &mut Progress, value: f32) {
    progress.value = Some(value.clamp(0.0, 1.0));
    progress.sliding = false;
}

pub fn slide_progress(progress: &mut Progress) {
    progress.value = None;
    progress.sliding = true;
}

pub fn clear_progress(progress: &mut Progress) {
    progress.value = None;
    progress.sliding = false;
}
