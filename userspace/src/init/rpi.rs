use crate::pedal_controller::rpi::PiPedalController;

mod init_screen;
mod low_level;

pub fn init() -> PiPedalController {
    PiPedalController::new(low_level::get_shared_mem().unwrap())
}
