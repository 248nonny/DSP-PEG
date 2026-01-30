use common::{codes::BareMetalStatus, shared_mem::SharedMemUserspace};

use crate::pedal_controller::PedalControllerStatus;

pub struct PiPedalController {
    shared_mem: SharedMemUserspace,
}

impl PiPedalController {
    pub fn new(shared_mem: SharedMemUserspace) -> Self {
        Self { shared_mem }
    }
}

impl super::PedalController for PiPedalController {
    fn status(&self) -> PedalControllerStatus {
        let status = BareMetalStatus::from_repr(self.shared_mem.read_bare_metal_status());

        match status {
            Some(x) => match x {
                BareMetalStatus::Uninitialized | BareMetalStatus::Error => {
                    PedalControllerStatus::Waiting
                }
                _ => PedalControllerStatus::Ready,
            },

            None => panic!("Bad bare metal status value, doesn't match enum!"),
        }
    }
}
