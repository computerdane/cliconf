use std::{collections::HashMap, env, fs::File, io::Read, path::Path};

use regex::Regex;
use serde_json::Value;

#[derive(Clone)]
pub enum FlagValue {
    BoolValue(bool),
    StringValue(String),
    Int64Value(i64),
    Int128Value(i128),
    Float64Value(f64),
    StringArrayValue(Vec<String>),
    Int64ArrayValue(Vec<i64>),
    Int128ArrayValue(Vec<i128>),
    Float64ArrayValue(Vec<f64>),
}

pub struct Flag<'a> {
    pub name: &'a str,
    pub shorthand: Option<char>,
    pub default_value: FlagValue,
    pub description: Option<&'a str>,
}

pub struct Gears<'a> {
    flags: HashMap<&'a str, Flag<'a>>,
    flag_values: HashMap<&'a str, FlagValue>,
    shorthand_names: HashMap<char, &'a str>,
    positionals: Vec<String>,
    config_files: Vec<String>,
}

impl<'a> Flag<'a> {
    fn assert_valid(&self) -> Result<(), String> {
        let re = Regex::new(r"^([a-z]|[0-9]|-)+$").expect("Failed to compile regex");
        if !re.is_match(self.name) {
            return Err(format!(
                "Flag name '{}' is invalid! Must be lowercase a-z with dashes only.",
                self.name
            ));
        }

        if let Some(c) = self.shorthand {
            if !(('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z') || ('0' <= c && c <= '9')) {
                return Err(format!(
                    "Flag shorthand '{c}' is invalid! Must be A-Z, a-z, or 0-9.",
                ));
            }
        }

        Ok(())
    }

    fn env_var_name(&self) -> String {
        return self.name.to_uppercase().replace("-", "_");
    }
}

impl<'a> Gears<'a> {
    pub fn new() -> Self {
        Gears {
            flags: HashMap::new(),
            shorthand_names: HashMap::new(),
            flag_values: HashMap::new(),
            positionals: vec![],
            config_files: vec![],
        }
    }

    pub fn add_config_file(&mut self, path: &str) {
        self.config_files.push(path.to_string())
    }

    pub fn add(&mut self, flag: Flag<'a>) {
        if let Err(e) = flag.assert_valid() {
            panic!("{e}")
        }

        if self.flags.contains_key(flag.name) {
            panic!("Flag with name '{}' already exists!", flag.name);
        }

        if let Some(c) = flag.shorthand {
            if self.shorthand_names.contains_key(&c) {
                panic!("Flag with shorthand '{c}' already exists!");
            }
            self.shorthand_names.insert(c, &flag.name);
        }

        let name = flag.name;
        let default_value = flag.default_value.clone();

        self.flags.insert(name, flag);
        self.flag_values.insert(name, default_value);
    }

    pub fn set(flag_values: &mut HashMap<&'a str, FlagValue>, name: &str, value: FlagValue) {
        match flag_values.get_mut(name) {
            Some(flag_value) => *flag_value = value,
            None => panic!("Cannot set flag. Flag not found: '{name}'"),
        }
    }

    pub fn get_bool(&self, name: &str) -> &bool {
        match self.flag_values.get(name) {
            Some(FlagValue::BoolValue(v)) => v,
            _ => panic!("Flag '{name}' is not of type bool!"),
        }
    }

    pub fn get_string(&self, name: &str) -> &String {
        match self.flag_values.get(name) {
            Some(FlagValue::StringValue(v)) => v,
            _ => panic!("Flag '{name}' is not of type String!"),
        }
    }

    pub fn get_i64(&self, name: &str) -> &i64 {
        match self.flag_values.get(name) {
            Some(FlagValue::Int64Value(v)) => v,
            _ => panic!("Flag '{name}' is not of type i64!"),
        }
    }

    pub fn get_i128(&self, name: &str) -> &i128 {
        match self.flag_values.get(name) {
            Some(FlagValue::Int128Value(v)) => v,
            _ => panic!("Flag '{name}' is not of type i128!"),
        }
    }

    pub fn get_f64(&self, name: &str) -> &f64 {
        match self.flag_values.get(name) {
            Some(FlagValue::Float64Value(v)) => v,
            _ => panic!("Flag '{name}' is not of type f64!"),
        }
    }

    pub fn get_string_array(&self, name: &str) -> &Vec<String> {
        match self.flag_values.get(name) {
            Some(FlagValue::StringArrayValue(v)) => v,
            _ => panic!("Flag '{name}' is not of type Vec<String>!"),
        }
    }

    pub fn get_i64_array(&self, name: &str) -> &Vec<i64> {
        match self.flag_values.get(name) {
            Some(FlagValue::Int64ArrayValue(v)) => v,
            _ => panic!("Flag '{name}' is not of type Vec<i64>!"),
        }
    }

    pub fn get_i128_array(&self, name: &str) -> &Vec<i128> {
        match self.flag_values.get(name) {
            Some(FlagValue::Int128ArrayValue(v)) => v,
            _ => panic!("Flag '{name}' is not of type Vec<i128>!"),
        }
    }

    pub fn get_f64_array(&self, name: &str) -> &Vec<f64> {
        match self.flag_values.get(name) {
            Some(FlagValue::Float64ArrayValue(v)) => v,
            _ => panic!("Flag '{name}' is not of type Vec<f64>!"),
        }
    }

    fn parse_string_and_set(
        flag_values: &mut HashMap<&'a str, FlagValue>,
        name: &str,
        value: String,
    ) -> Result<(), String> {
        let error_msg =
            |t: &str| format!("Failed to parse type {t} from flag '{name}' with value '{value}'");

        let name = name;

        match flag_values.get(name) {
            Some(FlagValue::BoolValue(_)) => Gears::set(
                flag_values,
                name,
                FlagValue::BoolValue(match value.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(error_msg("bool")),
                }),
            ),
            Some(FlagValue::StringValue(_)) => {
                Gears::set(flag_values, name, FlagValue::StringValue(value))
            }
            Some(FlagValue::Int64Value(_)) => Gears::set(
                flag_values,
                name,
                FlagValue::Int64Value(match value.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(error_msg("i64")),
                }),
            ),
            Some(FlagValue::Int128Value(_)) => Gears::set(
                flag_values,
                name,
                FlagValue::Int128Value(match value.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(error_msg("i128")),
                }),
            ),
            Some(FlagValue::Float64Value(_)) => Gears::set(
                flag_values,
                name,
                FlagValue::Float64Value(match value.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(error_msg("f64")),
                }),
            ),
            Some(FlagValue::StringArrayValue(_)) => todo!(),
            Some(FlagValue::Int64ArrayValue(_)) => todo!(),
            Some(FlagValue::Int128ArrayValue(_)) => todo!(),
            Some(FlagValue::Float64ArrayValue(_)) => todo!(),
            None => panic!("Cannot set flag. Flag not found: '{name}'"),
        };

        Ok(())
    }

    fn parse_args(&mut self, args: Vec<String>) -> Result<(), String> {
        if args.len() <= 1 {
            return Ok(());
        }

        let mut need_value_for_name = String::new();
        let mut as_positionals = false;

        for arg in args {
            let arg = arg.to_string();
            if as_positionals {
                self.positionals.push(arg);
            } else if need_value_for_name != "" {
                if let Err(e) =
                    Gears::parse_string_and_set(&mut self.flag_values, &need_value_for_name, arg)
                {
                    return Err(e);
                }
                need_value_for_name = String::from("");
            } else if arg == "-" {
                // Some programs use "-" to signify that data will be read from
                // stdin, so we treat it as a positional argument
                self.positionals.push(arg)
            } else if arg == "--" {
                // "--" is a special flag that treats all of the remaining
                // arguments as positional arguments
                as_positionals = true;
            } else if arg.starts_with("--") {
                let name = &arg[2..];
                match self.flag_values.get_mut(name) {
                    Some(FlagValue::BoolValue(v)) => *v = true,
                    Some(_) => need_value_for_name = name.to_string(),
                    None => return Err(format!("Unknown flag: --{name}")),
                }
            } else if arg.starts_with("-") {
                let shorthands = &arg[1..];
                for c in shorthands.chars() {
                    match self.shorthand_names.get(&c) {
                        Some(&name) => match self.flag_values.get_mut(name) {
                            Some(FlagValue::BoolValue(v)) => *v = true,
                            Some(_) => need_value_for_name = name.to_string(),
                            None => panic!("shorthand_names contains key '{c}', but flags does not contain key '{name}'"),
                        },
                        None => return Err(format!("Unknown flag: -{c}")),
                    }
                }
            } else {
                self.positionals.push(arg)
            }
        }

        Ok(())
    }

    fn parse_json(
        flag_values: &mut HashMap<&'a str, FlagValue>,
        json: String,
    ) -> Result<(), String> {
        match serde_json::from_str::<Value>(&json) {
            Ok(data) => match data {
                Value::Object(map) => {
                    for (key, value) in map {
                        let key = key.as_str();
                        match flag_values.get_mut(key) {
                            Some(flag_value) => match value {
                                Value::Bool(b) => match flag_value {
                                    FlagValue::BoolValue(v) => *v = b,
                                    _ => return Err(format!("Property '{key}' is not of type bool!"))
                                },
                                Value::Number(number) => match flag_value {
                                    FlagValue::Int64Value(v) => match number.as_i64() {
                                        Some(n) => *v = n,
                                        None => return Err(format!("Property '{key}' could not be parsed as an i64!")),
                                    },
                                    FlagValue::Int128Value(v) => match number.as_i128() {
                                        Some(n) => *v = n,
                                        None => return Err(format!("Property '{key}' could not be parsed as an i128!")),
                                    },
                                    FlagValue::Float64Value(v) => match number.as_f64() {
                                        Some(n) => *v = n,
                                        None => return Err(format!("Property '{key}' could not be parsed as a f64!")),
                                    },
                                    _ => return Err(format!("Property '{key}' is not of type number!"))
                                },
                                Value::String(s) => match flag_value {
                                    FlagValue::StringValue(v) => *v = s,
                                    _ => return Err(format!("Property '{key}' is not of type string!"))
                                },
                                Value::Array(_) => todo!(),
                                _ => return Err(format!("Property '{key}' is of the wrong type!")),
                            },
                            None => return Err(format!("Property '{key}' does not exist!"))
                        }
                    }
                },
                _ => return Err(
                    "Config must be a JSON Object with keys as flag names and values as flag values.".to_string()
                ),
            },
            Err(err) => return Err(format!("Failed to parse JSON: {err}")),
        };
        Ok(())
    }

    pub fn load(&mut self, args: Vec<String>) -> Result<(), String> {
        // 1. Config files
        for path in &self.config_files {
            if Path::new(path).exists() {
                match File::open(path) {
                    Ok(mut file) => {
                        let mut json = String::new();
                        if let Err(err) = file.read_to_string(&mut json) {
                            eprintln!("Failed to read config file '{path}': {err}")
                        }
                        if let Err(err) = Gears::parse_json(&mut self.flag_values, json) {
                            eprintln!("Config file '{path}' is invalid: {err}")
                        }
                    }
                    Err(err) => eprintln!("Failed to open config file '{path}': {err}"),
                }
            }
        }

        // 2. Environment variables
        for flag in self.flags.values() {
            if let Ok(value) = env::var(flag.env_var_name()) {
                Gears::parse_string_and_set(&mut self.flag_values, &String::from(flag.name), value)?
            }
        }

        // 3. Args
        self.parse_args(args)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_flag<'a>() -> Flag<'a> {
        return Flag {
            name: "my-bool",
            shorthand: Some('b'),
            default_value: FlagValue::BoolValue(false),
            description: None,
        };
    }

    #[test]
    #[should_panic]
    fn test_flag_assert_valid_fails_invalid_name() {
        let mut invalid_flag = sample_flag();
        invalid_flag.name = "My Invalid Flag Name!";
        invalid_flag.assert_valid().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_flag_assert_valid_fails_invalid_shorthand() {
        let mut invalid_flag = sample_flag();
        invalid_flag.shorthand = Some('$');
        invalid_flag.assert_valid().unwrap();
    }

    #[test]
    fn test_flag_assert_valid() {
        sample_flag().assert_valid().unwrap();
    }

    #[test]
    fn test_flag_env_var_name() {
        assert!(sample_flag().env_var_name() == "MY_BOOL")
    }

    fn sample_gears<'a>() -> Gears<'a> {
        let mut gears = Gears::new();
        gears.add(Flag {
            name: "my-bool",
            shorthand: Some('b'),
            default_value: FlagValue::BoolValue(false),
            description: None,
        });
        gears.add(Flag {
            name: "my-string",
            shorthand: Some('s'),
            default_value: FlagValue::StringValue(String::from("1")),
            description: None,
        });
        gears.add(Flag {
            name: "my-int64",
            shorthand: Some('i'),
            default_value: FlagValue::Int64Value(1),
            description: None,
        });
        gears.add(Flag {
            name: "my-int128",
            shorthand: Some('j'),
            default_value: FlagValue::Int128Value(1),
            description: None,
        });
        gears.add(Flag {
            name: "my-float64",
            shorthand: Some('g'),
            default_value: FlagValue::Float64Value(1.0),
            description: None,
        });
        return gears;
    }

    #[test]
    fn test_get() {
        let gears = sample_gears();
        assert!(*gears.get_bool("my-bool") == false);
        assert!(*gears.get_string("my-string") == String::from("1"));
        assert!(*gears.get_i64("my-int64") == 1);
        assert!(*gears.get_i128("my-int128") == 1);
        assert!(*gears.get_f64("my-float64") == 1.0);
    }

    fn assert_new_values_match(gears: &Gears) {
        assert!(*gears.get_bool("my-bool") == true);
        assert!(*gears.get_string("my-string") == "0");
        assert!(*gears.get_i64("my-int64") == 0);
        assert!(*gears.get_i128("my-int128") == 0);
        assert!(*gears.get_f64("my-float64") == 0.0);
    }

    #[test]
    fn test_parse_string_and_set() -> Result<(), String> {
        let mut gears = sample_gears();
        Gears::parse_string_and_set(&mut gears.flag_values, "my-bool", String::from("true"))?;
        Gears::parse_string_and_set(&mut gears.flag_values, "my-string", String::from("0"))?;
        Gears::parse_string_and_set(&mut gears.flag_values, "my-int64", String::from("0"))?;
        Gears::parse_string_and_set(&mut gears.flag_values, "my-int128", String::from("0"))?;
        Gears::parse_string_and_set(&mut gears.flag_values, "my-float64", String::from("0.0"))?;
        assert_new_values_match(&gears);
        Ok(())
    }

    fn to_string_vec(strs: Vec<&str>) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    fn sample_args() -> Vec<String> {
        to_string_vec(vec![
            "cmd",
            "--my-bool",
            "--my-string",
            "0",
            "--my-int64",
            "0",
            "--my-int128",
            "0",
            "--my-float64",
            "0.0",
        ])
    }

    #[test]
    fn test_parse_args() -> Result<(), String> {
        let mut gears = sample_gears();
        gears.parse_args(sample_args())?;
        assert_new_values_match(&gears);
        Ok(())
    }

    #[test]
    fn test_load() -> Result<(), String> {
        Ok(())
    }
}
