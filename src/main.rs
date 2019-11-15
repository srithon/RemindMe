#[macro_use]
extern crate clap;
use clap::App;
use directories::UserDirs;

fn main()
{
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    if let Some(c) = matches.value_of("set_file")
    {
        // TODO handle
        return;
    }

    let baseDirectory;
    if let Some(user_dirs) = UserDirs::new()
    {
        baseDirectory = user_dirs.home_dir().join(".remindme/");
        // println!("{}", baseDirectory.display());
    }
    else
    {
        panic!("Cannot find base directory");
    }

    let todoFile = baseDirectory.join("todo");

    match matches.subcommand()
    {
        ("finish", subMatchesMaybe) => {
            println!("Used finish");
        },
        ("add", subMatchesMaybe) => {
            println!("Used add");
        },
        // list is the default subcommand
        (_, subMatchesMaybe) => {
            println!("Used list");
        }
    }
}