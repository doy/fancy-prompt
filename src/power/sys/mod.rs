#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum PowerSupplyType {
    AC,
    Battery,
    Other,
}

#[derive(Debug, Clone)]
pub struct PowerSupplyInfo {
    pub name: String,
    pub ty: PowerSupplyType,
    pub energy_now: Option<u64>,
    pub energy_full: Option<u64>,
    pub online: Option<bool>,
}
