use std::collections::HashMap;

use gears::{Flag, FlagValue, Gears};

fn to_string_vec(strs: &Vec<&str>) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

fn assert_values_match(gears: &Gears) {
    assert_eq!(*gears.get_string("host"), "localhost");
    assert_eq!(*gears.get_i64("port"), 3000);
    assert_eq!(*gears.get_i128("max-size"), 999999999999999);
    assert_eq!(*gears.get_bool("timeout"), true);
    assert_eq!(*gears.get_f64("timeout-sec"), 1.5);
    assert_eq!(
        *gears.get_string_array("user"),
        to_string_vec(&vec!["scott", "allie"])
    );
    assert_eq!(*gears.get_i64_array("udp-port"), vec![5000, 5001, 5002]);
    assert_eq!(
        *gears.get_i128_array("allowed-input-range"),
        vec![20000000000000, 30000000000000]
    );
    assert_eq!(*gears.get_f64_array("allowed-output-range"), vec![6.5, 8.5]);
}

fn assert_overridden_values_match(gears: &Gears) {
    assert_eq!(*gears.get_string("host"), "127.0.0.1");
    assert_eq!(*gears.get_i64("port"), 8080);
    assert_eq!(*gears.get_i128("max-size"), 999999999999999);
    assert_eq!(*gears.get_bool("timeout"), true);
    assert_eq!(*gears.get_f64("timeout-sec"), 1.5);
    assert_eq!(
        *gears.get_string_array("user"),
        to_string_vec(&vec!["john", "aria"])
    );
    assert_eq!(*gears.get_i64_array("udp-port"), vec![4000, 4001]);
    assert_eq!(
        *gears.get_i128_array("allowed-input-range"),
        vec![20000000000000, 30000000000000]
    );
    assert_eq!(*gears.get_f64_array("allowed-output-range"), vec![6.5, 8.5]);
}

fn sample_gears<'a>() -> Gears<'a> {
    let mut gears = Gears::new();
    gears.add(Flag {
        name: "host",
        shorthand: Some('h'),
        default_value: FlagValue::String("".to_string()),
        description: None,
    });
    gears.add(Flag {
        name: "port",
        shorthand: Some('p'),
        default_value: FlagValue::Int64(80),
        description: None,
    });
    gears.add(Flag {
        name: "max-size",
        shorthand: None,
        default_value: FlagValue::Int128(10000),
        description: None,
    });
    gears.add(Flag {
        name: "timeout",
        shorthand: None,
        default_value: FlagValue::Bool(false),
        description: None,
    });
    gears.add(Flag {
        name: "timeout-sec",
        shorthand: None,
        default_value: FlagValue::Float64(10.0),
        description: None,
    });
    gears.add(Flag {
        name: "user",
        shorthand: None,
        default_value: FlagValue::StringArray(Vec::new()),
        description: None,
    });
    gears.add(Flag {
        name: "udp-port",
        shorthand: None,
        default_value: FlagValue::Int64Array(Vec::new()),
        description: None,
    });
    gears.add(Flag {
        name: "allowed-input-range",
        shorthand: None,
        default_value: FlagValue::Int128Array(Vec::new()),
        description: None,
    });
    gears.add(Flag {
        name: "allowed-output-range",
        shorthand: None,
        default_value: FlagValue::Float64Array(Vec::new()),
        description: None,
    });
    gears
}

#[test]
fn test_load_config_file() -> Result<(), String> {
    let mut gears = sample_gears();
    gears.add_config_file("tests/sample_config.json");
    gears.load(&HashMap::new(), &Vec::new())?;
    assert_values_match(&gears);
    Ok(())
}

#[test]
fn test_load_config_files() -> Result<(), String> {
    let mut gears = sample_gears();
    gears.add_config_file("tests/sample_config.json");
    gears.add_config_file("tests/sample_config_override.json");
    gears.load(&HashMap::new(), &Vec::new())?;
    assert_overridden_values_match(&gears);
    Ok(())
}
