use crate::domain::use_case::time_from_timestamp;
use crate::presentation::state::{DeviceMeasuresState, MeasureState, SensorMeasuresState};
use weather_station_client::{Device, Measure, Measurement};

const DASH: &str = "-";
const DEFAULT_DEVICE_NAME_PREFIX: &str = "Device";
const VALUE_TITLE_SEPARATOR: &str = " ";

pub fn to_device_measures_state(
    device: &Device,
    measurement: Option<Measurement>,
) -> DeviceMeasuresState {
    let device_name = device
        .name
        .clone()
        .unwrap_or_else(|| format!("{DEFAULT_DEVICE_NAME_PREFIX} {}", device.id));

    let Some(measurement) = measurement else {
        return DeviceMeasuresState {
            device_name,
            sensors: Vec::new(),
        };
    };

    let time = measurement.timestamp.as_deref().unwrap_or(DASH).to_owned();
    let sensors = group_measures_by_sensor(measurement.measures)
        .into_iter()
        .map(|group| SensorMeasuresState {
            sensor_name: group.sensor_name,
            sensor_place: group.sensor_place,
            measures: group
                .measures
                .into_iter()
                .map(|measure| to_measure_state(measure, &time))
                .collect(),
        })
        .collect();

    DeviceMeasuresState {
        device_name,
        sensors,
    }
}

fn group_measures_by_sensor(measures: Vec<Measure>) -> Vec<SensorMeasureGroup> {
    let mut sensors = Vec::<SensorMeasureGroup>::new();

    for measure in measures {
        if let Some(sensor) = sensors
            .iter_mut()
            .find(|sensor| sensor.sensor_id == measure.sensor_id)
        {
            sensor.measures.push(measure);
        } else {
            sensors.push(SensorMeasureGroup {
                sensor_id: measure.sensor_id.clone(),
                sensor_name: measure
                    .sensor_name
                    .clone()
                    .unwrap_or_else(|| DASH.to_owned()),
                sensor_place: measure.sensor_place.clone(),
                measures: vec![measure],
            });
        }
    }

    sensors
}

fn to_measure_state(measure: Measure, time: &str) -> MeasureState {
    let daily = measure.daily_calculation;
    MeasureState {
        title: format!(
            "{}{VALUE_TITLE_SEPARATOR}{}",
            measure.measure_name, measure.measure_unit
        ),
        value: measure.measure_value.to_string(),
        time: time.to_owned(),
        value_min: daily
            .as_ref()
            .map(|calculation| calculation.min_value.to_string())
            .unwrap_or_else(|| DASH.to_owned()),
        value_average: daily
            .as_ref()
            .map(|calculation| calculation.average_value.to_string())
            .unwrap_or_else(|| DASH.to_owned()),
        value_max: daily
            .as_ref()
            .map(|calculation| calculation.max_value.to_string())
            .unwrap_or_else(|| DASH.to_owned()),
        time_min: daily
            .as_ref()
            .and_then(|calculation| calculation.min_value_time.as_deref())
            .and_then(time_from_timestamp)
            .unwrap_or_else(|| DASH.to_owned()),
        time_max: daily
            .as_ref()
            .and_then(|calculation| calculation.max_value_time.as_deref())
            .and_then(time_from_timestamp)
            .unwrap_or_else(|| DASH.to_owned()),
    }
}

struct SensorMeasureGroup {
    sensor_id: String,
    sensor_name: String,
    sensor_place: String,
    measures: Vec<Measure>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_measures_from_same_sensor() {
        let device = Device {
            id: 1,
            device_type: None,
            name: Some("Outdoor device".to_owned()),
            location: Some("Garden".to_owned()),
        };
        let measurement = Measurement {
            timestamp: Some("2026-11-17T10:15:30.000+0300".to_owned()),
            time_zone_hours: 3,
            device: device.clone(),
            measures: vec![
                Measure {
                    sensor_id: "sensor-b".to_owned(),
                    sensor_name: Some("B sensor".to_owned()),
                    sensor_place: "first".to_owned(),
                    measure_name: "Pressure".to_owned(),
                    measure_value: 760.0,
                    measure_unit: "mmHg".to_owned(),
                    sensor_error: false,
                    daily_calculation: None,
                },
                Measure {
                    sensor_id: "sensor-a".to_owned(),
                    sensor_name: Some("A sensor".to_owned()),
                    sensor_place: "second".to_owned(),
                    measure_name: "Temperature".to_owned(),
                    measure_value: 18.5,
                    measure_unit: "C".to_owned(),
                    sensor_error: false,
                    daily_calculation: None,
                },
                Measure {
                    sensor_id: "sensor-a".to_owned(),
                    sensor_name: Some("A sensor".to_owned()),
                    sensor_place: "second".to_owned(),
                    measure_name: "Humidity".to_owned(),
                    measure_value: 72.0,
                    measure_unit: "%".to_owned(),
                    sensor_error: false,
                    daily_calculation: None,
                },
            ],
        };

        let state = to_device_measures_state(&device, Some(measurement));

        assert_eq!(state.device_name, "Outdoor device");
        assert_eq!(state.sensors.len(), 2);
        assert_eq!(state.sensors[0].sensor_name, "B sensor");
        assert_eq!(state.sensors[0].sensor_place, "first");
        assert_eq!(state.sensors[0].measures.len(), 1);
        assert_eq!(state.sensors[0].measures[0].title, "Pressure mmHg");
        assert_eq!(state.sensors[1].sensor_name, "A sensor");
        assert_eq!(state.sensors[1].sensor_place, "second");
        assert_eq!(state.sensors[1].measures.len(), 2);
        assert_eq!(state.sensors[1].measures[0].title, "Temperature C");
        assert_eq!(state.sensors[1].measures[1].title, "Humidity %");
    }
}
