use stellwerksim::protocol::SystemInfo;

pub mod presence;

#[derive(Debug)]
pub enum Event {
    UpdatePresence(SystemInfo),
    ClearPresence,
    Exit,
}
