use std::{collections::HashMap, env, fs::File, io::Read, path::Path};

use regex::Regex;

#[derive(Clone)]
pub enum FlagValue {
    BoolValue(bool),
    StringValue(String),
    Int32Value(i32),
    Int64Value(i64),
    Float32Value(f32),
    Float64Value(f64),
    StringArrayValue(Vec<String>),
    Int32ArrayValue(Vec<i32>),
    Int64ArrayValue(Vec<i64>),
    Float32ArrayValue(Vec<f32>),
    Float64ArrayValue(Vec<f64>),
}

pub struct Flag<'a> {
    pub name: &'a str,
    pub shorthand: Option<char>,
    pub default_value: FlagValue,
    pub env_var_delimeter: &'a str,
    pub description: &'a str,
    pub exclude_from_usage: bool,
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

    pub fn get_i32(&self, name: &str) -> &i32 {
        match self.flag_values.get(name) {
            Some(FlagValue::Int32Value(v)) => v,
            _ => panic!("Flag '{name}' is not of type i32!"),
        }
    }

    pub fn get_i64(&self, name: &str) -> &i64 {
        match self.flag_values.get(name) {
            Some(FlagValue::Int64Value(v)) => v,
            _ => panic!("Flag '{name}' is not of type i64!"),
        }
    }

    pub fn get_f32(&self, name: &str) -> &f32 {
        match self.flag_values.get(name) {
            Some(FlagValue::Float32Value(v)) => v,
            _ => panic!("Flag '{name}' is not of type f32!"),
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

    pub fn get_i32_array(&self, name: &str) -> &Vec<i32> {
        match self.flag_values.get(name) {
            Some(FlagValue::Int32ArrayValue(v)) => v,
            _ => panic!("Flag '{name}' is not of type Vec<i32>!"),
        }
    }

    pub fn get_i64_array(&self, name: &str) -> &Vec<i64> {
        match self.flag_values.get(name) {
            Some(FlagValue::Int64ArrayValue(v)) => v,
            _ => panic!("Flag '{name}' is not of type Vec<i64>!"),
        }
    }

    pub fn get_f32_array(&self, name: &str) -> &Vec<f32> {
        match self.flag_values.get(name) {
            Some(FlagValue::Float32ArrayValue(v)) => v,
            _ => panic!("Flag '{name}' is not of type Vec<f32>!"),
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
        name: &String,
        value: String,
    ) -> Result<(), String> {
        let error_msg =
            |t: &str| format!("Failed to parse type {t} from flag '{name}' with value '{value}'");

        let name = name.as_str();

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
            Some(FlagValue::Int32Value(_)) => Gears::set(
                flag_values,
                name,
                FlagValue::Int32Value(match value.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(error_msg("i32")),
                }),
            ),
            Some(FlagValue::Int64Value(_)) => Gears::set(
                flag_values,
                name,
                FlagValue::Int64Value(match value.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(error_msg("i64")),
                }),
            ),
            Some(FlagValue::Float32Value(_)) => Gears::set(
                flag_values,
                name,
                FlagValue::Float32Value(match value.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(error_msg("f32")),
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
            Some(FlagValue::Int32ArrayValue(_)) => todo!(),
            Some(FlagValue::Int64ArrayValue(_)) => todo!(),
            Some(FlagValue::Float32ArrayValue(_)) => todo!(),
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
            } else if arg == "-" {
                self.positionals.push(arg)
            } else if arg == "--" {
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

    pub fn load(&mut self, args: Vec<String>) -> Result<(), String> {
        // 1. Config files
        for path in &self.config_files {
            if Path::new(path).exists() {
                match File::open(path) {
                    Ok(mut file) => {
                        let mut contents = String::new();
                        if let Err(err) = file.read_to_string(&mut contents) {
                            eprintln!("Failed to read config file '{path}': {err}")
                        }
                        // parse json
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
