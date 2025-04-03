use std::{
    any::{type_name, Any},
    error::Error,
};

use serde_json::Value;

macro_rules! panic_unsupported {
    ($f: expr, $t:expr) => {
        panic!("Cannot use {}() on type {}", $f, $t)
    };
}

pub trait FlagValue {
    fn as_bool(&self) -> bool {
        panic_unsupported!("as_bool", type_name::<Self>())
    }

    fn as_string(&self) -> String {
        panic_unsupported!("as_string", type_name::<Self>())
    }

    fn as_i64(&self) -> i64 {
        panic_unsupported!("as_i64", type_name::<Self>())
    }

    fn as_i128(&self) -> i128 {
        panic_unsupported!("as_i128", type_name::<Self>())
    }

    fn as_f64(&self) -> f64 {
        panic_unsupported!("as_f64", type_name::<Self>())
    }

    fn as_vec_string(&self) -> Vec<String> {
        panic_unsupported!("as_vec_string", type_name::<Self>())
    }

    fn as_vec_i64(&self) -> Vec<i64> {
        panic_unsupported!("as_vec_i64", type_name::<Self>())
    }

    fn as_vec_i128(&self) -> Vec<i128> {
        panic_unsupported!("as_vec_i128", type_name::<Self>())
    }

    fn as_vec_f64(&self) -> Vec<f64> {
        panic_unsupported!("as_vec_f64", type_name::<Self>())
    }

    fn as_any(&self) -> &dyn Any;

    fn is_bool(&self) -> bool {
        false
    }

    fn is_vec(&self) -> bool {
        false
    }

    fn parse_and_set(&mut self, s: &str) -> Result<(), Box<dyn Error>>;

    fn set_true(&mut self) {
        panic_unsupported!("set_true", type_name::<Self>())
    }

    fn clear(&mut self) {}

    fn try_set_json(&mut self, _: Value) -> bool {
        false
    }
}

impl FlagValue for bool {
    fn as_bool(&self) -> bool {
        *self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_bool(&self) -> bool {
        true
    }

    fn parse_and_set(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        *self = s.parse()?;
        Ok(())
    }

    fn set_true(&mut self) {
        *self = true;
    }

    fn try_set_json(&mut self, value: Value) -> bool {
        if let Value::Bool(v) = value {
            *self = v;
            return true;
        }
        return false;
    }
}

impl FlagValue for String {
    fn as_string(&self) -> String {
        self.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn parse_and_set(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        *self = s.to_string();
        Ok(())
    }

    fn try_set_json(&mut self, value: Value) -> bool {
        if let Value::String(v) = value {
            *self = v;
            return true;
        }
        return false;
    }
}

impl FlagValue for i64 {
    fn as_i64(&self) -> i64 {
        *self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn parse_and_set(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        *self = s.parse()?;
        Ok(())
    }

    fn try_set_json(&mut self, value: Value) -> bool {
        if let Value::Number(v) = value {
            if let Some(n) = v.as_i64() {
                *self = n;
                return true;
            }
        }
        return false;
    }
}

impl FlagValue for i128 {
    fn as_i128(&self) -> i128 {
        *self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn parse_and_set(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        *self = s.parse()?;
        Ok(())
    }

    fn try_set_json(&mut self, value: Value) -> bool {
        if let Value::Number(v) = value {
            if let Some(n) = v.as_i128() {
                *self = n;
                return true;
            }
        }
        return false;
    }
}

impl FlagValue for f64 {
    fn as_f64(&self) -> f64 {
        *self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn parse_and_set(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        *self = s.parse()?;
        Ok(())
    }

    fn try_set_json(&mut self, value: Value) -> bool {
        if let Value::Number(v) = value {
            if let Some(n) = v.as_f64() {
                *self = n;
                return true;
            }
        }
        return false;
    }
}

impl FlagValue for Vec<String> {
    fn as_vec_string(&self) -> Vec<String> {
        self.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_vec(&self) -> bool {
        true
    }

    fn parse_and_set(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        self.push(s.to_string());
        Ok(())
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn try_set_json(&mut self, value: Value) -> bool {
        if let Value::Array(array) = value {
            for value in array {
                if let Value::String(v) = value {
                    self.push(v);
                    continue;
                }
                return false;
            }
            return true;
        }
        return false;
    }
}

impl FlagValue for Vec<i64> {
    fn as_vec_i64(&self) -> Vec<i64> {
        self.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_vec(&self) -> bool {
        true
    }

    fn parse_and_set(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        self.push(s.parse()?);
        Ok(())
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn try_set_json(&mut self, value: Value) -> bool {
        if let Value::Array(array) = value {
            for value in array {
                if let Value::Number(v) = value {
                    if let Some(n) = v.as_i64() {
                        self.push(n);
                        continue;
                    }
                }
                return false;
            }
            return true;
        }
        return false;
    }
}

impl FlagValue for Vec<i128> {
    fn as_vec_i128(&self) -> Vec<i128> {
        self.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_vec(&self) -> bool {
        true
    }

    fn parse_and_set(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        self.push(s.parse()?);
        Ok(())
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn try_set_json(&mut self, value: Value) -> bool {
        if let Value::Array(array) = value {
            for value in array {
                if let Value::Number(v) = value {
                    if let Some(n) = v.as_i128() {
                        self.push(n);
                        continue;
                    }
                }
                return false;
            }
            return true;
        }
        return false;
    }
}

impl FlagValue for Vec<f64> {
    fn as_vec_f64(&self) -> Vec<f64> {
        self.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_vec(&self) -> bool {
        true
    }

    fn parse_and_set(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        self.push(s.parse()?);
        Ok(())
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn try_set_json(&mut self, value: Value) -> bool {
        if let Value::Array(array) = value {
            for value in array {
                if let Value::Number(v) = value {
                    if let Some(n) = v.as_f64() {
                        self.push(n);
                        continue;
                    }
                }
                return false;
            }
            return true;
        }
        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_casts() {
        false.as_bool();
        String::new().as_string();
        0i64.as_i64();
        0i128.as_i128();
        0f64.as_f64();
        vec![String::new()].as_vec_string();
        vec![0i64].as_vec_i64();
        vec![0i128].as_vec_i128();
        vec![0f64].as_vec_f64();
    }

    #[test]
    fn parse_and_set() -> Result<(), Box<dyn Error>> {
        false.parse_and_set("true")?;
        String::new().parse_and_set("1")?;
        0i64.parse_and_set("1")?;
        0i128.parse_and_set("1")?;
        0f64.parse_and_set("1.0")?;
        vec![String::new()].parse_and_set("1")?;
        vec![0i64].parse_and_set("1")?;
        vec![0i128].parse_and_set("1")?;
        vec![0f64].parse_and_set("1.0")?;
        Ok(())
    }

    #[test]
    fn bool_is_bool() {
        assert!(true.is_bool());
        assert!(false.is_bool());
    }

    #[test]
    #[should_panic]
    fn invalid_cast() {
        false.as_string();
    }
}
