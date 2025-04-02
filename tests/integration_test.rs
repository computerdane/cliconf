use std::collections::HashMap;

use cliconf::{Flag, FlagValue, Flags};

fn to_string_vec(strs: &Vec<&str>) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

fn assert_values_match(flags: &Flags) {
    assert_eq!(*flags.get_string("host"), "localhost");
    assert_eq!(*flags.get_i64("port"), 3000);
    assert_eq!(*flags.get_i128("max-size"), 999999999999999);
    assert_eq!(*flags.get_bool("timeout"), true);
    assert_eq!(*flags.get_f64("timeout-sec"), 1.5);
    assert_eq!(
        *flags.get_string_array("user"),
        to_string_vec(&vec!["scott", "allie"])
    );
    assert_eq!(*flags.get_i64_array("udp-port"), vec![5000, 5001, 5002]);
    assert_eq!(
        *flags.get_i128_array("allowed-input-range"),
        vec![20000000000000, 30000000000000]
    );
    assert_eq!(*flags.get_f64_array("allowed-output-range"), vec![6.5, 8.5]);
}

fn assert_overridden_values_match(flags: &Flags) {
    assert_eq!(*flags.get_string("host"), "127.0.0.1");
    assert_eq!(*flags.get_i64("port"), 8080);
    assert_eq!(*flags.get_i128("max-size"), 999999999999999);
    assert_eq!(*flags.get_bool("timeout"), true);
    assert_eq!(*flags.get_f64("timeout-sec"), 1.5);
    assert_eq!(
        *flags.get_string_array("user"),
        to_string_vec(&vec!["john", "aria"])
    );
    assert_eq!(*flags.get_i64_array("udp-port"), vec![4000, 4001]);
    assert_eq!(
        *flags.get_i128_array("allowed-input-range"),
        vec![20000000000000, 30000000000000]
    );
    assert_eq!(*flags.get_f64_array("allowed-output-range"), vec![6.5, 8.5]);
}

fn assert_double_overridden_values_match(flags: &Flags) {
    assert_eq!(*flags.get_string("host"), "127.0.0.2");
    assert_eq!(*flags.get_i64("port"), 8080);
    assert_eq!(*flags.get_i128("max-size"), 999999999999999);
    assert_eq!(*flags.get_bool("timeout"), true);
    assert_eq!(*flags.get_f64("timeout-sec"), 1.5);
    assert_eq!(
        *flags.get_string_array("user"),
        to_string_vec(&vec!["john", "aria"])
    );
    assert_eq!(*flags.get_i64_array("udp-port"), vec![3000]);
    assert_eq!(
        *flags.get_i128_array("allowed-input-range"),
        vec![20000000000000, 30000000000000]
    );
    assert_eq!(*flags.get_f64_array("allowed-output-range"), vec![6.5, 8.5]);
}

fn sample_flags<'a>() -> Flags<'a> {
    let mut flags = Flags::new();
    flags.add(Flag {
        name: "host",
        shorthand: Some('h'),
        default_value: FlagValue::String("".to_string()),
        description: None,
    });
    flags.add(Flag {
        name: "port",
        shorthand: Some('p'),
        default_value: FlagValue::Int64(80),
        description: None,
    });
    flags.add(Flag {
        name: "max-size",
        shorthand: None,
        default_value: FlagValue::Int128(10000),
        description: None,
    });
    flags.add(Flag {
        name: "timeout",
        shorthand: None,
        default_value: FlagValue::Bool(false),
        description: None,
    });
    flags.add(Flag {
        name: "timeout-sec",
        shorthand: None,
        default_value: FlagValue::Float64(10.0),
        description: None,
    });
    flags.add(Flag {
        name: "user",
        shorthand: None,
        default_value: FlagValue::StringArray(Vec::new()),
        description: None,
    });
    flags.set_env_var_delimiter("user", ",");
    flags.add(Flag {
        name: "udp-port",
        shorthand: None,
        default_value: FlagValue::Int64Array(Vec::new()),
        description: None,
    });
    flags.set_env_var_delimiter("udp-port", ",");
    flags.add(Flag {
        name: "allowed-input-range",
        shorthand: None,
        default_value: FlagValue::Int128Array(Vec::new()),
        description: None,
    });
    flags.set_env_var_delimiter("allowed-input-range", "-");
    flags.add(Flag {
        name: "allowed-output-range",
        shorthand: None,
        default_value: FlagValue::Float64Array(Vec::new()),
        description: None,
    });
    flags.set_env_var_delimiter("allowed-output-range", "-");
    flags
}

#[test]
fn test_load_config_file() -> Result<(), String> {
    let mut flags = sample_flags();
    flags.add_config_file("tests/sample_config.json");
    flags.load(&HashMap::new(), &Vec::new())?;
    assert_values_match(&flags);
    Ok(())
}

#[test]
fn test_load_config_files_with_override() -> Result<(), String> {
    let mut flags = sample_flags();
    flags.add_config_file("tests/sample_config.json");
    flags.add_config_file("tests/sample_config_override.json");
    flags.load(&HashMap::new(), &Vec::new())?;
    assert_overridden_values_match(&flags);
    Ok(())
}

#[test]
fn test_load_env_vars() -> Result<(), String> {
    let mut flags = sample_flags();
    let mut env_vars = HashMap::new();
    env_vars.insert("HOST".to_string(), "localhost".to_string());
    env_vars.insert("PORT".to_string(), "3000".to_string());
    env_vars.insert("MAX_SIZE".to_string(), "999999999999999".to_string());
    env_vars.insert("TIMEOUT".to_string(), "true".to_string());
    env_vars.insert("TIMEOUT_SEC".to_string(), "1.5".to_string());
    env_vars.insert("USER".to_string(), "scott,allie".to_string());
    env_vars.insert("UDP_PORT".to_string(), "5000,5001,5002".to_string());
    env_vars.insert(
        "ALLOWED_INPUT_RANGE".to_string(),
        "20000000000000-30000000000000".to_string(),
    );
    env_vars.insert("ALLOWED_OUTPUT_RANGE".to_string(), "6.5-8.5".to_string());
    flags.load(&env_vars, &Vec::new())?;
    assert_values_match(&flags);
    Ok(())
}

#[test]
fn test_load_env_vars_with_override() -> Result<(), String> {
    let mut flags = sample_flags();
    let mut env_vars = HashMap::new();
    flags.add_config_file("tests/sample_config.json");
    env_vars.insert("HOST".to_string(), "127.0.0.1".to_string());
    env_vars.insert("PORT".to_string(), "8080".to_string());
    env_vars.insert("USER".to_string(), "john,aria".to_string());
    env_vars.insert("UDP_PORT".to_string(), "4000,4001".to_string());
    flags.load(&env_vars, &Vec::new())?;
    assert_overridden_values_match(&flags);
    Ok(())
}

#[test]
fn test_load_args() -> Result<(), String> {
    let mut flags = sample_flags();
    let args = to_string_vec(&vec![
        "-h",
        "localhost",
        "-p",
        "3000",
        "--max-size",
        "999999999999999",
        "--timeout",
        "--timeout-sec",
        "1.5",
        "--user",
        "scott",
        "--user",
        "allie",
        "--udp-port",
        "5000",
        "--udp-port",
        "5001",
        "--udp-port",
        "5002",
        "--allowed-input-range",
        "20000000000000",
        "--allowed-input-range",
        "30000000000000",
        "--allowed-output-range",
        "6.5",
        "--allowed-output-range",
        "8.5",
    ]);
    flags.load(&HashMap::new(), &args)?;
    assert_values_match(&flags);
    Ok(())
}

#[test]
fn test_double_override() -> Result<(), String> {
    let mut flags = sample_flags();
    let mut env_vars = HashMap::new();
    flags.add_config_file("tests/sample_config.json");
    env_vars.insert("HOST".to_string(), "127.0.0.1".to_string());
    env_vars.insert("PORT".to_string(), "8080".to_string());
    env_vars.insert("USER".to_string(), "john,aria".to_string());
    env_vars.insert("UDP_PORT".to_string(), "4000,4001".to_string());
    let args = to_string_vec(&vec!["-h", "127.0.0.2", "--udp-port", "3000"]);
    flags.load(&env_vars, &args)?;
    assert_double_overridden_values_match(&flags);
    Ok(())
}
