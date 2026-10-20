use super::super::capture::{Frame, Stamp};
use super::*;
use engine_core::automata::channel::{
    Backend, ChannelRow, ChannelSettings, Channels,
    metrics::{MetricAvailability, MetricId, MetricSample, Metrics},
};
use models::workbench::operations::channels::analysis::Field;
#[test]
fn csv_bundle_preserves_samples_fields_metadata_and_refuses_overwrite() {
    let ids = [ChannelId(5), ChannelId(9)];
    let channels = Channels {
        selected: Some(ids[0]),
        rows: ids
            .iter()
            .map(|id| ChannelRow {
                id: *id,
                kind: "life:conway".into(),
                backend: Backend::Cpu,
                status: Status::Paused,
                tick: 7,
                epoch: 2,
                settings: ChannelSettings::new(10.0),
            })
            .collect(),
    };
    let mut metrics = Metrics::default();
    for id in ids {
        metrics.declare(
            id,
            MetricId {
                name: "population",
                label: "alive",
                unit: "cells",
            },
            MetricAvailability::Live,
        );
        metrics.record(
            id,
            MetricSample {
                tick: 7,
                epoch: 2,
                value: 1.0,
            },
        );
    }
    let folder = std::env::temp_dir().join(format!(
        "soul-csv-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut c = Comparison {
        left: Some(ids[0]),
        right: Some(ids[1]),
        folder: folder.to_string_lossy().into_owned(),
        ..Default::default()
    };
    c.capture.result = Some(ids.map(|id| Frame {
        stamp: Stamp {
            id,
            epoch: 2,
            tick: 7,
            params: None,
            kernel: None,
            cpu_settings: None,
        },
        field: Field {
            size: [2, 1, 1],
            units: "density",
            values: vec![0.0, 1.0],
        },
    }));
    write(&c, &channels, &metrics).unwrap();
    let text = fs::read_to_string(folder.join("metrics.csv")).unwrap();
    assert_eq!(text.lines().count(), 3);
    assert!(text.contains("5,\"life:conway\",\"population\",\"cells\",2,7,1"));
    let text = fs::read_to_string(folder.join("differences.csv")).unwrap();
    assert_eq!(text.lines().count(), 3);
    assert!(text.contains("5,2,7,9,2,7,\"density\",1,0,0,1,1,0"));
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(folder.join("capture.json")).unwrap()).unwrap();
    assert_eq!(metadata[0]["tick"], 7);
    assert_eq!(metadata[1]["channel"], 9);
    assert!(write(&c, &channels, &metrics).is_err());
    fs::remove_dir_all(folder).unwrap();
}
