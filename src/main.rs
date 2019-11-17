#[macro_use]
extern crate clap;
use clap::App;
use directories::UserDirs;

use std::env;
use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, File};
use std::io::{BufRead, BufReader, Write};

mod create_app;

fn main()
{
    // let yaml = load_yaml!("cli.yml");
    // let matches = App::from(yaml).get_matches();
    let matches = create_app::create_app_object().get_matches();
    
    let baseDirectoryName: PathBuf;
    if let Some(user_dirs) = UserDirs::new()
    {
        baseDirectoryName = user_dirs.home_dir().join(".remindme/");
    }
    else
    {
        panic!("Cannot find base directory");
    }

    let defaultGroup = "general";

    let defaultGroupFileName = baseDirectoryName.join(&defaultGroup);

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

            let fileToOpen = match subMatchesMaybe {
                Some(subMatches) => {
                    if let Some(groupName) = subMatches.value_of("group")
                    {
                        println!("Got the group value!");
                        baseDirectoryName.join(groupName)
                    }
                    else
                    {
                        defaultGroupFileName
                    }
                },
                None => panic!("No matches in finish subMatchesMaybe match; todo")
            };

            if !fileToOpen.exists()
            {
                println!("Must create the group first! {}", fileToOpen.display());
                return;
            }

            // let mut file = OpenOptions::new()
            //     .read(true)
            //     .write(true)
            //     .create(false)
            //     .open(fileToOpen)
            //     .unwrap();
            
            // true
            let input_is_substring: bool = match subMatchesMaybe {
                Some(subMatches) => {
                    subMatches.is_present("substring")
                },
                None => panic!("No matches in finish subMatchesMaybe match 2; todo")
            };
            
            println!("Input_is_substring: {}", input_is_substring);

            // treat as indices by default

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
                .create(true)
                .open(defaultGroupFileName)
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
            let todoFile = File::open(defaultGroupFileName).unwrap();
            let reader = BufReader::new(todoFile);

            for (index, line) in reader.lines().enumerate() {
                let line = line.unwrap();
                println!("{}. {}", index + 1, line);
            }
        }
    }
}