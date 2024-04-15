use std::fs;

fn get_files(filename: String) -> Result<Vec<String>, std::io::Error> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(filename) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(file_name) = entry.file_name().to_str() {
                    files.push(file_name.to_string());
                }
            }
        }
    }
    Ok(files)
}

fn basic_print_files(file_names: Vec<String>) {
    for file in file_names {
        if file.starts_with(".") {
            continue;
        }
        println!("{}", file);
    }
}

fn main() {
    let files = get_files(".".to_string()).unwrap();
    basic_print_files(files);
}
