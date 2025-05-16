use std::{error::Error, fs, path::PathBuf, time::Duration};

use awful_aj::{
    api::ask,
    config::{self, AwfulJadeConfig},
    template::{self, ChatTemplate},
};
use clap::Parser;
use clap::command;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

/// CLI arguments
#[derive(Parser, Debug)]
#[command(name = "awful_knowledge_synthesizer")]
#[command(about = "Generate final exam questions from YAML book chunks", long_about = None)]
struct Args {
    /// Path to directory of .yaml book files
    #[arg(short, long)]
    dir: PathBuf,
    /// Configuration file
    #[arg(short, long)]
    config: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
struct Book {
    pub title: Option<String>,
    pub author: Option<String>,
    pub chunks: Vec<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct ExamQuestions {
    pub prompt: Option<String>,
    pub finalExamQuestion1: Option<String>,
    pub finalExamQuestion2: Option<String>,
    pub finalExamQuestion3: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let dir_path = args.dir;
    let conf_file = args.config;

    let template = template::load_template("book_knowledge_synthesizer").await?;

    let config =
        config::load_config(conf_file.to_str().expect("Not a valid config filename")).unwrap();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            let filename = path.file_name().unwrap().to_string_lossy();
            let contents = fs::read_to_string(&path)?;

            println!("File: {filename}\n");

            let author_and_title = filename.split_terminator('|').collect::<Vec<&str>>();
            let author = author_and_title[0].trim();
            let title = author_and_title[1]
                .trim()
                .split_terminator(".")
                .collect::<Vec<&str>>()[0];

            let book_result: Result<Book, serde_yaml::Error> = serde_yaml::from_str(&contents);

            match book_result {
                Ok(mut book) => {
                    book.author = Some(author.to_string());
                    book.title = Some(title.to_string());

                    println!("{:?}", book.author);
                    println!("{:?}", book.title);

                    let mut count = 1;
                    let total = book.chunks.len();

                    for chunk in book.chunks {
                        println!("Processing chunk {count}/{total}");

                        let book_details = format!("The text is from {title} by {author}.");
                        let question = format!("{chunk}\n\n{book_details}");

                        let response_string =
                            fetch_with_backoff(&config, &question, &template).await?;

                        let final_exam_questions: Result<ExamQuestions, serde_json::Error> =
                            serde_json::from_str(&response_string);

                        match final_exam_questions {
                            Ok(mut questions) => {
                                questions.prompt = Some(question);

                                // Serialize as single-item YAML
                                let yaml_entry = serde_yaml::to_string(&vec![questions])?; // serialize as 1-item array
                                let out_path = format!("{title}_questions.yaml");

                                use std::io::Write;
                                let mut file = fs::OpenOptions::new()
                                    .create(true)
                                    .append(true)
                                    .open(&out_path)?;

                                writeln!(file, "{yaml_entry}")?;
                                println!("Wrote to {out_path}");
                            }
                            err => {
                                println!("ERROR: {err:?}");
                            }
                        }

                        count += 1;
                    }
                }
                _ => println!("Failed to deserialize: {filename}"),
            }
        };
    }

    Ok(())
}

const MAX_RETRIES: u32 = 5;
const BASE_DELAY_MS: u64 = 500;

async fn fetch_with_backoff(
    config: &AwfulJadeConfig,
    chunk: &str,
    template: &ChatTemplate,
) -> Result<String, Box<dyn std::error::Error>> {
    for attempt in 0..=MAX_RETRIES {
        let res = ask(config, chunk.to_string(), template, None, None).await;

        match res {
            Ok(response) => {
                return Ok(response);
            }
            Err(err) => {
                eprintln!("Request failed: {err}");
            }
        }

        if attempt < MAX_RETRIES {
            let backoff = BASE_DELAY_MS * (2u64.pow(attempt));

            eprintln!("Retrying in {backoff}ms...");

            sleep(Duration::from_millis(backoff)).await;
        }
    }

    Err("Hyper timeout".into())
}
