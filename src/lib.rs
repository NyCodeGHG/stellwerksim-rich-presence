use stellwerksim::protocol::SystemInfo;

pub mod presence;
pub mod sts;
pub mod tray;

#[derive(Debug)]
pub enum Event {
    UpdatePresence(SystemInfo),
    ClearPresence,
    Exit,
}
