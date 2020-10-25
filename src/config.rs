use std::fs::File;
use std::io;
use std::io::*;

pub fn get_or_create_token_file() -> String {
    let home_directory = dirs::home_dir().expect("could not get home directory");
    let home_directory_str = home_directory
        .to_str()
        .expect("could not set home directory to str");
    let path = format!("{}/todoist_token.cfg", home_directory_str);

    let contents: String = match File::open(&path) {
        Ok(file) => read_file(file),
        Err(_) => create_file(path),
    };

    contents
}

fn read_file(file: File) -> String {
    let mut contents = String::new();
    let mut file = file;
    file.read_to_string(&mut contents)
        .expect("Could not read to string");

    contents
}

#[allow(clippy::unused_io_amount)]
fn create_file(path: String) -> String {
    let mut input = String::new();
    println!("Please enter your Todoist API token from https://todoist.com/prefs/integrations ");
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");

    let mut file = File::create(path).expect("could not create file");
    file.write(input.as_bytes())
        .expect("could not write to file");

    input
}
