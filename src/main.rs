use chrono::Local;
use colored::Colorize;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn nyra_exists() -> bool {
    let nyra_path = Path::new(".nyra");

    if nyra_path.exists() {
        return true;
    }

    false
}

fn init() {
    if nyra_exists() {
        println!("{} {}", "nyra".purple(), "already exists".red());
        return;
    }

    let result = fs::create_dir(".nyra");

    match result {
        Ok(_) => {}
        Err(_) => println!("Error {:?}", result.err()),
    }

    let _result = fs::create_dir(".nyra/objects");

    let now = Local::now();

    let layout = format!("{}\n{}\n\n{}", "[DATE]", now, "[STAGED]");

    fs::write(".nyra/info.txt", layout).expect("Failed to write file");
}

fn stage(file_name: &str) {
    if !nyra_exists() {
        println!("{}: {}", "nyra".purple(), "No .nyra foulder found in current directory".red());
        return;
    }

    let dir = env::current_dir().unwrap();
    let path = dir.join(file_name);

    if path.exists() && path.is_file() {
        let relative_path = PathBuf::from(file_name);

        let content = fs::read_to_string(".nyra/info.txt").expect("Failed to read file");

        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

        if let Some(_) = lines.iter().find(|l| l == &&relative_path.to_string_lossy().to_string()) {
            println!(
                "{}:{}",
                "nyra".purple(),
                format!(
                    " File is already staged {}",
                    relative_path.to_string_lossy().to_string()
                )
            );
            return;
        }

        if let Some(pos) = lines.iter().position(|l| l == "[STAGED]") {
            lines.insert(pos + 1, relative_path.to_string_lossy().to_string());
        } else {
            println!("{}: {}", "nyra".purple(), "Missing [STAGED] section".red());
            return;
        }

        let new_content = lines.join("\n");

        fs::write(".nyra/info.txt", new_content).expect("Failed to write file");

        println!("{}: Staged: {}", "nyra".purple(), file_name);
    } else {
        println!("{}: No file found: {}", "nyra".purple(), file_name);
    }
}

fn unstage(file_name: &str) {
    if !nyra_exists() {
        println!("{}: {}", "nyra".purple(), "No .nyra foulder found in current directory".red());
        return;
    }


}

fn commit(messege: &String) {}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => println!("{} nyra~", "nyra".purple()),
        2 => match args[1].to_lowercase().as_str() {
            "init" => init(),
            _ => println!("{}: {}", "Unknown command".red(), args[2]),
        },
        3 => match args[1].to_lowercase().as_str() {
            "stage" => stage(&args[2]),
            "unstage" => unstage(&args[2]),
            "commit" => commit(&args[2]),
            _ => println!("{}: {}", "Unknown command".red(), args[2]),
        },
        _ => println!("{}", "Error".red()),
    }
}
