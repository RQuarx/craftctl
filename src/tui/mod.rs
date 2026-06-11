pub mod events;
pub mod screens;
pub mod terminal;
pub mod theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginMode {
    Microsoft,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginPhase {
    Choosing,
    WaitingForMicrosoft,
    Complete,
}
