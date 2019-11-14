#[macro_use]
extern crate clap;
use clap::App;

fn main()
{
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    if let Some(c) = matches.value_of("set_file")
    {
        // TODO handle
        return;
    }
}