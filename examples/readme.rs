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

fn main() {
    let mut conf = Conf {
        spanish: false,
        name: "world".into(),
        repeat: 1,
        extra_names: vec![],
    };
    conf.parse_env(std::env::vars().collect());
    conf.parse_args(std::env::args().skip(1).collect());
    let conf = conf;

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
}
