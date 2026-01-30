use strum::FromRepr;

#[repr(usize)]
#[derive(Debug, FromRepr)]
pub enum BareMetalStatus {
    Uninitialized = 0,
    Error = 1,
    DoneInitializing = 2,
}
