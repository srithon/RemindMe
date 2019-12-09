extern crate clap;
use directories::{ProjectDirs};

use std::path::{Path, PathBuf};
use std::fs::{self, OpenOptions, File};
use std::io::{stdin, BufRead, BufReader, Write, Seek, SeekFrom};

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

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

    let bufwtr: BufferWriter = BufferWriter::stdout(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();

    let project_directory = project_directory.unwrap();

    let data_dir = project_directory.data_dir();

    if !Path::exists(&data_dir)
    {
        match fs::create_dir_all(&data_dir)
        {
            Ok(_) => (),
            Err(e) => {
                panic!("Error while trying to create base directory\n{}", e);
            }
        }
    }

    let default_group = "general";

    let default_group_file_name = data_dir.join(&default_group);

    let mut err_colorspec = ColorSpec::new();
    err_colorspec.set_fg(Some(Color::Rgb(255,0,0)));

    match matches.subcommand()
    {
        ("finish", sub_matches_maybe) => {
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

            buffer.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(240, 95, 95))));
            
            match input_parsed
            {
                InputValue::Indices(mut indices) => {
                    // step 1: sort indices
                    // todo safety

                    let file_read = File::open(&file_to_open);

                    if file_read.is_err() {
                        panic!("Cannot open file!");
                    }

                    indices.sort();

                    let reader = BufReader::new(file_read.unwrap());
                    let mut all_tasks: Vec<_> = reader.lines().filter_map(|task| task.ok()).collect();

                    let mut deleted: usize = 0;

                    for mut index in indices {
                        if index < 1 {
                            writeln!(&mut buffer, "Invalid index: {}", index);
                            continue;
                        }

                        index -= 1;

                        if index > all_tasks.len() {
                            return;
                        }

                        all_tasks.remove(index - deleted);

                        deleted += 1;
                    }

                    bufwtr.print(&buffer);

                    // file_read.

                    let mut file = OpenOptions::new()
                        .write(true)
                        .open(&file_to_open)
                        .unwrap();
                    
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
                    
                    for line in all_tasks
                    {
                        match writeln!(&mut file, "{}", line)
                        {
                            Ok(_) => (),
                            Err(e) => {
                                panic!("Error writing to file!\n{}", e);
                            }
                        }
                    }
                },
                InputValue::SubString(substring_to_search_for) => {
                    // look through the lines and find all that match
                    // print them out on screen
                    // let user specify indices or * to delete all
                    // if user has "no-prompt" option specified, default to *

                    if let Ok(file) = File::open(&file_to_open) {
                        let lines: Vec<(String, bool, usize)> = {
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
                            }).collect::<Vec<(String, bool, usize)>>()
                            // (line, matches, line_number)
                        };

                        let mut current_index: usize = 0;

                        let matching_lines: Vec<(&String, &usize, usize)> = lines.iter().filter_map(|(line, matching, line_num)| {
                            if *matching
                            {
                                current_index += 1;
                                Some((line, line_num, current_index))
                            }
                            else
                            {
                                None
                            }
                        }).collect::<Vec<(&String, &usize, usize)>>();

                        if matching_lines.len() == 0
                        {
                            writeln!(&mut buffer, "Substring did not match anything");
                            bufwtr.print(&buffer);
                            buffer.clear();
                            return;
                        }

                        let mut question_colorspec = ColorSpec::new();
                        question_colorspec.set_fg(Some(Color::Rgb(242,229,144)));

                        buffer.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(107, 222, 168))));

                        for (line, _, index) in matching_lines.iter() {
                            writeln!(&mut buffer, "{}. {}", index, line);
                        }
                        
                        bufwtr.print(&buffer);
                        buffer.clear();

                        let del: usize = if matching_lines.len() > 1 {
                            // 0 for all or index of line
                            loop {
                                buffer.clear();

                                buffer.set_color(&question_colorspec);
                                write!(&mut buffer, "\nWhich substring would you like to delete? (index or * for all) ");
                                let mut input = String::new();
                                bufwtr.print(&buffer);
                                buffer.clear();
                                stdin().read_line(&mut input).expect("Um what");

                                buffer.set_color(&err_colorspec);

                                if input.len() == 0
                                {
                                    writeln!(&mut buffer, "Please enter something");
                                    bufwtr.print(&buffer);
                                    continue;
                                }

                                if input.contains(" ")
                                {
                                    writeln!(&mut buffer, "Please only enter an index or a *");
                                    bufwtr.print(&buffer);
                                    continue;
                                }
                                else
                                {
                                    let mut chars_iter = input.chars();
                                    let first_char = {
                                        let c = chars_iter.next();
                                        match c
                                        {
                                            Some(n) => n,
                                            None => {
                                                eprintln!("Invalid input");
                                                continue;
                                            }
                                        }
                                    };
                                    if first_char == '*'
                                    {
                                        break 0;
                                    }
                                    else if first_char == '0'
                                    {
                                        writeln!(&mut buffer, "Index is not 0-based!");
                                        bufwtr.print(&buffer);
                                        buffer.clear();
                                        continue;
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

                                        let ind = first_word.parse::<usize>();

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
                                        let del: usize = {
                                            // println!("Current value of del: {}", del);
                                            // println!("Length of [matching_lines]: {}", matching_lines.len());

                                            buffer.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(0,0,0))));

                                            let x = matching_lines.get(del - 2).unwrap();
                                            writeln!(&mut buffer, "Deleted: {}", x.0);
                                            bufwtr.print(&buffer);

                                            *x.1
                                        };

                                        // println!("Del = {}", del);

                                        // 1 - based
                                        let mut current_index = 1;
                                        // do not write the 1 matched line
                                        for line in lines.iter()
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
                                                match writeln!(&mut file, "{}", line_string)
                                                {
                                                    Ok(_) => (),
                                                    Err(e) => {
                                                        panic!("Error writing to file!\n{}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                Err(e) => {
                                    panic!("Could not open file for writing!\n{}", e);
                                }
                            }
                    }
                }
            };
        },
        ("add", sub_matches_maybe) => {
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

            match file.write_fmt(format_args!("{}\n", string))
            {
                Ok(_) => (),
                Err(e) => {
                    panic!("Error fmt writing to file!\n{}", e);
                }
            }
                
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
            buffer.set_color(ColorSpec::new().set_fg(Some(Color::Ansi256(209))));

            let file_names: Vec<std::path::PathBuf> = if let Some(sub_matches) = sub_matches_maybe
                {
                    if let Some(group_name) = sub_matches.value_of("group-name")
                    {
                        if group_name.eq("*")
                        {
                            let files = fs::read_dir(data_dir);
                            if let Ok(file_list) = files
                            {
                                file_list.filter_map(|x| {
                                    if let Some(x) = x.ok()
                                    {
                                        Some(x.path())
                                    }
                                    else
                                    {
                                        None
                                    }
                                }).collect::<Vec<PathBuf>>()
                            }
                            else
                            {
                                println!("Error");
                                return;
                            }
                        }
                        else
                        {
                            vec![data_dir.join(group_name)]
                        }
                    }
                    else
                    {
                        vec![default_group_file_name]
                    }
                }
                else
                {
                    vec![default_group_file_name]
                };

            for file_name in file_names
            {
                if let Ok(file_to_list) = File::open(&file_name)
                {
                    if let Some(file_name_string) = file_name.file_name()
                    {
                        if let Some(real_file_name_string) = file_name_string.to_str()
                        {
                            buffer.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(158, 255, 168))));
                            writeln!(&mut buffer, "Group \"{}\"", real_file_name_string);
                            bufwtr.print(&buffer);
                            buffer.clear();
                            buffer.set_color(ColorSpec::new().set_fg(Some(Color::Ansi256(209))));
                        }
                    }
                    let reader = BufReader::new(file_to_list);

                    for (index, line) in reader.lines().enumerate() {
                        let line = line.unwrap();
                        writeln!(&mut buffer, "{}. {}", index + 1, line);
                    }
                }
                else
                {
                    buffer.set_color(&err_colorspec);
                    writeln!(&mut buffer, "Group not yet created");
                }

                bufwtr.print(&buffer);
                buffer.clear();

                println!();
            }
        }
    }
}