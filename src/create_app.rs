extern crate clap;
use clap::{App, AppSettings, Arg, SubCommand};

pub fn create_app_object() -> App<'static, 'static> {
    App::new("remind-me")
        .version("1.0")
        .author("Sridaran Thoniyil")
        .about("Reminds user to do things")
        .setting(AppSettings::DontDelimitTrailingValues)
        .setting(AppSettings::TrailingVarArg)
        .subcommand(
            SubCommand::with_name("finish")
                .about("Marks task as completed")
                .version("1.0")
                .author("Sridaran T.")
                .visible_alias("-")
                .args(
                    &[
                        Arg::with_name("group")
                            .short("g")
                            .long("group")
                            .help("Name of the group to search for task in")
                            .required(true)
                            .default_value(""),
                        Arg::with_name("index")
                            .short("i")
                            .long("index")
                            .help("Flag that indicates that you are trying to pass in indices")
                            .conflicts_with("substring"),
                        Arg::with_name("substring")
                            .short("s")
                            .long("substring")
                            .help("Flag that indicates that you are trying to pass in a substring or substrings")
                            .conflicts_with("index"),
                        Arg::with_name("INPUT")
                            .help("Takes in index or substring of task in group to mark completed")
                            .required(true)
                            .min_values(1)
                            .index(1)
                    ]
                ))
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds task to be completed")
                .version("1.0")
                .author("Sridaran T.")
                .visible_alias("+")
                .args(
                    &[
                        Arg::with_name("group")
                            .short("g")
                            .long("group")
                            .help("Name of the group to search for task in")
                            .required(true)
                            .default_value(""),
                        Arg::with_name("INPUT")
                            .help("Takes in string to be added to group")
                            .required(true)
                            .min_values(1)
                            .index(1)
                    ]
                ))
        .subcommand(
            SubCommand::with_name("list")
                .about("List current tasks to do")
                .version("1.0")
                .author("Sridaran T.")
                .alias("l")
                .arg(
                    Arg::with_name("group-name")
                        .help("Name of the group to search for task in")
                        .index(1)
                        .takes_value(true)
                ))
        .subcommand(
            SubCommand::with_name("group")
                .about("Handles all group-related commands")
                .version("1.0")
                .author("Sridaran T.")
                .subcommands(
                    vec![
                        SubCommand::with_name("create")
                            .about("Creates a new group")
                            .version("1.0")
                            .author("Sridaran T.")
                            .arg(
                                Arg::with_name("group-name")
                                    .required(true)
                                    .index(1)
                            ),
                        SubCommand::with_name("delete")
                            .about("Deletes an existing group")
                            .version("1.0")
                            .author("Sridaran T.")
                            .arg(
                                Arg::with_name("group-name")
                                    .required(true)
                                    .index(1)
                            ),
                        SubCommand::with_name("clear")
                            .about("Name of the group to be cleared")
                            .version("1.0")
                            .author("Sridaran T.")
                            .arg(
                                Arg::with_name("group-name")
                                    .required(true)
                                    .index(1)
                            )
                    ]
                ))
        .subcommand(
            SubCommand::with_name("config")
                .about("Edit configuration of RemindMe")
                .version("1.0")
                .author("Sridaran T.")
                .args(
                    &vec![
                        Arg::with_name("print_configuration")
                            .help("Prints all current configuration options")
                            .long("print")
                            .short("p"),
                        Arg::with_name("default")
                            .help("Sets the default group")
                            .long("default")
                            .short("d")
                            .takes_value(true)
                    ]
                )
                
        )       
}