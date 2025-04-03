use cliconf::CliConf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(CliConf, Default, Serialize, Deserialize)]
struct Conf {
    my_bool: bool,
    my_string: String,
    my_num: i32,
}

fn assertions(c: &Conf) {
    assert_eq!(c.my_bool, true);
    assert_eq!(c.my_string, "1");
    assert_eq!(c.my_num, 1);
}

#[test]
fn test_env() {
    let mut c = Conf::default();

    let mut vars: HashMap<&str, &str> = HashMap::new();
    vars.insert("MY_BOOL", "true");
    vars.insert("MY_STRING", "1");
    vars.insert("MY_NUM", "1");
    let vars: HashMap<String, String> = vars
        .iter()
        .map(|(key, val)| (key.to_string(), val.to_string()))
        .collect();

    c.parse_env(vars);
    assertions(&c);
}

#[test]
fn test_args() {
    let mut c = Conf::default();

    let args: Vec<String> = vec!["--my-bool", "--my-string", "1", "--my-num", "1"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    c.parse_args(args);
    assertions(&c);
}

#[test]
fn test_json() {
    let data = r#"
        {
            "my_bool": true,
            "my_string": "1",
            "my_num": 1
        }
    "#;

    let c: Conf = serde_json::from_str(data).unwrap();

    assertions(&c);
}
