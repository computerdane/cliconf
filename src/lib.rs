pub mod usage;

use std::{collections::HashMap, fs::File, io::Read, path::Path};

use dirs::home_dir;
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

pub struct Flag {
    name: String,
    default_value: FlagValue,
    pub value: FlagValue,
    pub shorthand: Option<char>,
    pub description: Option<String>,
    pub env_var_delimiter: Option<String>,
    pub exclude_from_usage: bool,

    is_set: bool,
}

impl Flag {
    pub fn new(name: &str, default_value: FlagValue) -> Self {
        let re = Regex::new(r"^([a-z]|[0-9]|-)+$").expect("Failed to compile regex");
        if !re.is_match(name) {
            panic!(
                "Flag name '{}' is invalid! Must be lowercase a-z with dashes only.",
                name
            );
        }
        let value = default_value.clone();
        Self {
            name: name.to_string(),
            default_value,
            value,
            shorthand: None,
            description: None,
            env_var_delimiter: None,
            exclude_from_usage: false,

            is_set: false,
        }
    }

    fn assert_types_match(&self, value: &FlagValue) {
        if std::mem::discriminant(&self.value) != std::mem::discriminant(value) {
            panic!(
                "Setting value for flag '{}' failed: Type mismatch.",
                self.name
            );
        }
    }

    pub fn set_value(&mut self, value: FlagValue) {
        self.assert_types_match(&value);
        self.value = value;
    }

    fn set_value_parsed(&mut self, value: String) -> Result<(), String> {
        let error_msg = |t: &str| {
            format!(
                "Failed to parse type {t} from flag '{}' with value '{value}'",
                self.name
            )
        };

        match &mut self.value {
            FlagValue::Bool(v) => match value.parse() {
                Ok(b) => *v = b,
                Err(_) => return Err(error_msg("bool")),
            },
            FlagValue::String(v) => *v = value,
            FlagValue::Int64(v) => match value.parse() {
                Ok(n) => *v = n,
                Err(_) => return Err(error_msg("i64")),
            },
            FlagValue::Int128(v) => match value.parse() {
                Ok(n) => *v = n,
                Err(_) => return Err(error_msg("i128")),
            },
            FlagValue::Float64(v) => match value.parse() {
                Ok(n) => *v = n,
                Err(_) => return Err(error_msg("f64")),
            },
            FlagValue::StringArray(_) => self.append_values(FlagValue::StringArray(vec![value])),
            FlagValue::Int64Array(_) => match value.parse() {
                Ok(n) => self.append_values(FlagValue::Int64Array(vec![n])),
                Err(_) => return Err(error_msg("i64")),
            },
            FlagValue::Int128Array(_) => match value.parse() {
                Ok(n) => self.append_values(FlagValue::Int128Array(vec![n])),
                Err(_) => return Err(error_msg("i128")),
            },
            FlagValue::Float64Array(_) => match value.parse() {
                Ok(n) => self.append_values(FlagValue::Float64Array(vec![n])),
                Err(_) => return Err(error_msg("f64")),
            },
        };

        Ok(())
    }

    fn update_array<T>(is_set: &mut bool, array: &mut Vec<T>, values: Vec<T>) {
        if *is_set {
            array.extend(values);
        } else {
            *array = values;
            *is_set = true;
        }
    }

    pub fn append_values(&mut self, value: FlagValue) {
        self.assert_types_match(&value);
        match &mut self.value {
            FlagValue::StringArray(curr) => {
                if let FlagValue::StringArray(values) = value {
                    Flag::update_array(&mut self.is_set, curr, values);
                } else {
                    panic!("Flag value type mismatch")
                }
            }
            FlagValue::Int64Array(curr) => {
                if let FlagValue::Int64Array(values) = value {
                    Flag::update_array(&mut self.is_set, curr, values);
                } else {
                    panic!("Flag value type mismatch")
                }
            }
            FlagValue::Int128Array(curr) => {
                if let FlagValue::Int128Array(values) = value {
                    Flag::update_array(&mut self.is_set, curr, values);
                } else {
                    panic!("Flag value type mismatch")
                }
            }
            FlagValue::Float64Array(curr) => {
                if let FlagValue::Float64Array(values) = value {
                    Flag::update_array(&mut self.is_set, curr, values);
                } else {
                    panic!("Flag value type mismatch")
                }
            }
            _ => panic!(
                "Cannot use append_values() on flag '{}': Flag is not an array.",
                self.name
            ),
        }
    }

    pub fn shorthand(mut self, c: char) -> Self {
        if !(('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z') || ('0' <= c && c <= '9')) {
            panic!("Flag shorthand '{c}' is invalid! Must be A-Z, a-z, or 0-9.",);
        }
        self.shorthand = Some(c);
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn env_var_delimiter(mut self, env_var_delimiter: &str) -> Self {
        self.env_var_delimiter = Some(env_var_delimiter.to_string());
        self
    }

    pub fn exclude_from_usage(mut self) -> Self {
        self.exclude_from_usage = true;
        self
    }

    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }

    pub fn get_default_value(&self) -> FlagValue {
        self.default_value.clone()
    }

    pub fn get_env_var_name(&self) -> String {
        return self.name.to_uppercase().replace("-", "_");
    }
}

pub struct Flags {
    flags: HashMap<String, Flag>,
    shorthand_names: HashMap<char, String>,
    positionals: Vec<String>,
    config_files: Vec<String>,
}

impl Flags {
    pub fn new() -> Self {
        Flags {
            flags: HashMap::new(),
            shorthand_names: HashMap::new(),
            positionals: vec![],
            config_files: vec![],
        }
    }

    pub fn add_config_file(&mut self, path: &str) {
        self.config_files.push(path.to_string())
    }

    pub fn add_home_config_file(&mut self, path: &str) {
        if let Some(home) = home_dir() {
            let path = home.join(path);
            let path = path.to_str().unwrap();
            self.add_config_file(path);
        } else {
            eprintln!("Warning: Could not locate user home directory!")
        }
    }

    pub fn add(&mut self, flag: Flag) {
        if self.flags.contains_key(&flag.name) {
            panic!("Flag with name '{}' already exists!", flag.name);
        }

        if let Some(c) = flag.shorthand {
            if self.shorthand_names.contains_key(&c) {
                panic!("Flag with shorthand '{c}' already exists!");
            }
            self.shorthand_names.insert(c, flag.name.to_owned());
        }

        self.flags.insert(flag.name.to_owned(), flag);
    }

    pub fn get(&self, name: &str) -> &Flag {
        self.flags
            .get(name)
            .expect(&format!("Failed to get unknown flag '{name}'"))
    }

    pub fn get_mut(&mut self, name: &str) -> &mut Flag {
        self.flags
            .get_mut(name)
            .expect(&format!("Failed to get unknown flag '{name}'"))
    }

    pub fn get_bool(&self, name: &str) -> bool {
        match self.flags.get(name) {
            Some(flag) => match flag.value {
                FlagValue::Bool(v) => v,
                _ => panic!("Flag '{name}' is not of type Bool!"),
            },
            None => panic!("Flag '{name}' does not exist!"),
        }
    }
    pub fn get_string(&self, name: &str) -> String {
        match self.flags.get(name) {
            Some(flag) => match flag.value.to_owned() {
                FlagValue::String(v) => v,
                _ => panic!("Flag '{name}' is not of type String!"),
            },
            None => panic!("Flag '{name}' does not exist!"),
        }
    }
    pub fn get_i64(&self, name: &str) -> i64 {
        match self.flags.get(name) {
            Some(flag) => match flag.value {
                FlagValue::Int64(v) => v,
                _ => panic!("Flag '{name}' is not of type i64!"),
            },
            None => panic!("Flag '{name}' does not exist!"),
        }
    }
    pub fn get_i128(&self, name: &str) -> i128 {
        match self.flags.get(name) {
            Some(flag) => match flag.value {
                FlagValue::Int128(v) => v,
                _ => panic!("Flag '{name}' is not of type i128!"),
            },
            None => panic!("Flag '{name}' does not exist!"),
        }
    }
    pub fn get_f64(&self, name: &str) -> f64 {
        match self.flags.get(name) {
            Some(flag) => match flag.value {
                FlagValue::Float64(v) => v,
                _ => panic!("Flag '{name}' is not of type f64!"),
            },
            None => panic!("Flag '{name}' does not exist!"),
        }
    }
    pub fn get_string_array(&self, name: &str) -> Vec<String> {
        match self.flags.get(name) {
            Some(flag) => match flag.value.to_owned() {
                FlagValue::StringArray(v) => v,
                _ => panic!("Flag '{name}' is not of type Vec<String>!"),
            },
            None => panic!("Flag '{name}' does not exist!"),
        }
    }
    pub fn get_i64_array(&self, name: &str) -> Vec<i64> {
        match self.flags.get(name) {
            Some(flag) => match flag.value.to_owned() {
                FlagValue::Int64Array(v) => v,
                _ => panic!("Flag '{name}' is not of type Vec<i64>!"),
            },
            None => panic!("Flag '{name}' does not exist!"),
        }
    }
    pub fn get_i128_array(&self, name: &str) -> Vec<i128> {
        match self.flags.get(name) {
            Some(flag) => match flag.value.to_owned() {
                FlagValue::Int128Array(v) => v,
                _ => panic!("Flag '{name}' is not of type Vec<i128>!"),
            },
            None => panic!("Flag '{name}' does not exist!"),
        }
    }
    pub fn get_f64_array(&self, name: &str) -> Vec<f64> {
        match self.flags.get(name) {
            Some(flag) => match flag.value.to_owned() {
                FlagValue::Float64Array(v) => v,
                _ => panic!("Flag '{name}' is not of type Vec<f64>!"),
            },
            None => panic!("Flag '{name}' does not exist!"),
        }
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
                // Flags::parse_string_and_set(&mut self.flags, &need_value_for_name, arg)?;
                self.flags
                    .get_mut(&need_value_for_name)
                    .expect("need_value_for_name set for unknown flag")
                    .set_value_parsed(arg)?;
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
                match self.flags.get_mut(name) {
                    Some(flag) => match &mut flag.value {
                        FlagValue::Bool(v) => *v = true,
                        _ => need_value_for_name = name.to_string(),
                    },
                    None => return Err(format!("Unknown flag: --{name}")),
                }
            } else if arg.starts_with("-") {
                let shorthands = &arg[1..];
                for c in shorthands.chars() {
                    match self.shorthand_names.get(&c) {
                        Some(name) => match self.flags.get_mut(name) {
                            Some(flag) => match &mut flag.value {
                            FlagValue::Bool(v) => *v = true,
                            _ => need_value_for_name = name.to_string(),
                            },
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

    fn parse_json(flags: &mut HashMap<String, Flag>, json: &String) -> Result<(), String> {
        match serde_json::from_str::<Value>(&json) {
            Ok(data) => match data {
                Value::Object(map) => {
                    for (name, value) in map {
                        let name = name.as_str();
                        match flags.get_mut(name) {
                            Some(flag) => match value {
                                Value::Bool(b) => match &mut flag.value {
                                    FlagValue::Bool(v) => *v = b,
                                    _ => return Err(format!("Property '{name}' is not of type bool!"))
                                },
                                Value::String(s) => match &mut flag.value {
                                    FlagValue::String(v) => *v = s,
                                    _ => return Err(format!("Property '{name}' is not of type string!"))
                                },
                                Value::Number(number) => match &mut flag.value {
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
                                            Value::String(s) => match &mut flag.value {
                                                FlagValue::StringArray(v) => if i == 0 { *v = vec![s.to_string()]} else {v.push(s.to_string())},
                                                _ => return Err(format!("Property '{name}' is not of type string[]!"))
                                            },
                                            Value::Number(number) => match &mut flag.value {
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
                        if let Err(err) = Flags::parse_json(&mut self.flags, &json) {
                            eprintln!("Config file '{path}' is invalid: {err}")
                        }
                    }
                    Err(err) => eprintln!("Failed to open config file '{path}': {err}"),
                }
            }
        }

        // 2. Environment variables
        for flag in self.flags.values_mut() {
            if let Some(value) = env_vars.get(&flag.get_env_var_name()) {
                match flag.default_value {
                    FlagValue::StringArray(_)
                    | FlagValue::Int64Array(_)
                    | FlagValue::Int128Array(_)
                    | FlagValue::Float64Array(_) => {
                        let delim = flag.env_var_delimiter.to_owned();
                        match delim {
                        Some(delim) => for item in value.split(&delim) {
                            flag.set_value_parsed(item.to_string())?;
                        },
                        None => eprintln!(
                            "Warning: Setting '{}' using the environment variable '{}' is unsupported.",
                            flag.name,
                            flag.get_env_var_name()
                        ),
                    }
                    }
                    _ => flag.set_value_parsed(value.to_string())?,
                }
            }
        }
        for flag in self.flags.values_mut() {
            flag.is_set = false;
        }

        // 3. Args
        self.parse_args(args)?;

        Ok(())
    }

    pub fn positionals(&self) -> &Vec<String> {
        &self.positionals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_flag() -> Flag {
        Flag::new("my-bool", FlagValue::Bool(false)).shorthand('b')
    }

    #[test]
    #[should_panic]
    fn test_new_flag_invalid_name() {
        Flag::new("My invalid flag name!", FlagValue::Bool(false));
    }

    #[test]
    #[should_panic]
    fn test_new_flag_invalid_shorthand() {
        Flag::new("My invalid flag name!", FlagValue::Bool(false)).shorthand('$');
    }

    #[test]
    fn test_new_flag() {
        sample_flag();
    }

    #[test]
    fn test_flag_env_var_name() {
        assert!(sample_flag().get_env_var_name() == "MY_BOOL")
    }

    fn to_string_vec(strs: Vec<&str>) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    fn sample_flags() -> Flags {
        let mut flags = Flags::new();
        flags.add(Flag::new("my-bool", FlagValue::Bool(false)).shorthand('b'));
        flags.add(Flag::new("my-string", FlagValue::String("1".into())).shorthand('s'));
        flags.add(Flag::new("my-int64", FlagValue::Int64(1)).shorthand('i'));
        flags.add(Flag::new("my-int128", FlagValue::Int128(1)).shorthand('j'));
        flags.add(Flag::new("my-float64", FlagValue::Float64(1.0)).shorthand('f'));
        flags.add(
            Flag::new(
                "my-string-array",
                FlagValue::StringArray(to_string_vec(vec!["1", "2"])),
            )
            .shorthand('S'),
        );
        flags.add(Flag::new("my-int64-array", FlagValue::Int64Array(vec![1, 2])).shorthand('I'));
        flags.add(Flag::new("my-int128-array", FlagValue::Int128Array(vec![1, 2])).shorthand('J'));
        flags.add(
            Flag::new("my-float64-array", FlagValue::Float64Array(vec![1.0, 2.0])).shorthand('F'),
        );
        flags
    }

    #[test]
    fn test_get() {
        let flags = sample_flags();
        assert_eq!(flags.get_bool("my-bool"), false);
        assert_eq!(flags.get_string("my-string"), String::from("1"));
        assert_eq!(flags.get_i64("my-int64"), 1);
        assert_eq!(flags.get_i128("my-int128"), 1);
        assert_eq!(flags.get_f64("my-float64"), 1.0);
        assert_eq!(
            flags.get_string_array("my-string-array"),
            to_string_vec(vec!["1", "2"])
        );
        assert_eq!(flags.get_i64_array("my-int64-array"), vec![1, 2]);
        assert_eq!(flags.get_i128_array("my-int128-array"), vec![1, 2]);
        assert_eq!(flags.get_f64_array("my-float64-array"), vec![1.0, 2.0]);
    }

    fn assert_new_values_match(flags: &Flags) {
        assert_eq!(flags.get_bool("my-bool"), true);
        assert_eq!(flags.get_string("my-string"), "0");
        assert_eq!(flags.get_i64("my-int64"), 0);
        assert_eq!(flags.get_i128("my-int128"), 0);
        assert_eq!(flags.get_f64("my-float64"), 0.0);
        assert_eq!(
            flags.get_string_array("my-string-array"),
            to_string_vec(vec!["3", "4"])
        );
        assert_eq!(flags.get_i64_array("my-int64-array"), vec![3, 4]);
        assert_eq!(flags.get_i128_array("my-int128-array"), vec![3, 4]);
        assert_eq!(flags.get_f64_array("my-float64-array"), vec![3.0, 4.0]);
    }

    #[test]
    fn test_set_value_parsed() -> Result<(), String> {
        let mut flags = sample_flags();
        flags.get_mut("my-bool").set_value_parsed("true".into())?;
        flags.get_mut("my-string").set_value_parsed("0".into())?;
        flags.get_mut("my-int64").set_value_parsed("0".into())?;
        flags.get_mut("my-int128").set_value_parsed("0".into())?;
        flags.get_mut("my-float64").set_value_parsed("0.0".into())?;
        flags
            .get_mut("my-string-array")
            .set_value_parsed("3".into())?;
        flags
            .get_mut("my-string-array")
            .set_value_parsed("4".into())?;
        flags
            .get_mut("my-int64-array")
            .set_value_parsed("3".into())?;
        flags
            .get_mut("my-int64-array")
            .set_value_parsed("4".into())?;
        flags
            .get_mut("my-int128-array")
            .set_value_parsed("3".into())?;
        flags
            .get_mut("my-int128-array")
            .set_value_parsed("4".into())?;
        flags
            .get_mut("my-float64-array")
            .set_value_parsed("3.0".into())?;
        flags
            .get_mut("my-float64-array")
            .set_value_parsed("4.0".into())?;
        assert_new_values_match(&flags);
        Ok(())
    }

    fn sample_args() -> Vec<String> {
        to_string_vec(vec![
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
        to_string_vec(vec![
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
        Flags::parse_json(&mut flags.flags, &sample_json())?;
        assert_new_values_match(&flags);
        Ok(())
    }
}
