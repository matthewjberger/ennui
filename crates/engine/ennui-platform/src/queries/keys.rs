use crate::resources::Input;
use winit::keyboard::KeyCode;

pub fn commanding(input: &Input) -> bool {
    let super_commands = cfg!(any(target_os = "macos", target_arch = "wasm32"));
    input.held.contains(&KeyCode::ControlLeft)
        || input.held.contains(&KeyCode::ControlRight)
        || (super_commands
            && (input.held.contains(&KeyCode::SuperLeft)
                || input.held.contains(&KeyCode::SuperRight)))
}
