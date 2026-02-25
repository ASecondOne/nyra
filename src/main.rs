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

fn collect_files_recursive(
    base_dir: &Path,
    current_dir: &Path,
    files: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(base_dir)
            .unwrap_or(path.as_path())
            .to_path_buf();

        if relative_path
            .components()
            .any(|component| component.as_os_str() == ".nyra")
        {
            continue;
        }

        if path.is_dir() {
            collect_files_recursive(base_dir, &path, files)?;
        } else if path.is_file() {
            files.push(relative_path);
        }
    }

    Ok(())
}

fn stage(file_name: &str) {
    if !nyra_exists() {
        println!(
            "{}: {}",
            "nyra".purple(),
            "No .nyra foulder found in current directory".red()
        );
        return;
    }

    let dir = env::current_dir().unwrap();
    let path = dir.join(file_name);

    if file_name == "any" {
        let content = fs::read_to_string(".nyra/info.txt").expect("Failed to read file");

        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

        if lines.iter().position(|l| l == "[STAGED]").is_none() {
            println!("{}: {}", "nyra".purple(), "Missing [STAGED] section".red());
            return;
        }

        let mut files = Vec::new();
        if let Err(err) = collect_files_recursive(&dir, &dir, &mut files) {
            println!(
                "{}: Failed to read directory tree: {}",
                "nyra".purple(),
                err
            );
            return;
        }

        files.sort();

        for relative_path in files {
            let relative_path_string = relative_path.to_string_lossy().to_string();

            if lines.iter().any(|line| line == &relative_path_string) {
                continue;
            }

            lines.push(relative_path_string.clone());
            println!("{}: Staged: {}", "nyra".purple(), relative_path_string);
        }

        let new_content = lines.join("\n");
        fs::write(".nyra/info.txt", new_content).expect("Failed to write file");

        return;
    }

    if path.exists() && path.is_file() {
        let relative_path = PathBuf::from(file_name);

        let content = fs::read_to_string(".nyra/info.txt").expect("Failed to read file");

        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

        if let Some(_) = lines
            .iter()
            .find(|l| l == &&relative_path.to_string_lossy().to_string())
        {
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
        println!(
            "{}: {}",
            "nyra".purple(),
            "No .nyra foulder found in current directory".red()
        );
        return;
    }

    let content = fs::read_to_string(".nyra/info.txt").expect("Failed to read file");

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    if let Some(l) = lines.iter_mut().position(|l| l.to_string() == file_name) {
        lines.remove(l);
    } else {
        println!(
            "{}: {}",
            "nyra".purple(),
            "File is not currently staged".red()
        );
        return;
    }

    let new_content = lines.join("\n");

    fs::write(".nyra/info.txt", new_content).expect("Failed");

    println!("{}: Unstaged: {}", "nyra".purple(), file_name);
}

fn commit(messege: &String) {
    if !nyra_exists() {
        println!(
            "{}: {}",
            "nyra".purple(),
            "No .nyra foulder found in current directory".red()
        );
        return;
    }

    let content = fs::read_to_string(".nyra/info.txt").expect("Failed to read file");

    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    let mut staged_section = false;

    let mut staged_files: Vec<String> = Vec::new();

    for line in lines {
        if line == "[STAGED]" {
            staged_section = true;
        }
        if staged_section && line != "[STAGED]" {
            if line.starts_with("[") {
                break;
            }
            if !line.is_empty() {
                staged_files.push(line);
            }
        }
    }

    if staged_files.len() == 0 {
        println!("{}: {}", "nyra".purple(), "No currently staged files");
        return;
    }

    let now = Local::now();

    let new_object_string = &format!(".nyra/objects/{}-OBJECT", now);
    let new_object_path = Path::new(new_object_string);

    fs::create_dir(new_object_path).unwrap();

    for file in &staged_files {
        let from = Path::new(file);

        let mut to = PathBuf::from(new_object_path);
        to.push(from);

        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::copy(from, to).unwrap();
    }

    let mut path_buff = PathBuf::new();
    path_buff.push(new_object_path);
    path_buff.push(".info.txt");

    let contents = format!("{}\n{}", Local::now(), messege);

    fs::write(path_buff, contents).unwrap();
}

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
