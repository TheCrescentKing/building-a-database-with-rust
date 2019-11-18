use std;

pub fn read_text() -> String {
    let mut input_text = String::new();

    let input = std::io::stdin();
    input.read_line(&mut input_text).unwrap();

    input_text
}

pub fn get_char() -> char {
    let input_character = read_text();

    match input_character.chars().next() {
        Some(n) => return n,
        _ => return '\n',
    };
}

pub fn get_options() {
    loop {
        match read_text().trim().parse() {
            Ok(v) => {
                match call_option_function(v) {
                    Ok(_) => break,
                    Err(_) => continue,
                };
            }
            Err(_) => {
                println!("That not a valid number, please try again.");
                print_options();
            }
        }
    }
}

pub fn print_options() {
    println!("What would you like to do?");
    println!("1. Remove a character");
    println!("2. Turn string to uppercase",);
    println!("3. Change the case of a character");
    println!("4. Split the text by some String");
}

pub fn call_option_function(opt: i32) -> Result<i32, i32> {
    match opt {
        1 => remove_character(),
        2 => all_to_uppercase(),
        3 => char_change_case(),
        4 => split_by_string(),
        _ => {
            println!("Option '{}' is not avaiable. Please try again:", opt);
            return Err(-1);
        }
    }
    Ok(1)
}

pub fn get_choice(char1: char, char2: char) -> char {
    loop {
        let choice = read_text();

        match choice.trim().chars().next() {
            Some(c) => {
                if c == char1 {
                    return char1;
                } else if c == char2 {
                    return char2;
                } else {
                    println!(
                        "Invalid input '{}'. Please try again. ({}/{})",
                        c, char1, char2
                    );
                }
            }

            _ => (),
        };
    }
}

pub fn remove_character() {
    println!("Please type something to eliminate a character: ");
    let input_text = read_text();

    let mut input_char: char;
    loop {
        println!("Please type the character you wish to delete: ");
        input_char = get_char();
        match input_char {
            '\n' => continue,
            _ => break,
        }
    }

    let result = input_text
        .trim()
        .chars()
        .filter(|c| *c != input_char)
        .collect::<String>();

    println!("The resulting string is: '{}'", result);

    save_to_file(result);
}

pub fn all_to_uppercase() {
    println!("Please type what you want to capitalize",);

    let input_text = read_text();
    let input_text = input_text.trim().to_uppercase();
    println!("Result: '{}'", input_text);

    save_to_file(input_text);
}

pub fn char_change_case() {
    println!("Please type the text to modify");
    let input_text = read_text();

    let mut input_char: char;
    loop {
        println!("Please type the character to modify (case sensitive)");
        input_char = get_char();
        match input_char {
            '\n' => continue,
            _ => break,
        }
    }

    println!("Would you like to uppercase or lowercase it? u/l");
    let chosen_case = get_choice('u', 'l');

    let mut result = String::new();

    match chosen_case {
        'u' => {
            result = input_text
                .trim()
                .chars()
                .map(|c| {
                    if c == input_char {
                        return c.to_uppercase().to_string();
                    } else {
                        return c.to_string();
                    }
                })
                .collect::<String>();
        }
        'l' => {
            result = input_text
                .trim()
                .chars()
                .map(|c| {
                    if c == input_char {
                        return c.to_lowercase().to_string();
                    } else {
                        return c.to_string();
                    }
                })
                .collect::<String>();
        }

        _ => (),
    }

    println!("The resulting string is: '{}'", result);

    save_to_file(result);
}

pub fn split_by_string() {
    println!("What text would you like to split?");
    let mut input_text = read_text();
    trim_newline(&mut input_text);

    println!("What whould you like to split by?");
    let mut splitter_string = read_text().chars().collect::<String>();
    trim_newline(&mut splitter_string);

    let result = input_text
        .split(&splitter_string)
        .collect::<Vec<&str>>()
        .join("|");

    println!("The result is {}", result);

    save_to_file(result);
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

pub fn save_to_file(string_to_save: String) {
    println!("Would you like to save the result to a file? type y/n");

    let choice = get_choice('y', 'n');

    match choice {
        'y' => {
            use std::io::Write;
            let mut file = std::fs::File::create("result.txt").unwrap();
            file.write_all(string_to_save.as_bytes()).unwrap();
            println!("Done.");
        }
        'n' => (),
        _ => (),
    }
}
