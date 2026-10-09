pub const PORT_FILE: &str = ".ennui/editor/port";
pub const EVENTS: &str = "events from the user:";
pub const KEPT_FOLDER: &str = "ennui";
pub const OPEN_EDITORS: &str = "editor/open";
pub const WAIT_SECONDS: u64 = 600;

pub fn serving(root: &std::path::Path) -> Option<u16> {
    let port = std::fs::read_to_string(root.join(PORT_FILE))
        .ok()?
        .trim()
        .parse::<u16>()
        .ok()?;
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    std::net::TcpStream::connect_timeout(&address, std::time::Duration::from_millis(300))
        .ok()
        .map(|_| port)
}
