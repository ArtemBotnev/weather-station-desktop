use std::collections::BTreeMap;
use weather_station_client::Device;

const OTHER_DEVICE_LOCATION: &str = "other";

pub fn group_devices_by_location(devices: Vec<Device>) -> BTreeMap<String, Vec<Device>> {
    let mut result = BTreeMap::new();

    for device in devices {
        let location = device
            .location
            .clone()
            .filter(|location| !location.trim().is_empty())
            .unwrap_or_else(|| OTHER_DEVICE_LOCATION.to_owned());

        result.entry(location).or_insert_with(Vec::new).push(device);
    }

    result
}
