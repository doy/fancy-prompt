pub fn power_supplies() -> Vec<super::PowerSupplyInfo> {
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
        let ty = super::super::slurp(entry.path().join("type"))
            .map(|t: String| super::PowerSupplyType::from_str(&t))
            .expect("couldn't find power supply type");
        let energy_full =
            super::super::slurp(entry.path().join("energy_full"));
        let energy_now = super::super::slurp(entry.path().join("energy_now"));
        let online = super::super::slurp(entry.path().join("online"))
            .map(|n: u8| n != 0);

        power_supplies.push(super::PowerSupplyInfo {
            name,
            ty,
            energy_now,
            energy_full,
            online,
        })
    }
    power_supplies
}

impl super::PowerSupplyType {
    fn from_str(ty: &str) -> Self {
        match ty {
            "Mains" => super::PowerSupplyType::AC,
            "Battery" => super::PowerSupplyType::Battery,
            "USB" => super::PowerSupplyType::Other,
            _ => panic!("unknown power supply type {}", ty),
        }
    }
}
