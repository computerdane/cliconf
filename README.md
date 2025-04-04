# cliconf

Dead-simple configuration for Rust CLI tools

## Example

Create a struct that defines your program's flags:

```rs
use cliconf::Parse;

#[derive(Parse)]
struct Conf {
    spanish: bool,

    #[cliconf(shorthand = 'n')]
    name: String,

    #[cliconf(shorthand = 'r')]
    repeat: i32,

    #[cliconf(shorthand = 'N', delimiter = ",")]
    extra_names: Vec<String>,
}
```

Initialize the struct, then parse configuration from environment variables and
command-line arguments:

```rs
let mut conf = Conf {
    spanish: false,
    name: "world".into(),
    repeat: 1,
    extra_names: vec![],
};
conf.parse_env(std::env::vars().collect());
conf.parse_args(std::env::args().skip(1).collect());
let conf = conf;
```

Use the config throughout your program:

```rs
let (and, hello) = if conf.spanish {
    ("y", "Hola")
} else {
    ("and", "Hello")
};

for _ in 0..conf.repeat {
    println!("{hello}, {}!", conf.name);
    for name in &conf.extra_names {
        println!(" {and} {hello}, {}!", name);
    }
}
```

Now, your program will automatically configure itself when given matching
environment variables and/or command-line arguments. Here are some examples:

```sh
hello
# Hello, world!

hello --name john
# Hello, john!

hello --name john --spanish
# Hola, john!

hello --name john --repeat 3
# Hello, john!
# Hello, john!
# Hello, john!

hello -n john -r 3
#Hello, john!
#Hello, john!
#Hello, john!

hello -n john -N aria -N scott -N allie
# Hello, john!
#  and Hello, aria!
#  and Hello, scott!
#  and Hello, allie!

NAME=john hello
# Hello, john!

NAME=john hello --name scott
# Hello, scott!

SPANISH=true NAME=john EXTRA_NAMES=aria,scott,allie hello
# Hola, john!
#  y Hola, aria!
#  y Hola, scott!
#  y Hola, allie!
```
