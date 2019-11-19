#[macro_use]
extern crate clap;
use clap::App;
use directories::{ProjectDirs};

use std::env;
use std::path::{Path, PathBuf};
use std::fs::{self, OpenOptions, File};
use std::io::{BufRead, BufReader, Write, Seek, SeekFrom};

mod create_app;

fn main()
{
    // let yaml = load_yaml!("cli.yml");
    // let matches = App::from(yaml).get_matches();
    let matches = create_app::create_app_object().get_matches();
    let projectDirectory = ProjectDirs::from("", "Sridaran Thoniyil", "RemindMe");

    if let None = projectDirectory {
        panic!("Cannot find base directory");
    }

    let projectDirectory = projectDirectory.unwrap();

    let dataDir = projectDirectory.data_dir();

    if !Path::exists(&dataDir)
    {
        fs::create_dir_all(&dataDir);
    }

    let defaultGroup = "general";

    let defaultGroupFileName = dataDir.join(&defaultGroup);

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

            let subMatches = subMatchesMaybe.unwrap();

            let fileToOpen = if let Some(groupName) = subMatches.value_of("group")
                {
                    println!("Got the group value!");
                    if groupName != ""
                    {
                        dataDir.join(groupName)
                    }
                    else
                    {
                        defaultGroupFileName
                    }
                }
                else
                {
                    // there is a default value, why is it none?
                    panic!("Else for group value? what?");
                };

            if !fileToOpen.exists()
            {
                println!("Must create the group first! {}", fileToOpen.display());
                return;
            }
            
            // true
            let input_is_substring: bool = subMatches.is_present("substring");
            
            let mut indices_to_mark: Option<Vec<usize>> = None;

            // treat as indices by default
            if !input_is_substring {
                // step 1: sort indices
                let input = subMatches.values_of("INPUT");
                match input {
                    Some(values_list) => {
                        // todo safety
                        indices_to_mark = Some(values_list
                            .map(|string| {
                                if let Ok(parsed) = string.parse::<usize>() {
                                    parsed
                                }
                                else
                                {
                                    println!("Invalid index: {}", string);
                                    panic!("!!!!");
                                    // return
                                }
                            })
                            .collect::<Vec<usize>>());
                    }
                    None => println!("Empty...")
                }

                let file_read = File::open(&fileToOpen);

                if file_read.is_err() {
                    println!("Cannot open file!");
                    return;
                }

                let mut indices = indices_to_mark.unwrap();
                indices.sort();

                let reader = BufReader::new(file_read.unwrap());
                let mut all_tasks: Vec<_> = reader.lines().collect();

                let mut deleted: usize = 0;

                for mut index in indices {
                    index -= 1;

                    if index < 0 {
                        println!("Invalid index: {}", index);
                        continue;
                    }

                    if index > all_tasks.len() {
                        return;
                    }

                    // todo handle err
                    all_tasks.remove(index - deleted);
                    deleted += 1;
                }

                // file_read.

                let mut file = OpenOptions::new()
                    .write(true)
                    .open(&fileToOpen)
                    .unwrap();
                
                file.set_len(0);
                file.seek(SeekFrom::Start(0));
                
                for line in all_tasks
                {
                    if let Ok(line) = line {
                        // todo handle err
                        writeln!(&mut file, "{}", line);
                    }
                }
            }
            else {
                // todo implement substring
            }
        },
        ("add", subMatchesMaybe) => {
            println!("Used add");

            let subMatches = subMatchesMaybe.unwrap();

            let mut string = "".to_string();
            
            let fileToOpen = {
                let groupName = subMatches.value_of("group").unwrap();

                if groupName != ""
                {
                    dataDir.join(groupName)
                }
                else
                {
                    defaultGroupFileName
                }
            };

            if let Some(listOfWords) = subMatches.values_of("INPUT")
            {
                listOfWords.for_each(|word| {
                        string.push_str(&word);
                        string.push_str(&" ".to_string());
                    }
                );
            }
            else
            {
                println!("No input passed in! Nothing to add!");
            }

            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(fileToOpen)
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

            let fileName = if let Some(subMatches) = subMatchesMaybe
                {
                    if let Some(groupName) = subMatches.value_of("group-name")
                    {
                        dataDir.join(groupName)
                    }
                    else
                    {
                        defaultGroupFileName
                    }
                }
                else
                {
                    defaultGroupFileName
                };

            if let Ok(fileToList) = File::open(&fileName)
            {
                let reader = BufReader::new(fileToList);

                for (index, line) in reader.lines().enumerate() {
                    let line = line.unwrap();
                    println!("{}. {}", index + 1, line);
                }
            }
            else
            {
                println!("Group not yet created");
            }
        }
    }
}