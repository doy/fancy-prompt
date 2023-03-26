use std;

use std::io::Read;

mod sys;

// XXX maybe extract this out into a separate crate?

#[derive(Debug)]
pub struct PowerInfo {
    power_supplies: Vec<sys::PowerSupplyInfo>,
}

impl PowerInfo {
    pub fn new() -> PowerInfo {
        let power_supplies = sys::power_supplies();

        PowerInfo { power_supplies }
    }

    pub fn battery_usage(&self) -> Option<f64> {
        let mut total_now = 0;
        let mut total_full = 0;
        for battery in self.batteries() {
            if let Some(now) = battery.energy_now {
                total_now += now;
            } else {
                return None;
            }
            if let Some(full) = battery.energy_full {
                total_full += full;
            } else {
                return None;
            }
        }

        if total_full > 0 {
            Some((total_now as f64) / (total_full as f64))
        } else {
            None
        }
    }

    pub fn charging(&self) -> bool {
        for mains in self.mains() {
            if mains.online == Some(true) {
                return true;
            }
        }
        false
    }

    pub fn has_batteries(&self) -> bool {
        self.batteries().count() > 0
    }

    fn batteries(&self) -> impl Iterator<Item = &sys::PowerSupplyInfo> {
        self.power_supplies
            .iter()
            .filter(|p| p.ty == sys::PowerSupplyType::Battery)
    }

    fn mains(&self) -> impl Iterator<Item = &sys::PowerSupplyInfo> {
        self.power_supplies
            .iter()
            .filter(|p| p.ty == sys::PowerSupplyType::AC)
    }
}

fn slurp<T, U>(path: U) -> Option<T>
where
    T: std::str::FromStr,
    U: AsRef<std::path::Path>,
{
    let mut contents = String::new();
    std::fs::File::open(path).ok().and_then(|mut fh| {
        fh.read_to_string(&mut contents)
            .ok()
            .and_then(|_| contents.trim().parse().ok())
    })
}
