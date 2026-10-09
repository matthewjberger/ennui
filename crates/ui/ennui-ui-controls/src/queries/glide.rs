use crate::data::Ease;

pub(crate) fn eased(ease: Ease, progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    let late = progress - 1.0;
    match ease {
        Ease::Linear => progress,
        Ease::Cubic => 1.0 + late * late * late,
        Ease::Back(overshoot) => 1.0 + late * late * ((overshoot + 1.0) * late + overshoot),
    }
}
