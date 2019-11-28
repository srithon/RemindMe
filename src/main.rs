#[macro_use]
extern crate clap;
use clap::App;
use directories::{ProjectDirs};

use std::env;
use std::path::{Path, PathBuf};
use std::fs::{self, OpenOptions, File};
use std::io::{stdin, stdout, BufRead, BufReader, Write, Seek, SeekFrom};

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

            let input = subMatches.values_of("INPUT");

            // treat as indices by default
            if !input_is_substring {
                // step 1: sort indices
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
                let substring_to_search_for: String = match input {
                    Some(substring_words_list) => {
                        substring_words_list.collect::<Vec<&str>>().join(" ")
                    },
                    None => {
                        println!("Nothing passed in as input");
                        return;
                    }
                };

                // look through the lines and find all that match
                // print them out on screen
                // let user specify indices or * to delete all
                // if user has "no-prompt" option specified, default to *

                if let Ok(file) = File::open(&fileToOpen) {
                    let lines: Vec<(String, bool, u16)> = {
                        let reader = BufReader::new(file);
                        let mut current_line = 0;
                        reader.lines().filter_map(|line| {
                            current_line += 1;
                            if let Ok(line_unwrapped) = line {
                                if line_unwrapped.contains(&substring_to_search_for)
                                {
                                    Some((line_unwrapped, true, current_line))
                                }
                                else
                                {
                                    Some((line_unwrapped, false, current_line))
                                }
                            }
                            else {
                                None
                            }
                        }).collect::<Vec<(String, bool, u16)>>()
                        // (line, matches, line_number)
                    };

                    let matching_lines: Vec<(&String, &u16)> = lines.iter().filter_map(|(line, matching, line_num)| {
                        if *matching
                        {
                            Some((line, line_num))
                        }
                        else
                        {
                            None
                        }
                    }).collect::<Vec<(&String, &u16)>>();

                    if matching_lines.len() == 0
                    {
                        println!("Substring did not match anything");
                        return;
                    }

                    for (line, num) in matching_lines.iter() {
                        println!("{}. {}", num, line);
                    }
                    
                    let del = if matching_lines.len() > 1 {
                        print!("\nWhich substring would you like to delete? (index or * for all) ");

                        // 0 for all or index of line
                        loop {
                            let mut input = String::new();
                            let _ = stdout().flush();
                            stdin().read_line(&mut input).expect("Um what");

                            if input.len() == 0
                            {
                                println!("Please enter something");
                                continue;
                            }

                            if input.contains(" ")
                            {
                                println!("Please only enter an index or a *");
                                continue;
                            }
                            else
                            {
                                let mut chars_iter = input.chars();
                                let first_char = chars_iter.next().unwrap();
                                if first_char == '*'
                                {
                                    break 0;
                                }
                                else
                                {
                                    let mut first_word = String::with_capacity(5);
                                    first_word.push(first_char);
                                    for c in chars_iter
                                    {
                                        if c.is_whitespace()
                                        {
                                            break;
                                        }
                                        else
                                        {
                                            first_word.push(c);
                                        }
                                    }

                                    let ind = first_word.parse::<u16>();

                                    if let Err(e) = ind {
                                        println!("{}", e);
                                        return;
                                    }

                                    // 0 is reserved for *
                                    let ind = ind.unwrap() + 1;

                                    break ind;
                                }
                            }
                        }
                    }
                    else
                    {
                        0
                    };

                    match OpenOptions::new()
                        .write(true)
                        .open(&fileToOpen)
                        {
                            Ok(mut file) => {
                                match file.set_len(0) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        eprintln!("Error setting file length to 0: {}", e);
                                    }
                                };
                                match file.seek(SeekFrom::Start(0)) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        eprintln!("Error seeking to start of file: {}", e);
                                    }
                                };

                                if del != 0
                                {
                                    // println!("Del = {}", del);

                                    // 1 - based
                                    let mut current_index = 2;
                                    // do not write the 1 matched line
                                    for line in lines
                                    {
                                        if current_index != del
                                        {
                                            match writeln!(file, "{}", line.0)
                                            {
                                                Ok(_) => (),
                                                Err(e) => println!("Error while trying to write line! {}", e)
                                            }
                                        }

                                        current_index += 1;
                                    }

                                    // println!("About to write debugging material!");
                                    // writeln!(file, "Debugging material");
                                }
                                else
                                {
                                    // write none of the matching lines
                                    for line in lines
                                    {
                                        if let (line_string, false, _) = line {
                                            writeln!(file, "{}", line_string);
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("{}", e);
                            }
                        }
                    
                    
                    
                    
                }
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