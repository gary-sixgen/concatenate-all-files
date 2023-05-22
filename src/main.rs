use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write, Read};
use mime_guess::mime::TEXT;
use walkdir::WalkDir;
use chardetng::EncodingDetector;
use clap::Parser;
use mime_guess::from_path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// output file name
    #[arg(short, long)]
    output: String,
}

/// A simple app to concatenate all text files in a directory into one file.
fn main() {
    let args = Args::parse();

    let out_path = env::current_dir().expect("failed to get current directory").join(&args.output);
    let mut out_file = File::create(&out_path).expect("failed to create output file");

    let root = env::current_dir().expect("failed to get current directory");
    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.path().to_string_lossy().contains("node_modules"))
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path() != out_path)  // Skip the output file
    {
        let path = entry.path();
        println!("{:?}", path);

        
        // Determine MIME type of the file, proceed only if it's a text file
        if let Some(mime_type) = from_path(path).first() {
            if mime_type.type_() == TEXT {
                let mut detector = EncodingDetector::new();
                let mut buffer = vec![0; 4096];
                let mut file = File::open(&path).expect("failed to open file");
                loop {
                    let n = file.read(&mut buffer).expect("read failed");
                    detector.feed(&buffer[..n], false);
                    if n < buffer.len() {
                        break;
                    }
                }
                let encoding = detector.guess(None, true);
                if encoding.name() == "windows-1252" || encoding.name().starts_with("UTF-") {
                    writeln!(out_file, "\n\n\n{}:", path.to_string_lossy()).expect("failed to write file name");
                    let file = File::open(path).expect("failed to open file");
                    for line in BufReader::new(file).lines() {
                        if let Ok(l) = line {
                            writeln!(out_file, "{}", l).expect("failed to write line");
                        } else {
                            eprintln!("Warning: encountered a non-UTF-8 line in file {}", path.display());
                            continue;
                        }
                    }
                }
            }
        }
    }
}