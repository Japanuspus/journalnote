use chrono;
use std::path;
use std::fs;
use std::io::{BufRead, Write};
use std::env;

fn format_date<D: chrono::Datelike>(d: &D) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day())
}

fn days_till_friday<D: chrono::Datelike>(d: &D) -> u32 {
    let day_number = d.weekday().number_from_monday(); // monday is 1
    ((5+7)-day_number) as u32 % 7  //next friday is 5+7
}

fn note_file_name<D: chrono::Datelike>(friday: &D) -> String {
    format!("{}-journal.txt", format_date(friday))
}

fn main() {
    let today = chrono::offset::Local::now().date();
    let friday = today + chrono::Duration::days(days_till_friday(&today) as i64);

    let note_folder_str = env::var("JOURNAL_NOTE_FOLDER").expect("Environment variable JOURNAL_NOTE_FOLDER must be defined");
    let note_folder = path::Path::new(&note_folder_str);
    if !note_folder.is_absolute() {
        panic!("Note folder path must be absolute")
    }

    fs::create_dir_all(note_folder).expect("Unable to create note folder");

    let note_file = note_folder.join(path::Path::new(&note_file_name(&friday)));

    // Buffer for all additions to file, to make sure we get an atomic write.
    let mut buffer = String::new();
    let mut has_today: bool = false;

    let mut file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&note_file) {
            Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let existing_file = fs::OpenOptions::new()
                    .read(true)
                    .append(true)
                    .open(&note_file).expect("Failed to open existing file");
                let day_header = format!("## {}", format_date(&today));
                for line in std::io::BufReader::new(&existing_file).lines() {
                    if line.expect("Invalid UTF8").starts_with(&day_header) {
                        has_today = true;
                        break;
                    }
                }    
                existing_file
            },
            Ok(f) => {
                buffer.push_str(&format!("# Journal for week ending at {}\n\n", 
                    format_date(&friday)));
                f
            },
            _ => panic!("Failed to create new file")
        };
    
    if ! has_today {
        buffer.push_str(&format!("## {}\n\n", format_date(&today)));
    }

    let message = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    if !message.is_empty() {
        buffer.push_str(&format!("{}\n", message));
    }
    
    // This write should be atomic...
    file.write_all(buffer.as_bytes()).expect("Unable to write to file");
}

