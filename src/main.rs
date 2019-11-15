#[macro_use]
extern crate clap;
use clap::App;
use directories::UserDirs;

use std::env;
use std::fs::{OpenOptions, File};
use std::io::{BufRead, BufReader, Write};

fn main()
{
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    if let Some(c) = matches.value_of("set_file")
    {
        // TODO handle
        return;
    }

    let baseDirectoryName;
    if let Some(user_dirs) = UserDirs::new()
    {
        baseDirectoryName = user_dirs.home_dir().join(".remindme/");
        // println!("{}", baseDirectoryName.display());
    }
    else
    {
        panic!("Cannot find base directory");
    }

    let todoFileName = baseDirectoryName.join("todo");

    match matches.subcommand()
    {
        ("finish", subMatchesMaybe) => {
            println!("Used finish");
        },
        ("add", subMatchesMaybe) => {
            println!("Used add");

            let mut string = "".to_string();

            if let Some(subMatches) = subMatchesMaybe {
                if let Some(listOfWords) = subMatches.values_of("INPUT") {
                    listOfWords.for_each(|word| {
                        string.push_str(&word);
                        string.push_str(&" ".to_string())
                        }
                    );
                    // println!("Full string: \"{}\"", string);
                }
                else
                {
                    // what does this mean?
                }
            }            
            else
            {
                // added empty string
                // TODO
            }

            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(todoFileName)
                .unwrap();
            
            // if let Err(e) = writeln!(file, "{}", string)
            // {
            //     eprintln!("{}", e);
            // }

            file.write_fmt(format_args!("{}\n", string));
                
        },
        // list is the default subcommand
        (_, subMatchesMaybe) => {
            println!("Used list");
            let todoFile = File::open(todoFileName).unwrap();
            let reader = BufReader::new(todoFile);

            for (index, line) in reader.lines().enumerate() {
                let line = line.unwrap();
                println!("{}. {}", index + 1, line);
            }
        }
    }
}