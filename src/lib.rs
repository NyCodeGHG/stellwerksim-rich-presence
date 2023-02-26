use stellwerksim::protocol::SystemInfo;

pub mod presence;
pub mod sts;

#[derive(Debug)]
pub enum Event {
    UpdatePresence(SystemInfo),
    ClearPresence,
    Exit,
}
