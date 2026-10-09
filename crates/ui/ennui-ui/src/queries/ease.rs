pub fn toward(held: f32, wanted: f32, step: f32, enter: f32, leave: f32) -> f32 {
    let pace = match wanted > held {
        true => enter,
        false => leave,
    };
    let share = (step * pace).clamp(0.0, 1.0);
    held + (wanted - held) * share
}
