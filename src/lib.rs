use chrono::{self, Datelike, Duration};
use std::path;
use std::fs;
use std::io::{BufRead, Write};
use std::env;

pub type DateTime = chrono::DateTime<chrono::Local>;

pub struct Config {
    note_folder: path::PathBuf,
}

pub fn make_config() -> Config {
    let note_folder_str = env::var("JOURNAL_NOTE_FOLDER").expect("Environment variable JOURNAL_NOTE_FOLDER must be defined");
    let note_folder = path::Path::new(&note_folder_str).to_owned();
    if !note_folder.is_absolute() {
        panic!("Note folder path must be absolute")
    }
    Config{note_folder}
}

struct NoteFile {
    friday: DateTime,
    note_path: path::PathBuf,
}

fn this_friday(d: &DateTime) -> DateTime {
    let day_number = d.weekday().number_from_monday(); // monday is 1
    let days_till_friday = ((5+7)-day_number) as u32 % 7;  //next friday is 5+7
    *d + Duration::days(days_till_friday as i64)
}

fn format_date<D: Datelike>(d: &D) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day())
}

fn note_file_name<D: Datelike>(friday: &D) -> String {
    format!("{}-journal.txt", format_date(friday))
}

impl Config {
    fn get_note_file(&self, d: &DateTime) -> NoteFile {
        let friday = this_friday(d);    
        let note_path = self.note_folder.join(path::Path::new(&note_file_name(&friday)));        
        NoteFile{friday, note_path}
    }
}

pub fn enter_message_at_time(config: &Config, date: &DateTime, message: &str, )  {
    // Buffer for all additions to file, to make sure we get an atomic write.
    let mut buffer = String::new();
    let mut has_today: bool = false;
    let day_header = format!("## {}", format_date(&date.date()));
    let note_file = config.get_note_file(date);

    fs::create_dir_all(note_file.note_path.parent().unwrap()).expect("Unable to create note folder");

    let mut file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&note_file.note_path) {
            Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let existing_file = fs::OpenOptions::new()
                    .read(true)
                    .append(true)
                    .open(note_file.note_path).expect("Failed to open existing file");
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
                    format_date(&note_file.friday)));
                f
            },
            _ => panic!("Failed to create new file")
        };
    
    if ! has_today {
        buffer.push_str(&format!("\n{} - {}\n\n", &day_header, &date.format("%a")));
    }

    if !message.is_empty() {
        buffer.push_str(&format!("{} - {}\n", &date.format("%H:%M"), message));
    }
    
    // This write should be atomic...
    file.write_all(buffer.as_bytes()).expect("Unable to write to file");
}

pub fn now() -> DateTime {
    chrono::offset::Local::now()
}

pub fn enter_message(message: &str) {
    let config = make_config();
    enter_message_at_time(&config, &now(), message);
}