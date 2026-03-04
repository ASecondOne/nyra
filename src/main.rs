use chrono::{DateTime, FixedOffset, Local};
use colored::Colorize;
use std::{
    collections::{BTreeMap, BTreeSet},
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

fn latest_object_data_dir() -> std::io::Result<Option<PathBuf>> {
    let mut latest_object: Option<(DateTime<FixedOffset>, PathBuf)> = None;

    for entry in fs::read_dir(".nyra/objects")? {
        let entry = entry?;
        let object_path = entry.path();

        if !object_path.is_dir() {
            continue;
        }

        let Some(folder_name) = object_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        let Some(date_text) = folder_name.strip_suffix("-OBJECT") else {
            continue;
        };

        let Ok(parsed_date) = DateTime::parse_from_str(date_text, "%Y-%m-%d %H:%M:%S%.f %:z")
        else {
            continue;
        };

        match &latest_object {
            Some((latest_date, _)) if parsed_date <= *latest_date => {}
            _ => latest_object = Some((parsed_date, object_path)),
        }
    }

    Ok(latest_object.map(|(_, mut object_path)| {
        object_path.push("Data");
        object_path
    }))
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

    let mut data_path: PathBuf = PathBuf::new();
    data_path.push(new_object_path);
    data_path.push("Data");

    fs::create_dir(new_object_path).unwrap();
    fs::create_dir(&data_path).unwrap();

    for file in &staged_files {
        let from = Path::new(file);

        let mut to = PathBuf::from(&data_path);
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

    let refreshed_info = format!("[DATE]\n{}\n\n[STAGED]", Local::now());
    fs::write(".nyra/info.txt", refreshed_info).unwrap();
}

fn status() {
    if !nyra_exists() {
        println!(
            "{}: {}",
            "nyra".purple(),
            "No .nyra foulder found in current directory".red()
        );
        return;
    }

    let dir = env::current_dir().unwrap();

    let mut staged_files: Vec<String> = Vec::new();
    let content = match fs::read_to_string(".nyra/info.txt") {
        Ok(content) => content,
        Err(err) => {
            println!(
                "{}: Failed to read status information: {}",
                "nyra".purple(),
                err
            );
            return;
        }
    };

    let mut found_staged_section = false;
    for line in content.lines() {
        if line == "[STAGED]" {
            found_staged_section = true;
            continue;
        }
        if found_staged_section {
            if line.starts_with("[") {
                break;
            }
            if !line.is_empty() {
                staged_files.push(line.to_string());
            }
        }
    }
    staged_files.sort();
    staged_files.dedup();

    let mut working_relative_files = Vec::new();
    if let Err(err) = collect_files_recursive(&dir, &dir, &mut working_relative_files) {
        println!(
            "{}: Failed to read directory tree: {}",
            "nyra".purple(),
            err
        );
        return;
    }
    working_relative_files.sort();

    let mut working_map: BTreeMap<String, PathBuf> = BTreeMap::new();
    for relative_path in &working_relative_files {
        let relative_path_string = relative_path.to_string_lossy().to_string();
        working_map.insert(relative_path_string, dir.join(relative_path));
    }

    let staged_set: BTreeSet<String> = staged_files.iter().cloned().collect();

    let latest_data_dir = match latest_object_data_dir() {
        Ok(path) => path,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => {
            println!(
                "{}: Failed to read objects directory: {}",
                "nyra".purple(),
                err
            );
            return;
        }
    };

    let mut snapshot_map: BTreeMap<String, PathBuf> = BTreeMap::new();
    if let Some(data_dir) = latest_data_dir {
        if data_dir.exists() {
            let mut snapshot_relative_files = Vec::new();
            if let Err(err) =
                collect_files_recursive(&data_dir, &data_dir, &mut snapshot_relative_files)
            {
                println!(
                    "{}: Failed to read latest object data: {}",
                    "nyra".purple(),
                    err
                );
                return;
            }

            snapshot_relative_files.sort();
            for relative_path in snapshot_relative_files {
                let relative_path_string = relative_path.to_string_lossy().to_string();
                snapshot_map.insert(relative_path_string, data_dir.join(relative_path));
            }
        }
    }

    let mut changed_files: Vec<(char, String)> = Vec::new();
    for (relative_path, working_file) in &working_map {
        match snapshot_map.get(relative_path) {
            None => changed_files.push(('A', relative_path.clone())),
            Some(snapshot_file) => match (fs::read(working_file), fs::read(snapshot_file)) {
                (Ok(current_bytes), Ok(snapshot_bytes)) => {
                    if current_bytes != snapshot_bytes {
                        changed_files.push(('M', relative_path.clone()));
                    }
                }
                (Err(err), _) => {
                    println!(
                        "{}: Failed to read file {}: {}",
                        "nyra".purple(),
                        relative_path,
                        err
                    );
                }
                (_, Err(err)) => {
                    println!(
                        "{}: Failed to read snapshot file {}: {}",
                        "nyra".purple(),
                        relative_path,
                        err
                    );
                }
            },
        }
    }

    for relative_path in snapshot_map.keys() {
        if !working_map.contains_key(relative_path) {
            changed_files.push(('D', relative_path.clone()));
        }
    }

    changed_files.sort_by(|a, b| a.1.cmp(&b.1));
    let mut unstaged_files: Vec<(char, String)> = changed_files
        .iter()
        .filter(|(_, path)| !staged_set.contains(path))
        .map(|(change_type, path)| (*change_type, path.clone()))
        .collect();
    unstaged_files.sort_by(|a, b| a.1.cmp(&b.1));

    println!("{} {}", "nyra".purple(), "status".bold());
    println!();

    println!("{}", "Staged files:".green().bold());
    if staged_files.is_empty() {
        println!("  {}", "none".dimmed());
    } else {
        for file in staged_files {
            println!("  {} {}", "staged:".green(), file.green());
        }
    }

    println!();
    println!("{}", "Unstaged files:".yellow().bold());
    if unstaged_files.is_empty() {
        println!("  {}", "none".dimmed());
    } else {
        for (change_type, file) in unstaged_files {
            match change_type {
                'A' => println!("  {} {}", "new file:".yellow(), file.yellow()),
                'M' => println!("  {} {}", "modified:".yellow(), file.yellow()),
                'D' => println!("  {} {}", "deleted:".yellow(), file.yellow()),
                _ => {}
            }
        }
    }

    println!();
    println!("{}", "Changed since last commit:".cyan().bold());
    if changed_files.is_empty() {
        println!("  {}", "none".dimmed());
    } else {
        for (change_type, file) in changed_files {
            match change_type {
                'A' => println!("  {} {}", "new file:".green(), file.green()),
                'M' => println!("  {} {}", "modified:".yellow(), file.yellow()),
                'D' => println!("  {} {}", "deleted:".red(), file.red()),
                _ => {}
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => println!("{} nyra~", "nyra".purple()),
        2 => match args[1].to_lowercase().as_str() {
            "init" => init(),
            "status" => status(),
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
