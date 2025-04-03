use core::panic;
use std::{
    any::type_name,
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::Read,
    path::Path,
};

use dirs::home_dir;
use flag_value::FlagValue;
use regex::Regex;
use serde_json::Value;

pub mod flag_value;

#[derive(Debug)]
struct StringError(String);

impl StringError {
    fn new(msg: String) -> Box<StringError> {
        Box::new(StringError(msg))
    }
}

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for StringError {}

pub struct Flag {
    name: String,
    default_value: Box<dyn FlagValue>,
    pub value: Box<dyn FlagValue>,
    pub shorthand: Option<char>,
    pub description: Option<String>,
    env_var_delimiter: Option<String>,
    pub exclude_from_usage: bool,
}

impl Flag {
    pub fn new(name: &str, default_value: impl FlagValue + Clone + 'static) -> Self {
        let re = Regex::new(r"^([a-z]|[0-9]|-)+$").expect("Failed to compile regex");
        if !re.is_match(name) {
            panic!(
                "Flag name '{}' is invalid! Must be lowercase a-z with dashes only.",
                name
            );
        }
        let value = default_value.clone();
        Flag {
            name: name.to_string(),
            default_value: Box::new(default_value),
            value: Box::new(value),
            shorthand: None,
            description: None,
            env_var_delimiter: None,
            exclude_from_usage: false,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }

    pub fn get_default_value(&self) -> &Box<dyn FlagValue> {
        &self.default_value
    }

    pub fn get_env_var_delimiter(&self) -> Option<String> {
        self.env_var_delimiter.clone()
    }

    pub fn get_env_var_name(&self) -> String {
        self.name.to_uppercase().replace("-", "_")
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
        if !self.value.is_vec() {
            panic!("env_var_delimiter() can only be used on Flags with a Vec<_> value");
        }
        self.env_var_delimiter = Some(env_var_delimiter.to_string());
        self
    }

    pub fn exclude_from_usage(mut self) -> Self {
        self.exclude_from_usage = true;
        self
    }
}

pub struct Flags {
    flags: HashMap<String, Flag>,
    shorthand_names: HashMap<char, String>,
    set_flags: HashSet<String>,
    positionals: Vec<String>,
    config_files: Vec<String>,
}

impl Flags {
    pub fn new() -> Self {
        Flags {
            flags: HashMap::new(),
            shorthand_names: HashMap::new(),
            set_flags: HashSet::new(),
            positionals: vec![],
            config_files: vec![],
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

        self.flags.insert(flag.name.clone(), flag);
    }

    pub fn get_flag_value(&self, name: &str) -> &Box<dyn FlagValue> {
        &self
            .flags
            .get(name)
            .expect(&format!("Unknown flag: {name}"))
            .value
    }

    pub fn get<T: FlagValue + Clone + 'static>(&self, name: &str) -> T {
        self.get_flag_value(name)
            .as_any()
            .downcast_ref::<T>()
            .expect(&format!(
                "Could not cast flag '{name}' to {}",
                type_name::<T>()
            ))
            .clone()
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

    fn parse_args(&mut self, args: Vec<String>) -> Result<(), Box<dyn Error>> {
        if args.len() == 0 {
            return Ok(());
        }

        let mut need_value_for_name: Option<String> = None;
        let mut as_positionals = false;

        for arg in args {
            if as_positionals {
                self.positionals.push(arg.to_string())
            } else if let Some(name) = need_value_for_name {
                let value = &mut self
                    .flags
                    .get_mut(&name)
                    .expect(&format!(
                        "need_value_for_name set for unknown flag '{name}'"
                    ))
                    .value;
                if !self.set_flags.contains(&name) {
                    value.clear();
                    self.set_flags.insert(name);
                }
                value.parse_and_set(&arg)?;
                need_value_for_name = None;
            } else if arg == "-" {
                // Some programs use "-" to signify that data will be read from
                // stdin, so we treat it as a positional argument
                self.positionals.push(arg.to_string());
            } else if arg == "--" {
                // "--" is a special flag that treats all of the remaining
                // arguments as positional arguments
                as_positionals = true;
            } else if arg.starts_with("--") {
                let name = &arg[2..];
                match self.flags.get_mut(name) {
                    Some(flag) => {
                        if flag.value.is_bool() {
                            flag.value.set_true();
                        } else {
                            need_value_for_name = Some(flag.name.clone());
                        }
                    }
                    None => return Err(StringError::new(format!("Unknown flag: --{name}"))),
                }
            } else if arg.starts_with("-") {
                let shorthands = &arg[1..];
                for c in shorthands.chars() {
                    match self.shorthand_names.get(&c) {
                        Some(name) => match self.flags.get_mut(name) {
                            Some(flag) => {
                                if flag.value.is_bool() {
                                    flag.value.set_true();
                                } else {
                                    need_value_for_name = Some(flag.name.clone());
                                }
                            }
                            None => panic!("shorthand_names contains key '{c}', but flags does not contain key '{name}'"),
                        },
                        None => return Err(StringError::new(format!("Unknown flag: -{c}"))),
                    }
                }
            } else {
                self.positionals.push(arg.to_string());
            }
        }

        Ok(())
    }

    fn parse_json(&mut self, json: String) -> Result<(), Box<dyn Error>> {
        let value = serde_json::from_str::<Value>(&json)?;
        if let Value::Object(map) = value {
            for (name, value) in map {
                if let Some(flag) = self.flags.get_mut(&name) {
                    if !self.set_flags.contains(&flag.name) {
                        flag.value.clear();
                        self.set_flags.insert(flag.name.clone());
                    }
                    if !flag.value.try_set_json(value) {
                        return Err(StringError::new(format!(
                            "JSON value for flag '{}' is of the wrong type",
                            flag.name
                        )));
                    }
                } else {
                    return Err(StringError::new(format!(
                        "Unknown flag found in JSON: {name}"
                    )));
                }
            }
            Ok(())
        } else {
            Err(StringError::new(format!(
                "Config must be a JSON Object with flag names as keys and flag values as values"
            )))
        }
    }

    pub fn load(
        &mut self,
        env_vars: HashMap<String, String>,
        args: Vec<String>,
    ) -> Result<(), Box<dyn Error>> {
        // 1. Config files
        for path in &self.config_files.clone() {
            if Path::new(path).exists() {
                match File::open(path) {
                    Ok(mut file) => {
                        let mut json = String::new();
                        if let Err(err) = file.read_to_string(&mut json) {
                            eprintln!("Failed to read config file '{path}': {err}")
                        } else if let Err(err) = self.parse_json(json) {
                            eprintln!("Config file '{path}' is invalid: {err}")
                        }
                    }
                    Err(err) => eprintln!("Failed to open config file '{path}': {err}"),
                }
            }
        }

        // 2. Environment variables
        self.set_flags.clear();
        for flag in self.flags.values_mut() {
            if let Some(value) = env_vars.get(&flag.get_env_var_name()) {
                if flag.value.is_vec() {
                    if let Some(delim) = flag.get_env_var_delimiter() {
                        for value in value.split(&delim) {
                            flag.value.parse_and_set(value)?;
                        }
                    } else {
                        eprintln!("Warning: Setting '{}' using the environment variable '{}' is unsupported.", flag.name, flag.get_env_var_name());
                    }
                } else {
                    flag.value.parse_and_set(value)?;
                }
            }
        }

        // 3. Args
        self.set_flags.clear();
        self.parse_args(args)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_flag() -> Flag {
        Flag::new("my-bool", false).shorthand('b')
    }

    #[test]
    #[should_panic]
    fn test_new_flag_invalid_name() {
        Flag::new("My invalid flag name!", false);
    }

    #[test]
    #[should_panic]
    fn test_new_flag_invalid_shorthand() {
        Flag::new("My invalid flag name!", false).shorthand('$');
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
        flags.add(Flag::new("my-bool", false).shorthand('b'));
        flags.add(Flag::new("my-string", "1".to_string()).shorthand('s'));
        flags.add(Flag::new("my-int64", 1i64).shorthand('i'));
        flags.add(Flag::new("my-int128", 1i128).shorthand('j'));
        flags.add(Flag::new("my-float64", 1.0).shorthand('f'));
        flags.add(Flag::new("my-string-array", to_string_vec(vec!["1", "2"])).shorthand('S'));
        flags.add(Flag::new("my-int64-array", vec![1i64, 2i64]).shorthand('I'));
        flags.add(Flag::new("my-int128-array", vec![1i128, 2i128]).shorthand('J'));
        flags.add(Flag::new("my-float64-array", vec![1.0, 2.0]).shorthand('F'));
        flags
    }

    #[test]
    fn test_get() {
        let flags = sample_flags();
        assert_eq!(flags.get::<bool>("my-bool"), false);
        assert_eq!(flags.get::<String>("my-string"), String::from("1"));
        assert_eq!(flags.get::<i64>("my-int64"), 1);
        assert_eq!(flags.get::<i128>("my-int128"), 1);
        assert_eq!(flags.get::<f64>("my-float64"), 1.0);
        assert_eq!(
            flags.get::<Vec<String>>("my-string-array"),
            to_string_vec(vec!["1", "2"])
        );
        assert_eq!(flags.get::<Vec<i64>>("my-int64-array"), vec![1, 2]);
        assert_eq!(flags.get::<Vec<i128>>("my-int128-array"), vec![1, 2]);
        assert_eq!(flags.get::<Vec<f64>>("my-float64-array"), vec![1.0, 2.0]);
    }

    fn assert_new_values_match(flags: &Flags) {
        assert_eq!(flags.get::<bool>("my-bool"), true);
        assert_eq!(flags.get::<String>("my-string"), "0");
        assert_eq!(flags.get::<i64>("my-int64"), 0);
        assert_eq!(flags.get::<i128>("my-int128"), 0);
        assert_eq!(flags.get::<f64>("my-float64"), 0.0);
        assert_eq!(
            flags.get::<Vec<String>>("my-string-array"),
            to_string_vec(vec!["3", "4"])
        );
        assert_eq!(flags.get::<Vec<i64>>("my-int64-array"), vec![3, 4]);
        assert_eq!(flags.get::<Vec<i128>>("my-int128-array"), vec![3, 4]);
        assert_eq!(flags.get::<Vec<f64>>("my-float64-array"), vec![3.0, 4.0]);
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
    fn test_parse_args() -> Result<(), Box<dyn Error>> {
        let mut flags = sample_flags();
        flags.parse_args(sample_args())?;
        assert_new_values_match(&flags);
        Ok(())
    }

    #[test]
    fn test_parse_args_shorthand() -> Result<(), Box<dyn Error>> {
        let mut flags = sample_flags();
        flags.parse_args(sample_args_shorthand())?;
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
    fn test_parse_json() -> Result<(), Box<dyn Error>> {
        let mut flags = sample_flags();
        flags.parse_json(sample_json())?;
        assert_new_values_match(&flags);
        Ok(())
    }
}
