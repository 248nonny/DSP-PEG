pub enum PedalControllerStatus {
    Ready,
    Waiting,
}

pub trait PedalController {
    fn status(&self) -> PedalControllerStatus;

    // fn poll_pedal_info_stream(&self) -> Vec<(PedalMessageType, String)>;
}

#[cfg(feature = "rpi")]
pub mod rpi;

#[cfg(feature = "remote_linux")]
mod remote_linux;
