use std;
use walkdir;

use std::io::Read;

// XXX maybe extract this out into a separate crate?

#[derive(PartialEq, Eq, Debug, Clone)]
enum PowerSupplyType {
    AC,
    Battery,
}

#[derive(Debug)]
pub struct PowerInfo {
    power_supplies: Vec<PowerSupplyInfo>,
}

#[derive(Debug, Clone)]
struct PowerSupplyInfo {
    name: String,
    ty: PowerSupplyType,
    energy_now: Option<u64>,
    energy_full: Option<u64>,
    online: Option<bool>,
}

impl PowerInfo {
    pub fn new() -> PowerInfo {
        let mut power_supplies = vec![];
        for entry in walkdir::WalkDir::new("/sys/class/power_supply/")
            .min_depth(1)
            .max_depth(1)
            .follow_links(true)
        {
            let entry = entry.unwrap();

            let name = entry
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let ty = slurp(entry.path().join("type"))
                .map(|t: String| PowerSupplyType::from_str(&t))
                .expect("couldn't find power supply type");
            let full = slurp(entry.path().join("energy_full"));
            let now = slurp(entry.path().join("energy_now"));
            let online =
                slurp(entry.path().join("online")).map(|n: u8| n != 0);

            power_supplies.push(PowerSupplyInfo {
                name: name,
                ty: ty,
                energy_now: now,
                energy_full: full,
                online: online,
            })
        }

        PowerInfo {
            power_supplies: power_supplies,
        }
    }

    pub fn battery_usage(&self) -> Option<f64> {
        let mut total_now = 0;
        let mut total_full = 0;
        for battery in self.batteries() {
            if let Some(now) = battery.energy_now {
                total_now += now;
            }
            else {
                return None;
            }
            if let Some(full) = battery.energy_full {
                total_full += full;
            }
            else {
                return None;
            }
        }

        if total_full > 0 {
            Some((total_now as f64) / (total_full as f64))
        }
        else {
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

    fn batteries(&self) -> Vec<PowerSupplyInfo> {
        self.power_supplies
            .iter()
            .cloned()
            .filter(|p| p.ty == PowerSupplyType::Battery)
            .collect()
    }

    fn mains(&self) -> Vec<PowerSupplyInfo> {
        self.power_supplies
            .iter()
            .filter(|p| p.ty == PowerSupplyType::AC)
            .cloned()
            .collect()
    }
}

impl PowerSupplyType {
    fn from_str(ty: &str) -> Self {
        match ty {
            "Mains" => PowerSupplyType::AC,
            "Battery" => PowerSupplyType::Battery,
            _ => panic!("unknown power supply type {}", ty),
        }
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
