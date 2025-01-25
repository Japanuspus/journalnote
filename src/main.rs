use anyhow::Context;
use chrono;
use clap::Parser;
// use std::fmt::Write;
// use std::io::Write;
use std::env;
use std::fs;
use std::io::{BufRead, Read, Seek, SeekFrom, Write};
use std::path;
use std::path::PathBuf;

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

struct Message<'a> {
    is_continuation: bool,
    content: &'a str,
}

/// Seek to just before terminating newline, or EOF if no such exists
fn file_seek_existing(note_file: &PathBuf, day_header: &str) -> anyhow::Result<(fs::File, bool)> {
    let mut existing_file = fs::OpenOptions::new()
        .read(true)
        .write(true) // not append - we do our own (non-atomic) seek
        .open(&note_file)
        .with_context(|| format!("Failed to open detected existing file {:?}", &note_file))?;
    let has_today = std::io::BufReader::new(&existing_file)
        .lines()
        .any(|line| line.is_ok_and(|s| s.starts_with(&day_header)));

    let mut buffer = [0; 1];
    if existing_file.seek(SeekFrom::End(-1)).is_ok()
        && existing_file.read(&mut buffer).is_ok()
        && buffer[0] == b'\n'
    {
        let _ = existing_file.seek_relative(-1);
    }

    Ok((existing_file, has_today))
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let now = chrono::offset::Local::now();
    let today = now.date();
    let friday = today + chrono::Duration::days(days_till_friday(&today) as i64);

    let args_joined = args.content.join("\n");
    let message = {
        let trimmed = args_joined.trim();
        if trimmed.starts_with("...") {
            Message {
                is_continuation: true,
                content: &trimmed[3..],
            }
        } else {
            Message {
                is_continuation: false,
                content: &trimmed,
            }
        }
    };

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
    let day_header = format!("## {}", format_date(&today));

    let (mut file, has_today, mut allow_continuation) = match fs::File::options()
        .write(true)
        .create_new(true)
        .open(&note_file)
    {
        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let (f, h) = file_seek_existing(&note_file, &day_header)?;
            (f, h, true)
        }
        Ok(f) => {
            buffer.push_str(&format!(
                "# Journal for week ending at {}\n",
                format_date(&friday)
            ));
            (f, false, false)
        }
        // Err(ref e) => return Err(e).with_context(|| format!("Unable to create journal file {:?}", &note_file))
        _ => panic!("Unable to create journal file {:?}", &note_file),
    };

    //Terminate these by only one newline. One more will be added with message.
    if !has_today {
        allow_continuation = false;
        buffer.push_str(&format!("\n\n{} - {}\n", &day_header, &today.format("%a")));
    }
    if let Some(header) = args.header {
        allow_continuation = false;
        buffer.push_str(&format!("\n\n### {} - {}\n", &now.format("%H:%M"), header));
    }

    if !message.content.is_empty() {
        if !(message.is_continuation && allow_continuation) {
            buffer.push_str("\n");
        } else {
            buffer.push_str(" ");
        }
        buffer.push_str(&format!("{}\n", message.content));
    }
    // This write should be atomic...
    file.write_all(buffer.as_bytes())
        .expect("Unable to write to file");
    Ok(())
}
