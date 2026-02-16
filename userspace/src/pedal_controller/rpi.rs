use common::shared_mem::{
    types::{CoreID, CoreStatus},
    SharedMemUserspace,
};

pub struct PiPedalController {
    shared_mem: SharedMemUserspace,
}

impl PiPedalController {
    pub fn new(shared_mem: SharedMemUserspace) -> Self {
        Self { shared_mem }
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
}
