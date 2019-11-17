#[macro_use]
extern crate clap;
use clap::App;
use directories::UserDirs;

use std::env;
use std::fs::{OpenOptions, File};
use std::io::{BufRead, BufReader, Write};

mod create_app;

fn main()
{
    // let yaml = load_yaml!("cli.yml");
    // let matches = App::from(yaml).get_matches();
    let matches = create_app::create_app_object().get_matches();
    
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

    let baseTodoFileName = "todo";

    let todoFileName = baseDirectoryName.join(&baseTodoFileName);

    match matches.subcommand()
    {
        ("finish", subMatchesMaybe) => {
            println!("Used finish");

            /*
            finish takes in either
                a) index of the task in the file
                b) substring of the task
            if index is provided, add "FINISHED"
            if substring provided, list all tasks that match,
                and then do same index thing
            */

            // figure out if index or substring
            if let Some(subMatches) = subMatchesMaybe {
                let finishGroup;
                if let Some(groupName) = subMatches.value_of("group")
                {
                    finishGroup = groupName;
                }
                else
                {
                    // default value
                    finishGroup = baseTodoFileName;
                }

                
            }
        },
        ("add", subMatchesMaybe) => {
            println!("Used add");

            let mut string = "".to_string();

            if let Some(subMatches) = subMatchesMaybe {
                if let Some(groupName) = subMatches.value_of("group-name") {
                    println!("group name: {}", groupName);
                }
                if let Some(listOfWords) = subMatches.values_of("INPUT") {
                    if let Some(words) = subMatches.value_of("INPUT") {
                        println!("first value: {}", words);
                    }
                    let mut word_count = 0;
                    listOfWords.for_each(|word| {
                            string.push_str(&word);
                            string.push_str(&" ".to_string());
                            word_count += 1;
                        }
                    );
                    println!("Word count: {}", word_count);
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
        ("config", subMatchesMaybe) => {
            if let Some(subMatches) = subMatchesMaybe {
                if let Some(sub_group) = subMatches.value_of("group") {
                    println!("Group has a value: {}", sub_group);
                }
                else
                {
                    println!("Group does not have a value");
                }
            }
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