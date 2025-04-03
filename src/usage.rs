use std::{
    cmp::min,
    io::{self, Write},
};

use crate::{FlagValue, Flags};

pub fn generate<W: Write>(flags: &Flags, width: usize, w: &mut W) -> io::Result<()> {
    let indentation = "    ";
    let max_desc_width = width - indentation.len();

    let mut names: Vec<String> = flags.flags.keys().cloned().collect();
    names.sort();
    let names = names;

    for (n, name) in names.iter().enumerate() {
        let flag = flags.get(name);
        if let None = flag.description {
            continue;
        }
        if flag.exclude_from_usage {
            continue;
        }

        w.write(b"--")?;
        w.write(flag.name.as_bytes())?;
        if let Some(c) = flag.shorthand {
            w.write(b" / -")?;
            w.write(&[c as u8])?;
        }
        w.write(b"\n")?;

        let mut desc = flag.description.as_ref().unwrap().to_string();
        let mut append_default_value = |value: String| {
            desc += &format!(" (default: {value})");
        };

        match flag.default_value.clone() {
            FlagValue::Bool(v) => append_default_value(v.to_string()),
            FlagValue::String(v) => append_default_value(v),
            FlagValue::Int64(v) => append_default_value(v.to_string()),
            FlagValue::Int128(v) => append_default_value(v.to_string()),
            FlagValue::Float64(v) => append_default_value(v.to_string()),
            FlagValue::StringArray(a) => append_default_value(format!("[{}]", a.join(", "))),
            FlagValue::Int64Array(a) => {
                let strings: Vec<String> = a.iter().map(|v| v.to_string()).collect();
                append_default_value(format!("[{}]", strings.join(", ")))
            }
            FlagValue::Int128Array(a) => {
                let strings: Vec<String> = a.iter().map(|v| v.to_string()).collect();
                append_default_value(format!("[{}]", strings.join(", ")))
            }
            FlagValue::Float64Array(a) => {
                let strings: Vec<String> = a.iter().map(|v| v.to_string()).collect();
                append_default_value(format!("[{}]", strings.join(", ")))
            }
        }

        let mut l = 0;
        while l < desc.len() {
            let remaining = desc.len() - l;
            let max_wrapped_width = min(max_desc_width, remaining);
            let mut wrapped_width = max_wrapped_width;
            let chars: Vec<char> = desc.chars().collect();
            while remaining > max_desc_width && chars[l + wrapped_width - 1] != ' ' {
                if wrapped_width == 0 {
                    wrapped_width = max_wrapped_width;
                    break;
                }
                wrapped_width -= 1;
            }
            w.write(format!("{indentation}{}\n", &desc[l..l + wrapped_width]).as_bytes())?;
            l += wrapped_width;
        }

        if n != names.len() - 1 {
            w.write(b"\n")?;
        }
    }

    w.flush()
}

pub fn generate_string(flags: &Flags, width: usize) -> String {
    let mut w = Vec::new();
    generate(flags, width, &mut w).expect("Failed to generate usage");
    String::from_utf8(w).expect("Failed to get usage string as utf-8")
}

#[cfg(test)]
mod tests {
    use crate::Flag;

    use super::*;

    #[test]
    fn test_generate() {
        let mut flags = Flags::new();
        flags.add(
            Flag::new("name", FlagValue::String("john".into()))
                .shorthand('n')
                .description("The person we want to greet"),
        );
        flags.add(
            Flag::new("long", FlagValue::String("long".into()))
                .shorthand('l')
                .description("A flag with a super duper long description. Like, this is a very long description and is totally overwhelming the user. We really need to stop making things so long and complicated guys. The poor users can't handle it!"),
        );
        flags.add(
            Flag::new("zzz", FlagValue::Bool(false)).description("An argument with no shorthand!"),
        );
        flags.add(
            Flag::new("excluded", FlagValue::Bool(false))
                .description("This flag is excluded from the usage string")
                .exclude_from_usage(),
        );

        let target = "--long / -l
    A flag with a super duper long description. Like, this is a very long 
    description and is totally overwhelming the user. We really need to stop 
    making things so long and complicated guys. The poor users can't handle it! 
    (default: long)

--name / -n
    The person we want to greet (default: john)

--zzz
    An argument with no shorthand! (default: false)
";

        let result = generate_string(&flags, 80);
        assert_eq!(result, target);
    }
}
