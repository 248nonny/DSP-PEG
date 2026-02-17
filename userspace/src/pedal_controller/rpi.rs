use simple_log::{info, warn};

use common::shared_mem::{
    types::{AtomicRingBufferErr, BaremetalMessage, CoreID, CoreStatus},
    SharedMemUserspace,
};

pub struct PiPedalController {
    shared_mem: SharedMemUserspace,
}

impl PiPedalController {
    pub fn new(shared_mem: SharedMemUserspace) -> Self {
        Self { shared_mem }
    }

    pub fn test_writing_message(&self) {
        info!(
            "testing sending a message: {:?}",
            self.shared_mem
                .write_message(CoreID::Core1, BaremetalMessage::Ping)
        );
    }
}

impl super::PedalController for PiPedalController {
    fn status(&self) -> Option<[CoreStatus; 3]> {
        let status: [Option<CoreStatus>; 3] = core::array::from_fn(|i| {
            self.shared_mem
                .read_core_status(CoreID::from_repr(i as u8).unwrap())
        });

        match status {
            [Some(x), Some(y), Some(z)] => Some([x, y, z]),
            _ => None,
        }
    }

    fn print_new_messages(&self) {
        loop {
            match self.shared_mem.read_message(CoreID::Core1) {
                Ok(x) => info!("Message: {:?}", x),
                Err(AtomicRingBufferErr::NoMessagesErr) => {
                    info!("No new messages.");
                    break;
                }
                Err(x) => warn!("Message error: {:?}", x),
            }
        }
    }
}
