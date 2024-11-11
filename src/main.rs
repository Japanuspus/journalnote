use chrono;
use clap::Parser;
// use std::fmt::Write;
// use std::io::Write;
use std::env;
use std::fs;
use std::io::{BufRead, Read, Seek, SeekFrom, Write};
use std::path;

fn format_date<D: chrono::Datelike>(d: &D) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day())
}

fn days_till_friday<D: chrono::Datelike>(d: &D) -> u32 {
    let day_number = d.weekday().number_from_monday(); // monday is 1
    ((5 + 7) - day_number) as u32 % 7 //next friday is 5+7
}

fn note_file_name<D: chrono::Datelike>(friday: &D) -> String {
    format!("{} journal.md", format_date(friday))
}

/// Journal note - add entry to todays journal file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Header to use for content
    #[arg(long)]
    header: Option<String>,
    /// Content lines
    content: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let now = chrono::offset::Local::now();
    let today = now.date();
    let friday = today + chrono::Duration::days(days_till_friday(&today) as i64);

    let note_folder_str = env::var("JOURNAL_NOTE_FOLDER")
        .expect("Environment variable JOURNAL_NOTE_FOLDER must be defined");
    let note_folder = path::Path::new(&note_folder_str);
    if !note_folder.is_absolute() {
        panic!("Note folder path must be absolute")
    }

    fs::create_dir_all(note_folder).expect("Unable to create note folder");

    let note_file = note_folder.join(path::Path::new(&note_file_name(&friday)));

    // Buffer for all additions to file, to make sure we get an atomic write.
    let mut buffer = String::new();
    let mut has_today: bool = false;
    let mut clean_line: bool = false; //
    let day_header = format!("## {}", format_date(&today));

    let mut file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&note_file)
    {
        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let mut existing_file = fs::OpenOptions::new()
                .read(true)
                .append(true)
                .open(&note_file)
                .expect("Failed to open existing file");
            for line in std::io::BufReader::new(&existing_file).lines() {
                if line.expect("Invalid UTF8").starts_with(&day_header) {
                    has_today = true;
                    break;
                }
            }
            {
                let mut buffer = [0; 1];
                existing_file
                    .seek(SeekFrom::End(-1))
                    .expect("Unable to seek to end of file");
                match existing_file.read(&mut buffer) {
                    Ok(1) => {
                        if buffer[0] == b'\n' {
                            clean_line = true;
                        }
                    }
                    _ => panic!("Could not read end of file"),
                }
            }
            existing_file
        }
        Ok(f) => {
            buffer.push_str(&format!(
                "# Journal for week ending at {}\n\n",
                format_date(&friday)
            ));
            clean_line = true;
            f
        }
        _ => panic!("Failed to create new file"),
    };

    if !has_today {
        buffer.push_str(&format!("\n{} - {}\n\n", &day_header, &today.format("%a")));
        clean_line = true;
    }

    if let Some(header) = args.header {
        buffer.push_str(&format!("\n### {} - {}\n\n", &now.format("%H:%M"), header));
        clean_line = true;
    }
    let message = args.content.join("\n");
    if !message.is_empty() {
        if !clean_line {
            buffer.push_str("\n");
        }
        buffer.push_str(&format!("{}\n", message));
    }
    // This write should be atomic...
    file.write_all(buffer.as_bytes())
        .expect("Unable to write to file");
}
