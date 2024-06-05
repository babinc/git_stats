use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path};
use std::process::Command;
use num_format::{Locale, ToFormattedString};

#[derive(Parser, Debug)]
#[command(author, version, about = "Counts lines of code per author in a Git repository",
long_about = "Counts lines of code per author in a Git repository.\n\n\
                        Example usage:\n\
                        cargo run -- --path C:/Path/To/Your/Repo --extensions rs js py")]
struct Args {
    /// Path to the Git repository
    #[arg(short, long, required = true)]
    path: String,

    /// File extensions to include
    #[arg(short, long, use_value_delimiter = true, value_delimiter = ',', required = true)]
    extensions: Vec<String>,
}

struct Model {
    author: String,
    lines: u32,
    percent: f64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let repo_path = Path::new(&args.path);
    let extensions: Vec<String> = args.extensions.iter().map(|e| e.to_lowercase()).collect();

    let files = get_tracked_files(repo_path)?;
    let mut author_lines: HashMap<String, usize> = HashMap::new();


    for file in files {
        if let Some(ext) = Path::new(&file).extension() {
            if extensions.contains(&ext.to_str().unwrap().to_lowercase()) {
                println!("Processing {}...", file);
                let blame_output = get_blame_output(repo_path, &file)?;
                parse_blame_output(&blame_output, &mut author_lines);
            }
        }
    }

    let mut total_lines: i32 = 0;
    for (_, count) in &author_lines {
        total_lines = total_lines + count.to_owned() as i32;
    }

    let formatted_total_lines = total_lines.to_formatted_string(&Locale::en);
    println!("\nTotal Lines of Code: {:}", formatted_total_lines);

    println!("\nLines of code per author:");

    let mut data: Vec<Model> = vec![];
    for (author, count) in &author_lines {
        let percent = (count.to_owned() as f64 / total_lines as f64) * 100.0;
        data.push(Model {
            author: author.to_string(),
            lines: count.to_owned() as u32,
            percent,
        })
    }

    data.sort_by_key(|x| (x.percent as i32) * -1);

    for item in data.iter() {
        let lines = item.lines.to_formatted_string(&Locale::en);
        println!("{}: {}, {:.1}%", item.author, lines, item.percent);
    }

    Ok(())
}

fn get_tracked_files(repo_path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("ls-files")
        .current_dir(repo_path)
        .output()?;

    let files = String::from_utf8(output.stdout)?;
    Ok(files.lines().map(|s| s.to_string()).collect())
}

fn get_blame_output(repo_path: &Path, file: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("blame")
        .arg("--line-porcelain")
        .arg(file)
        .current_dir(repo_path)
        .output()?;

    Ok(String::from_utf8(output.stdout)?)
}

fn parse_blame_output(blame_output: &str, author_lines: &mut HashMap<String, usize>) {
    let author_re = Regex::new(r"^author (.+)$").unwrap();

    for line in blame_output.lines() {
        if let Some(caps) = author_re.captures(line) {
            let author = caps.get(1).unwrap().as_str().to_string();
            *author_lines.entry(author).or_insert(0) += 1;
        }
    }
}

