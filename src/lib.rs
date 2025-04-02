use std::{collections::HashMap, fs::File, io::Read, path::Path};

use regex::Regex;
use serde_json::Value;

#[derive(Clone)]
pub enum FlagValue {
    Bool(bool),
    String(String),
    Int64(i64),
    Int128(i128),
    Float64(f64),
    StringArray(Vec<String>),
    Int64Array(Vec<i64>),
    Int128Array(Vec<i128>),
    Float64Array(Vec<f64>),
}

pub struct Flag<'a> {
    pub name: &'a str,
    pub shorthand: Option<char>,
    pub default_value: FlagValue,
    pub description: Option<&'a str>,
}

pub struct Flags<'a> {
    flags: HashMap<&'a str, Flag<'a>>,
    flag_values: HashMap<&'a str, FlagValue>,
    unset_flags: HashMap<&'a str, bool>,
    shorthand_names: HashMap<char, &'a str>,
    env_var_delimiters: HashMap<&'a str, &'a str>,
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

impl<'a> Flags<'a> {
    pub fn new() -> Self {
        Flags {
            flags: HashMap::new(),
            flag_values: HashMap::new(),
            unset_flags: HashMap::new(),
            shorthand_names: HashMap::new(),
            env_var_delimiters: HashMap::new(),
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
        self.unset_flags.insert(name, true);
    }

    pub fn set_env_var_delimiter(&mut self, name: &'a str, delimiter: &'a str) {
        if !self.flags.contains_key(name) {
            panic!("Cannot set env var delimiter for unknown flag '{name}'")
        }
        self.env_var_delimiters.insert(name, delimiter);
    }

    pub fn set(flag_values: &mut HashMap<&'a str, FlagValue>, name: &str, value: FlagValue) {
        match flag_values.get_mut(name) {
            Some(flag_value) => *flag_value = value,
            None => panic!("Cannot set flag. Flag not found: '{name}'"),
        }
    }

    pub fn get_bool(&self, name: &str) -> &bool {
        match self.flag_values.get(name) {
            Some(FlagValue::Bool(v)) => v,
            None => panic!("Flag '{name}' does not exist!"),
            _ => panic!("Flag '{name}' is not of type bool!"),
        }
    }

    pub fn get_string(&self, name: &str) -> &String {
        match self.flag_values.get(name) {
            Some(FlagValue::String(v)) => v,
            None => panic!("Flag '{name}' does not exist!"),
            _ => panic!("Flag '{name}' is not of type String!"),
        }
    }

    pub fn get_i64(&self, name: &str) -> &i64 {
        match self.flag_values.get(name) {
            Some(FlagValue::Int64(v)) => v,
            None => panic!("Flag '{name}' does not exist!"),
            _ => panic!("Flag '{name}' is not of type i64!"),
        }
    }

    pub fn get_i128(&self, name: &str) -> &i128 {
        match self.flag_values.get(name) {
            Some(FlagValue::Int128(v)) => v,
            None => panic!("Flag '{name}' does not exist!"),
            _ => panic!("Flag '{name}' is not of type i128!"),
        }
    }

    pub fn get_f64(&self, name: &str) -> &f64 {
        match self.flag_values.get(name) {
            Some(FlagValue::Float64(v)) => v,
            None => panic!("Flag '{name}' does not exist!"),
            _ => panic!("Flag '{name}' is not of type f64!"),
        }
    }

    pub fn get_string_array(&self, name: &str) -> &Vec<String> {
        match self.flag_values.get(name) {
            Some(FlagValue::StringArray(v)) => v,
            None => panic!("Flag '{name}' does not exist!"),
            _ => panic!("Flag '{name}' is not of type Vec<String>!"),
        }
    }

    pub fn get_i64_array(&self, name: &str) -> &Vec<i64> {
        match self.flag_values.get(name) {
            Some(FlagValue::Int64Array(v)) => v,
            None => panic!("Flag '{name}' does not exist!"),
            _ => panic!("Flag '{name}' is not of type Vec<i64>!"),
        }
    }

    pub fn get_i128_array(&self, name: &str) -> &Vec<i128> {
        match self.flag_values.get(name) {
            Some(FlagValue::Int128Array(v)) => v,
            None => panic!("Flag '{name}' does not exist!"),
            _ => panic!("Flag '{name}' is not of type Vec<i128>!"),
        }
    }

    pub fn get_f64_array(&self, name: &str) -> &Vec<f64> {
        match self.flag_values.get(name) {
            Some(FlagValue::Float64Array(v)) => v,
            None => panic!("Flag '{name}' does not exist!"),
            _ => panic!("Flag '{name}' is not of type Vec<f64>!"),
        }
    }

    fn set_arary<T>(array: &mut Vec<T>, value: T, is_unset: &mut bool) {
        if *is_unset {
            *array = vec![value];
            *is_unset = false;
        } else {
            array.push(value);
        }
    }

    fn parse_string_and_set(
        flag_values: &mut HashMap<&'a str, FlagValue>,
        unset_flags: &mut HashMap<&'a str, bool>,
        name: &str,
        value: &String,
    ) -> Result<(), String> {
        let error_msg =
            |t: &str| format!("Failed to parse type {t} from flag '{name}' with value '{value}'");

        let name = name;

        match flag_values.get_mut(name) {
            Some(FlagValue::Bool(v)) => match value.parse() {
                Ok(b) => *v = b,
                Err(_) => return Err(error_msg("bool")),
            },
            Some(FlagValue::String(v)) => *v = value.to_string(),
            Some(FlagValue::Int64(v)) => match value.parse() {
                Ok(n) => *v = n,
                Err(_) => return Err(error_msg("i64")),
            },
            Some(FlagValue::Int128(v)) => match value.parse() {
                Ok(n) => *v = n,
                Err(_) => return Err(error_msg("i128")),
            },
            Some(FlagValue::Float64(v)) => match value.parse() {
                Ok(n) => *v = n,
                Err(_) => return Err(error_msg("f64")),
            },
            Some(FlagValue::StringArray(v)) => {
                Flags::set_arary(v, value.to_string(), unset_flags.get_mut(name).unwrap())
            }
            Some(FlagValue::Int64Array(v)) => match value.parse() {
                Ok(n) => Flags::set_arary(v, n, unset_flags.get_mut(name).unwrap()),
                Err(_) => return Err(error_msg("i64")),
            },
            Some(FlagValue::Int128Array(v)) => match value.parse() {
                Ok(n) => Flags::set_arary(v, n, unset_flags.get_mut(name).unwrap()),
                Err(_) => return Err(error_msg("i128")),
            },
            Some(FlagValue::Float64Array(v)) => match value.parse() {
                Ok(n) => Flags::set_arary(v, n, unset_flags.get_mut(name).unwrap()),
                Err(_) => return Err(error_msg("f64")),
            },
            None => panic!("Cannot set flag. Flag not found: '{name}'"),
        };

        Ok(())
    }

    fn parse_args(&mut self, args: &Vec<String>) -> Result<(), String> {
        if args.len() == 0 {
            return Ok(());
        }

        let mut need_value_for_name = String::new();
        let mut as_positionals = false;

        for arg in args {
            let arg = arg.to_string();
            if as_positionals {
                self.positionals.push(arg);
            } else if need_value_for_name != "" {
                Flags::parse_string_and_set(
                    &mut self.flag_values,
                    &mut self.unset_flags,
                    &need_value_for_name,
                    &arg,
                )?;
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
                    Some(FlagValue::Bool(v)) => *v = true,
                    Some(_) => need_value_for_name = name.to_string(),
                    None => return Err(format!("Unknown flag: --{name}")),
                }
            } else if arg.starts_with("-") {
                let shorthands = &arg[1..];
                for c in shorthands.chars() {
                    match self.shorthand_names.get(&c) {
                        Some(&name) => match self.flag_values.get_mut(name) {
                            Some(FlagValue::Bool(v)) => *v = true,
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
        json: &String,
    ) -> Result<(), String> {
        match serde_json::from_str::<Value>(&json) {
            Ok(data) => match data {
                Value::Object(map) => {
                    for (name, value) in map {
                        let name = name.as_str();
                        match flag_values.get_mut(name) {
                            Some(flag_value) => match value {
                                Value::Bool(b) => match flag_value {
                                    FlagValue::Bool(v) => *v = b,
                                    _ => return Err(format!("Property '{name}' is not of type bool!"))
                                },
                                Value::String(s) => match flag_value {
                                    FlagValue::String(v) => *v = s,
                                    _ => return Err(format!("Property '{name}' is not of type string!"))
                                },
                                Value::Number(number) => match flag_value {
                                    FlagValue::Int64(v) => match number.as_i64() {
                                        Some(n) => *v = n,
                                        None => return Err(format!("Property '{name}' could not be parsed as an i64!")),
                                    },
                                    FlagValue::Int128(v) => match number.as_i128() {
                                        Some(n) => *v = n,
                                        None => return Err(format!("Property '{name}' could not be parsed as an i128!")),
                                    },
                                    FlagValue::Float64(v) => match number.as_f64() {
                                        Some(n) => *v = n,
                                        None => return Err(format!("Property '{name}' could not be parsed as a f64!")),
                                    },
                                    _ => return Err(format!("Property '{name}' is not of type number!"))
                                },
                                Value::Array(array) => {
                                    for (i, array_value) in array.iter().enumerate() {
                                        match array_value {
                                            Value::String(s) => match flag_value {
                                                FlagValue::StringArray(v) => if i == 0 { *v = vec![s.to_string()]} else {v.push(s.to_string())},
                                                _ => return Err(format!("Property '{name}' is not of type string[]!"))
                                            },
                                            Value::Number(number) => match flag_value {
                                                FlagValue::Int64Array(v) => match number.as_i64() {
                                                    Some(n) => if i == 0 { *v = vec![n]} else {v.push(n)},
                                                    None => return Err(format!("Property '{name}' could not be parsed as a Vec<i64>!")),
                                                },
                                                FlagValue::Int128Array(v) => match number.as_i128() {
                                                    Some(n) => if i == 0 { *v = vec![n]} else {v.push(n)},
                                                    None => return Err(format!("Property '{name}' could not be parsed as a Vec<i128>!")),
                                                },
                                                FlagValue::Float64Array(v) => match number.as_f64() {
                                                    Some(n) => if i == 0 { *v = vec![n]} else {v.push(n)},
                                                    None => return Err(format!("Property '{name}' could not be parsed as a Vec<f64>!")),
                                                },
                                                _ => return Err(format!("Property '{name}' is not of type number[]!"))
                                            },
                                            _ => {},
                                        }
                                    }
                                },
                                _ => return Err(format!("Property '{name}' is of the wrong type!")),
                            },
                            None => return Err(format!("Unknown property: {name}"))
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

    pub fn load(
        &mut self,
        env_vars: &HashMap<String, String>,
        args: &Vec<String>,
    ) -> Result<(), String> {
        // 1. Config files
        for path in &self.config_files {
            if Path::new(path).exists() {
                match File::open(path) {
                    Ok(mut file) => {
                        let mut json = String::new();
                        if let Err(err) = file.read_to_string(&mut json) {
                            eprintln!("Failed to read config file '{path}': {err}")
                        }
                        if let Err(err) = Flags::parse_json(&mut self.flag_values, &json) {
                            eprintln!("Config file '{path}' is invalid: {err}")
                        }
                    }
                    Err(err) => eprintln!("Failed to open config file '{path}': {err}"),
                }
            }
        }

        // 2. Environment variables
        for flag in self.flags.values() {
            if let Some(value) = env_vars.get(&flag.env_var_name()) {
                match flag.default_value {
                    FlagValue::StringArray(_)
                    | FlagValue::Int64Array(_)
                    | FlagValue::Int128Array(_)
                    | FlagValue::Float64Array(_) => match self.env_var_delimiters.get(flag.name) {
                        Some(delim) => for item in value.split(delim) {
                            Flags::parse_string_and_set(
                                &mut self.flag_values,
                                &mut self.unset_flags,
                                &String::from(flag.name),
                                &item.to_string(),
                            )?
                        },
                        None => eprintln!(
                            "Warning: Setting '{}' using the environment variable '{}' is unsupported.",
                            flag.name,
                            flag.env_var_name()
                        ),
                    },
                    _ => Flags::parse_string_and_set(
                        &mut self.flag_values,
                        &mut self.unset_flags,
                        &String::from(flag.name),
                        value,
                    )?,
                }
            }
        }
        for is_unset in self.unset_flags.values_mut() {
            *is_unset = true;
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
            default_value: FlagValue::Bool(false),
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

    fn to_string_vec(strs: &Vec<&str>) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    fn sample_flags<'a>() -> Flags<'a> {
        let mut flags = Flags::new();
        flags.add(Flag {
            name: "my-bool",
            shorthand: Some('b'),
            default_value: FlagValue::Bool(false),
            description: None,
        });
        flags.add(Flag {
            name: "my-string",
            shorthand: Some('s'),
            default_value: FlagValue::String("1".to_string()),
            description: None,
        });
        flags.add(Flag {
            name: "my-int64",
            shorthand: Some('i'),
            default_value: FlagValue::Int64(1),
            description: None,
        });
        flags.add(Flag {
            name: "my-int128",
            shorthand: Some('j'),
            default_value: FlagValue::Int128(1),
            description: None,
        });
        flags.add(Flag {
            name: "my-float64",
            shorthand: Some('f'),
            default_value: FlagValue::Float64(1.0),
            description: None,
        });
        flags.add(Flag {
            name: "my-string-array",
            shorthand: Some('S'),
            default_value: FlagValue::StringArray(to_string_vec(&vec!["1", "2"])),
            description: None,
        });
        flags.add(Flag {
            name: "my-int64-array",
            shorthand: Some('I'),
            default_value: FlagValue::Int64Array(vec![1, 2]),
            description: None,
        });
        flags.add(Flag {
            name: "my-int128-array",
            shorthand: Some('J'),
            default_value: FlagValue::Int128Array(vec![1, 2]),
            description: None,
        });
        flags.add(Flag {
            name: "my-float64-array",
            shorthand: Some('F'),
            default_value: FlagValue::Float64Array(vec![1.0, 2.0]),
            description: None,
        });
        flags
    }

    #[test]
    fn test_get() {
        let flags = sample_flags();
        assert_eq!(*flags.get_bool("my-bool"), false);
        assert_eq!(*flags.get_string("my-string"), String::from("1"));
        assert_eq!(*flags.get_i64("my-int64"), 1);
        assert_eq!(*flags.get_i128("my-int128"), 1);
        assert_eq!(*flags.get_f64("my-float64"), 1.0);
        assert_eq!(
            *flags.get_string_array("my-string-array"),
            to_string_vec(&vec!["1", "2"])
        );
        assert_eq!(*flags.get_i64_array("my-int64-array"), vec![1, 2]);
        assert_eq!(*flags.get_i128_array("my-int128-array"), vec![1, 2]);
        assert_eq!(*flags.get_f64_array("my-float64-array"), vec![1.0, 2.0]);
    }

    fn assert_new_values_match(flags: &Flags) {
        assert_eq!(*flags.get_bool("my-bool"), true);
        assert_eq!(*flags.get_string("my-string"), "0");
        assert_eq!(*flags.get_i64("my-int64"), 0);
        assert_eq!(*flags.get_i128("my-int128"), 0);
        assert_eq!(*flags.get_f64("my-float64"), 0.0);
        assert_eq!(
            *flags.get_string_array("my-string-array"),
            to_string_vec(&vec!["3", "4"])
        );
        assert_eq!(*flags.get_i64_array("my-int64-array"), vec![3, 4]);
        assert_eq!(*flags.get_i128_array("my-int128-array"), vec![3, 4]);
        assert_eq!(*flags.get_f64_array("my-float64-array"), vec![3.0, 4.0]);
    }

    #[test]
    fn test_parse_string_and_set() -> Result<(), String> {
        let mut flags = sample_flags();
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-bool",
            &"true".to_string(),
        )?;
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-string",
            &"0".to_string(),
        )?;
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-int64",
            &"0".to_string(),
        )?;
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-int128",
            &"0".to_string(),
        )?;
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-float64",
            &"0.0".to_string(),
        )?;
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-string-array",
            &"3".to_string(),
        )?;
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-string-array",
            &"4".to_string(),
        )?;
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-int64-array",
            &"3".to_string(),
        )?;
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-int64-array",
            &"4".to_string(),
        )?;
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-int128-array",
            &"3".to_string(),
        )?;
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-int128-array",
            &"4".to_string(),
        )?;
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-float64-array",
            &"3.0".to_string(),
        )?;
        Flags::parse_string_and_set(
            &mut flags.flag_values,
            &mut flags.unset_flags,
            "my-float64-array",
            &"4.0".to_string(),
        )?;
        assert_new_values_match(&flags);
        Ok(())
    }

    fn sample_args() -> Vec<String> {
        to_string_vec(&vec![
            "--my-bool",
            "--my-string",
            "0",
            "--my-int64",
            "0",
            "--my-int128",
            "0",
            "--my-float64",
            "0.0",
            "--my-string-array",
            "3",
            "--my-string-array",
            "4",
            "--my-int64-array",
            "3",
            "--my-int64-array",
            "4",
            "--my-int128-array",
            "3",
            "--my-int128-array",
            "4",
            "--my-float64-array",
            "3.0",
            "--my-float64-array",
            "4.0",
        ])
    }

    fn sample_args_shorthand() -> Vec<String> {
        to_string_vec(&vec![
            "-b", "-s", "0", "-i", "0", "-j", "0", "-f", "0.0", "-S", "3", "-S", "4", "-I", "3",
            "-I", "4", "-J", "3", "-J", "4", "-F", "3.0", "-F", "4.0",
        ])
    }

    #[test]
    fn test_parse_args() -> Result<(), String> {
        let mut flags = sample_flags();
        flags.parse_args(&sample_args())?;
        assert_new_values_match(&flags);
        Ok(())
    }

    #[test]
    fn test_parse_args_shorthand() -> Result<(), String> {
        let mut flags = sample_flags();
        flags.parse_args(&sample_args_shorthand())?;
        assert_new_values_match(&flags);
        Ok(())
    }

    fn sample_json() -> String {
        String::from(
            r#"
            {
               "my-bool": true,
               "my-string": "0",
               "my-int64": 0,
               "my-int128": 0,
               "my-float64": 0.0,
               "my-string-array": ["3","4"],
               "my-int64-array": [3,4],
               "my-int128-array": [3,4],
               "my-float64-array": [3.0,4.0]
            }"#,
        )
    }

    #[test]
    fn test_parse_json() -> Result<(), String> {
        let mut flags = sample_flags();
        Flags::parse_json(&mut flags.flag_values, &sample_json())?;
        assert_new_values_match(&flags);
        Ok(())
    }
}
