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
    let project_directory = ProjectDirs::from("", "Sridaran Thoniyil", "RemindMe");

    if let None = project_directory {
        panic!("Cannot find base directory");
    }

    let project_directory = project_directory.unwrap();

    let data_dir = project_directory.data_dir();

    if !Path::exists(&data_dir)
    {
        fs::create_dir_all(&data_dir);
    }

    let default_group = "general";

    let default_group_file_name = data_dir.join(&default_group);

    match matches.subcommand()
    {
        ("finish", sub_matches_maybe) => {
            println!("Used finish");

            /*
            finish takes in either
                a) index of the task in the file
                b) substring of the task
            if index is provided, add "FINISHED"
            if substring provided, list all tasks that match,
                and then do same index thing
            */

            let sub_matches = sub_matches_maybe.unwrap();

            let file_to_open = if let Some(group_name) = sub_matches.value_of("group")
                {
                    println!("Got the group value!");
                    if group_name != ""
                    {
                        data_dir.join(group_name)
                    }
                    else
                    {
                        default_group_file_name
                    }
                }
                else
                {
                    // there is a default value, why is it none?
                    panic!("Else for group value? what?");
                };

            if !file_to_open.exists()
            {
                println!("Must create the group first! {}", file_to_open.display());
                return;
            }

            let input = sub_matches.values_of("INPUT").unwrap();

            enum InputValue
            {
                SubString(String),
                Indices(Vec<usize>)
            };

            let input_parsed: InputValue = {
                if sub_matches.is_present("substring")
                {
                    InputValue::SubString(
                        input.collect::<Vec<&str>>().join(" ")
                    )
                }
                else if sub_matches.is_present("index")
                {
                    InputValue::Indices(input
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
                        .collect::<Vec<usize>>())
                }
                else
                {
                    // iterate through and create
                    // a list of strings
                    // if any of the strings cannot be
                    // parsed as numbers, treat as substrings

                    let individual_words: Vec<&str> = input.collect::<Vec<&str>>();

                    let mut is_substring = false;

                    let mut indices: Vec<usize> = Vec::with_capacity(individual_words.len());

                    for word in individual_words.iter()
                    {
                        match word.parse::<usize>()
                        {
                            Ok(number) => {
                                indices.push(number);
                            },
                            Err(_) => {
                                is_substring = true;
                                break;
                            }
                        }
                    }

                    if is_substring
                    {
                        InputValue::SubString(
                            individual_words.join(" ")
                        )
                    }
                    else
                    {
                        InputValue::Indices(indices)
                    }
                }
            };
            
            match input_parsed
            {
                InputValue::Indices(mut indices) => {
                    // step 1: sort indices
                    // todo safety

                    let file_read = File::open(&file_to_open);

                    if file_read.is_err() {
                        println!("Cannot open file!");
                        return;
                    }

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
                        .open(&file_to_open)
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
                },
                InputValue::SubString(substring_to_search_for) => {
                    // look through the lines and find all that match
                    // print them out on screen
                    // let user specify indices or * to delete all
                    // if user has "no-prompt" option specified, default to *

                    if let Ok(file) = File::open(&file_to_open) {
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
                            .open(&file_to_open)
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
            };
        },
        ("add", sub_matches_maybe) => {
            println!("Used add");

            let sub_matches = sub_matches_maybe.unwrap();

            let mut string = "".to_string();
            
            let file_to_open = {
                let group_name = sub_matches.value_of("group").unwrap();

                if group_name != ""
                {
                    data_dir.join(group_name)
                }
                else
                {
                    default_group_file_name
                }
            };

            if let Some(list_of_words) = sub_matches.values_of("INPUT")
            {
                list_of_words.for_each(|word| {
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
                .open(file_to_open)
                .unwrap();
            
            // if let Err(e) = writeln!(file, "{}", string)
            // {
            //     eprintln!("{}", e);
            // }

            file.write_fmt(format_args!("{}\n", string));
                
        },
        ("config", sub_matches_maybe) => {
            if let Some(sub_matches) = sub_matches_maybe {
                if let Some(sub_group) = sub_matches.value_of("group") {
                    println!("Group has a value: {}", sub_group);
                }
                else
                {
                    println!("Group does not have a value");
                }
            }
        },
        // list is the default subcommand
        (_, sub_matches_maybe) => {
            println!("Used list");

            let file_name = if let Some(sub_matches) = sub_matches_maybe
                {
                    if let Some(group_name) = sub_matches.value_of("group-name")
                    {
                        data_dir.join(group_name)
                    }
                    else
                    {
                        default_group_file_name
                    }
                }
                else
                {
                    default_group_file_name
                };

            if let Ok(file_to_list) = File::open(&file_name)
            {
                let reader = BufReader::new(file_to_list);

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