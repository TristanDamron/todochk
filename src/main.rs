/// todochk - simple command line utility that checks for TODOs
/// in your source code.
///
/// USAGE: todochk

use std::fs::File;
use std::io::prelude::*;
use std::env::current_dir;
use walkdir::WalkDir;
use std::fs::metadata;
use regex::Regex;
use colored::*;
use spinners::{Spinner, Spinners};
use std::time::SystemTime;


const VERSION: &str = env!("CARGO_PKG_VERSION");
const TODO_REGEX: &str = r"(//|#|--|%)\s*(TODO:|TODO\s*-|TODO\s)|(\*|=begin|\{-|<!--)(.|\n)*(TODO:|TODO\s*-\s|TODO\s)(.|\n)*(\*|=end|-\}|-->)";
const SMALL_TODO_REGEX: &str = r"(//|#|--|%)\s*(todo:|todo\s*-|todo\s)|(\*|=begin|\{-|<!--)(.|\n)*(todo:|todo\s*-\s|todo\s)(.|\n)*(\*|=end|-\}|-->)";

struct Todo {
    file_id: i32,
    file_name: String,
    line_number: i32,
    line_contents: String,
    line_above_number: i32,
    line_above: String,
}

fn main() {
    let start_time: SystemTime = SystemTime::now();
    let pwd: String = current_dir().unwrap().into_os_string().into_string().unwrap();

    println!("Running todochk v{}", VERSION);
    let mut loading_spinner: Spinner = Spinner::new(Spinners::Aesthetic, format!("Collecting todos from directory {}", pwd).into());

    let collected_todos: Vec<Todo> = collect_todos_from_files_recursively(&pwd);
    loading_spinner.stop();
    println!("");
    show_todos(collected_todos);
    println!(" in {} seconds", (start_time.elapsed().expect("").as_secs()))
}


fn show_todos(mut todos: Vec<Todo>) {
    let todo_regex: Regex = Regex::new(TODO_REGEX).unwrap();
    let small_todo_regex: Regex = Regex::new(SMALL_TODO_REGEX).unwrap();

    let fmt_found_message = format!("Found {} TODOs", todos.len()).underline().red().bold();
    todos.sort_by_key(|x| x.file_id);

    let mut current_file: String = String::new();
    for todo in todos {
        if current_file != todo.file_name {
            println!("");
            println!(">>> In {}...", todo.file_name);
            current_file = todo.file_name;
        }

        let mut fmt_line_contents: String = format!("Line {}: {}",
                                        todo.line_number.to_string().green(),
                                        todo_regex.replace_all(&todo.line_contents, &"TODO".underline().red().bold().to_string()));
        fmt_line_contents = small_todo_regex.replace_all(&fmt_line_contents, &"todo".underline().red().bold().to_string()).to_string();
        println!("{}", "_".repeat(fmt_line_contents.len()));
        if todo.line_above_number != -1 {
            println!("Line {}: {}", todo.line_above_number, todo.line_above);
        }
        println!("{}", fmt_line_contents);
    }

    print!("{}", fmt_found_message);
}


fn is_file(path: &String) -> bool {
    let path_md = metadata(&path).unwrap();
    return path_md.is_file();
}


fn collect_todos_from_files_recursively(dir: &String) -> Vec<Todo> {
    let todo_regex: Regex = Regex::new(TODO_REGEX).unwrap();
    let small_todo_regex: Regex = Regex::new(SMALL_TODO_REGEX).unwrap();
    let mut ret: Vec<Todo> = Vec::<Todo>::new();
    let walk_directory: WalkDir = WalkDir::new(&dir);

    for item in walk_directory {
        let item_path: String = item.unwrap().path().display().to_string();
        let mut file_id: i32 = 1;
        if is_file(&item_path) {
            let mut file = match File::open(&item_path) {
                Err(_) => continue,
                Ok(file) => file,
            };

            let mut file_contents: String = String::new();
            match file.read_to_string(&mut file_contents) {
                Err(_) => continue,
                Ok(_) => {
                    let lines: Vec<&str> = file_contents.split("\n").collect();
                    let mut prev_line: String = String::new();
                    let mut line_counter: i32 = 1;

                    for line in lines {
                        if todo_regex.is_match(line) || small_todo_regex.is_match(line) {
                            let mut todo: Todo = Todo {
                                file_id: file_id,
                                file_name: item_path.to_string(),  // Essentially copies the value of item_path to a new string
                                line_number: line_counter,
                                line_contents: line.to_string(),
                                line_above_number: -1,
                                line_above: String::new(),
                            };

                            if line_counter > 1 {
                                todo.line_above_number = line_counter - 1;
                                todo.line_above = prev_line;
                            }

                            ret.push(todo)
                        }
                        line_counter += 1;
                        prev_line = line.to_string();
                    }
                },
            }

            file_id += 1;
        }
    }

    return ret;
}
