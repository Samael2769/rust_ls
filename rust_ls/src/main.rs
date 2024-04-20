use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::env;
use chrono::{DateTime, Local};
use std::os::unix::fs::PermissionsExt;
use users::{get_user_by_uid, get_group_by_gid};
use std::path::{Path};
use std::time::SystemTime;

#[derive(Debug)]
#[derive(Clone)]
struct File {
    name: String,
    size: u64,
    is_dir: bool,
    user: String,
    group: String,
    permissions: String,
    date: String,
    nlink: u64,
    nblocks: u64,
}

fn numeric_to_symbolic(mode: u32) -> String {
    let mut permissions = String::with_capacity(10); // Allocate space for the 10 characters in the symbolic representation

    // Define the symbolic representation for each permission
    let symbols = ['r', 'w', 'x'];

    // Owner permissions
    for i in 0..3 {
        let mask = 1 << (8 - i);
        permissions.push(if mode & mask != 0 { symbols[i] } else { '-' });
    }

    // Group permissions
    for i in 0..3 {
        let mask = 1 << (5 - i);
        permissions.push(if mode & mask != 0 { symbols[i] } else { '-' });
    }

    // Other permissions
    for i in 0..3 {
        let mask = 1 << (2 - i);
        permissions.push(if mode & mask != 0 { symbols[i] } else { '-' });
    }

    permissions
}

fn get_files(filename: String, is_a: bool, is_r: bool) -> Result<Vec<File>, io::Error> {
    let mut files = Vec::new();
    let mut entries = fs::read_dir(&filename)?;

    if is_a && !is_r{
        // If is_a is true, manually add "." and ".." entries
        let mut file = get_file_info(Path::new("./"))?;
        file.name = ".".to_string();
        files.push(file);
        file = get_file_info(Path::new("../"))?;
        file.name = "..".to_string();
        files.push(file);
    }

    while let Some(entry) = entries.next() {
        let entry = entry?;
        if let Some(file_name) = entry.file_name().to_str() {
            if file_name != "./" && file_name != "../" {
                let file_info = get_file_info(&entry.path())?;
                files.push(file_info);
            }
        }
    }

    Ok(files)
}

fn get_file_info(path: &Path) -> Result<File, io::Error> {
    let metadata = fs::metadata(path)?;
    let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let user = get_user_by_uid(metadata.uid()).unwrap().name().to_string_lossy().to_string();
    let group = get_group_by_gid(metadata.gid()).unwrap().name().to_string_lossy().to_string();
    let permissions = numeric_to_symbolic(metadata.permissions().mode());
    let modified_time = metadata.modified().unwrap();
    let date = format_system_time(modified_time);
    let nlink = metadata.nlink();
    let nblocks = metadata.blocks();
    let file = File {
        name: file_name,
        size: metadata.len(),
        is_dir: metadata.is_dir(),
        user,
        group,
        permissions,
        date,
        nlink,
        nblocks,
    };
    Ok(file)
}

fn format_system_time(time: SystemTime) -> String {
    let dt = DateTime::<Local>::from(time);
    // Get components
    let naive_utc = dt.naive_utc();
    let offset = dt.offset().clone();
    // Serialize, pass through FFI... and recreate the `DateTime`:
    let dt_new = DateTime::<Local>::from_naive_utc_and_offset(naive_utc, offset);
    dt_new.format("%b %e %H:%M").to_string()
}

fn print_files(files: Vec<File>, is_a: bool, is_l: bool) {
    let nblocks = files.iter().map(|f| f.nblocks).sum::<u64>();
    if is_l {
        println!("total {}", nblocks);
    }
    for file in files {
        if !is_a && file.name.starts_with(".") {
            continue;
        }
        if is_l {
            if file.is_dir {
                print!("d");
            } else {
                print!("-");
            }
            
            print!("{}", file.permissions);
            print!(" ");
            print!("{}", file.nlink);
            print!(" ");
            print!("{}", file.user);
            print!(" ");
            print!("{}", file.group);
            print!(" ");
            print!("{}", file.size);
            print!(" ");
            print!("{}", file.date);
            print!(" ");
            print!("{}", file.name);
            println!();
        } else {
            print!("{}", file.name);
            print!(" ");
        }
    }
    println!();
}

fn print_recursive_files(files: Vec<File>, is_a: bool, is_l: bool, filepath: String) {
    println!("{}:", filepath);
    print_files(files.clone(), is_a, is_l);
    println!();
    for file in files {
        if !is_a && file.name.starts_with(".") {
            continue;
        }
        if file.is_dir {
            let filepath = format!("{}/{}", filepath, file.name);
            let new_files = get_files(filepath.clone(), is_a, true).unwrap();
            print_recursive_files(new_files, is_a, is_l, filepath);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut filepath = ".".to_string();
    if args.len() >= 2  && !args[1].starts_with("-") {
        filepath = args[1].clone().to_string();
        if let Err(file) = fs::File::open(&filepath) {
            eprintln!("Error: {}", file);
            std::process::exit(1);
        }
    }
    let mut is_a = false;
    let mut is_l = false;
    let mut is_r = false;
    for arg in args {
        if arg == "-a" {
            is_a = true;
        }
        if arg == "-l" {
            is_l = true;
        }
        if arg == "-R" {
            is_r = true;
        }
    }
    if is_r {
        let files = get_files(filepath.clone(), is_a, true).unwrap();
        print_recursive_files(files, is_a, is_l, filepath);
        return;
    }
    let files = get_files(filepath, is_a, false).unwrap();
    print_files(files, is_a, is_l);
}
