use std::{collections::HashMap, env};

use cliconf::{Flag, FlagValue, Flags};

fn to_string_vec(strs: Vec<&str>) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

fn assert_values_match(flags: &Flags) {
    assert_eq!(flags.get_string("host"), "localhost");
    assert_eq!(flags.get_i64("port"), 3000);
    assert_eq!(flags.get_i128("max-size"), 999999999999999);
    assert_eq!(flags.get_bool("timeout"), true);
    assert_eq!(flags.get_f64("timeout-sec"), 1.5);
    assert_eq!(
        flags.get_string_array("user"),
        to_string_vec(vec!["scott", "allie"])
    );
    assert_eq!(flags.get_i64_array("udp-port"), vec![5000, 5001, 5002]);
    assert_eq!(
        flags.get_i128_array("allowed-input-range"),
        vec![20000000000000, 30000000000000]
    );
    assert_eq!(flags.get_f64_array("allowed-output-range"), vec![6.5, 8.5]);
}

fn assert_overridden_values_match(flags: &Flags) {
    assert_eq!(flags.get_string("host"), "127.0.0.1");
    assert_eq!(flags.get_i64("port"), 8080);
    assert_eq!(flags.get_i128("max-size"), 999999999999999);
    assert_eq!(flags.get_bool("timeout"), true);
    assert_eq!(flags.get_f64("timeout-sec"), 1.5);
    assert_eq!(
        flags.get_string_array("user"),
        to_string_vec(vec!["john", "aria"])
    );
    assert_eq!(flags.get_i64_array("udp-port"), vec![4000, 4001]);
    assert_eq!(
        flags.get_i128_array("allowed-input-range"),
        vec![20000000000000, 30000000000000]
    );
    assert_eq!(flags.get_f64_array("allowed-output-range"), vec![6.5, 8.5]);
}

fn assert_double_overridden_values_match(flags: &Flags) {
    assert_eq!(flags.get_string("host"), "127.0.0.2");
    assert_eq!(flags.get_i64("port"), 8080);
    assert_eq!(flags.get_i128("max-size"), 999999999999999);
    assert_eq!(flags.get_bool("timeout"), true);
    assert_eq!(flags.get_f64("timeout-sec"), 1.5);
    assert_eq!(
        flags.get_string_array("user"),
        to_string_vec(vec!["john", "aria"])
    );
    assert_eq!(flags.get_i64_array("udp-port"), vec![3000]);
    assert_eq!(
        flags.get_i128_array("allowed-input-range"),
        vec![20000000000000, 30000000000000]
    );
    assert_eq!(flags.get_f64_array("allowed-output-range"), vec![6.5, 8.5]);
}

fn sample_flags() -> Flags {
    let mut flags = Flags::new();
    flags.add(Flag::new("host", FlagValue::String("".into())).shorthand('h'));
    flags.add(Flag::new("port", FlagValue::Int64(80)).shorthand('p'));
    flags.add(Flag::new("max-size", FlagValue::Int128(10000)));
    flags.add(Flag::new("timeout", FlagValue::Bool(false)));
    flags.add(Flag::new("timeout-sec", FlagValue::Float64(10.0)));
    flags.add(Flag::new("user", FlagValue::StringArray(Vec::new())).env_var_delimiter(","));
    flags.add(Flag::new("udp-port", FlagValue::Int64Array(Vec::new())).env_var_delimiter(","));
    flags.add(
        Flag::new("allowed-input-range", FlagValue::Int128Array(Vec::new())).env_var_delimiter("-"),
    );
    flags.add(
        Flag::new("allowed-output-range", FlagValue::Float64Array(Vec::new()))
            .env_var_delimiter("-"),
    );
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

fn assert_positionals_match(positionals: &Vec<String>) {
    assert_eq!(
        *positionals,
        to_string_vec(vec![
            "pos0",
            "pos1",
            "pos2",
            "-",
            "pos3",
            "--not-a-flag",
            "-f"
        ])
    );
}

#[test]
fn test_load_args() -> Result<(), String> {
    let mut flags = sample_flags();
    let args = to_string_vec(vec![
        "pos0",
        "-h",
        "localhost",
        "-p",
        "3000",
        "--max-size",
        "999999999999999",
        "--timeout",
        "pos1",
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
        "pos2",
        "--udp-port",
        "5002",
        "--allowed-input-range",
        "20000000000000",
        "-",
        "--allowed-input-range",
        "30000000000000",
        "--allowed-output-range",
        "6.5",
        "--allowed-output-range",
        "8.5",
        "pos3",
        "--",
        "--not-a-flag",
        "-f",
    ]);
    flags.load(&HashMap::new(), &args)?;
    assert_values_match(&flags);
    assert_positionals_match(&flags.positionals());
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
    let args = to_string_vec(vec![
        "pos0",
        "-h",
        "127.0.0.2",
        "pos1",
        "--udp-port",
        "3000",
        "pos2",
        "-",
        "pos3",
        "--",
        "--not-a-flag",
        "-f",
    ]);
    flags.load(&env_vars, &args)?;
    assert_double_overridden_values_match(&flags);
    assert_positionals_match(&flags.positionals());
    Ok(())
}

#[test]
fn test_readme_example() -> Result<(), String> {
    let mut flags = Flags::new();
    flags.add(
        Flag::new("hello-name", FlagValue::String("world".into()))
            .shorthand('n')
            .description("Who to say hello to."),
    );

    flags.add_config_file("/var/lib/hello-world/config.json");
    flags.add_home_config_file(".config/hello-world/config.json");

    let env_vars: HashMap<String, String> = env::vars().collect();

    let args: Vec<String> = env::args().collect();
    let args = args[1..].to_vec(); // Exclude name of executable

    flags.load(&env_vars, &args)?;

    let name = flags.get_string("hello-name");
    println!("Hello, {name}!");

    let _positionals = flags.positionals();

    Ok(())
}
