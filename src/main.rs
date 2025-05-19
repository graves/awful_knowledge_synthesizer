use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use awful_aj::{
    api::ask,
    config::{self, AwfulJadeConfig},
    template::{self, ChatTemplate},
};
use clap::Parser;
use clap::command;
use serde::{Deserialize, Serialize};
use text_splitter::{CodeSplitter, MarkdownSplitter, TextSplitter};
use tokio::time::sleep;
use walkdir::WalkDir;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, clap::ValueEnum, Ord, Debug)]
enum SourceType {
    Book,
    Manpage,
    Mdbook,
}

/// CLI arguments
#[derive(Parser, Debug)]
#[command(name = "awful_knowledge_synthesizer")]
#[command(about = "Generate final exam questions from YAML book chunks", long_about = None)]
struct Args {
    /// Path to directory of inputs
    #[arg(short, long)]
    input_dir: PathBuf,
    /// Configuration file
    #[arg(short, long)]
    config: PathBuf,
    /// Source type
    #[clap(value_enum)]
    #[arg(short, long)]
    source_type: SourceType,
    /// mdbook project name
    #[arg(short, long, requires_if("mdbook", "source_type"))]
    mdbook_name: String,
    /// Path to directory to output files
    #[arg(short, long)]
    output_dir: PathBuf,
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

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct ManpageQuestions {
    pub prompt: Option<String>,
    pub manpageQuestion1: Option<String>,
    pub manpageQuestion2: Option<String>,
    pub manpageQuestion3: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct MdbookQuestions {
    pub prompt: Option<String>,
    pub mdbookQuestion1: Option<String>,
    pub mdbookQuestion2: Option<String>,
    pub mdbookQuestion3: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let input_dir = args.input_dir;
    let conf_file = args.config;
    let source_type = args.source_type;
    let output_dir = args.output_dir;

    let template = match source_type {
        SourceType::Book => template::load_template("book_knowledge_synthesizer").await?,
        SourceType::Manpage => template::load_template("manpage_knowledge_synthesizer").await?,
        SourceType::Mdbook => template::load_template("mdbook_knowledge_synthesizer").await?,
    };

    let config =
        config::load_config(conf_file.to_str().expect("Not a valid config filename")).unwrap();

    println!("Reading {input_dir:?}");

    match source_type {
        SourceType::Book => {
            for entry in fs::read_dir(&input_dir)? {
                let entry = entry?;
                let path = entry.path();

                run_for_books(&path, &output_dir, &config, &template).await?;
            }
        }
        SourceType::Manpage => {
            for entry in fs::read_dir(&input_dir)? {
                let entry = entry?;
                let path = entry.path();

                run_for_manpages(&path, &output_dir, &config, &template).await?;
            }
        }
        SourceType::Mdbook => {
            let mdbook_name = args.mdbook_name;

            for entry in WalkDir::new(&input_dir) {
                let entry = entry?;
                let path = entry.path();

                run_for_mdbook(&mdbook_name, path, &output_dir, &config, &template).await?;
            }
        }
    };

    Ok(())
}

async fn run_for_mdbook(
    mdbook_name: &String,
    input_dir: &Path,
    output_dir: &Path,
    config: &AwfulJadeConfig,
    template: &ChatTemplate,
) -> Result<(), Box<dyn std::error::Error>> {
    if input_dir.is_file() && input_dir.extension().and_then(|s| s.to_str()) == Some("md") {
        let filename = input_dir.file_name().unwrap().to_string_lossy();
        let output_dir_name = output_dir.to_string_lossy();
        let page_contents = fs::read_to_string(input_dir)?;

        println!("File: {filename}\n");

        let page_and_extension = filename.split_terminator('.').collect::<Vec<&str>>();
        let page_name = page_and_extension[0].trim();

        let max_characters = 10_00..20_000;
        let splitter = MarkdownSplitter::new(max_characters);
        let chunks = splitter.chunks(&page_contents);

        let mut count = 1;
        let total = chunks.count();

        let chunks = splitter.chunks(&page_contents);

        for chunk in chunks {
            println!("Processing chunk {count}/{total}");

            let mdbook_details = format!(
                "You are playing the role of a college professor. Here is some text copied from the `{page_name} page of the documentation provided by {mdbook_name}`.\n\n"
            );
            let question = format!("{mdbook_details}\n\nDocumentation text:\n\n{chunk}");

            let response_string = fetch_with_backoff(config, &question, template).await?;

            let manpage_questions: Result<ManpageQuestions, serde_json::Error> =
                serde_json::from_str(&response_string);

            match manpage_questions {
                Ok(mut questions) => {
                    questions.prompt = Some(question);

                    // Serialize as single-item YAML
                    let yaml_entry = serde_yaml::to_string(&vec![questions])?; // serialize as 1-item array
                    let out_path = format!("{output_dir_name}/{mdbook_name}_questions.yaml");

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

    Ok(())
}

async fn run_for_manpages(
    input_dir: &PathBuf,
    output_dir: &Path,
    config: &AwfulJadeConfig,
    template: &ChatTemplate,
) -> Result<(), Box<dyn std::error::Error>> {
    if input_dir.is_file() && input_dir.extension().and_then(|s| s.to_str()) == Some("txt") {
        let filename = input_dir.file_name().unwrap().to_string_lossy();
        let output_dir_name = output_dir.to_string_lossy();
        let manpage_contents = fs::read_to_string(input_dir)?;

        println!("File: {filename}\n");

        let resource_and_extension = filename.split_terminator('.').collect::<Vec<&str>>();
        let resource = resource_and_extension[0].trim();

        let max_characters = 10_00..20_000;
        let splitter = TextSplitter::new(max_characters);

        let chunks = splitter.chunks(&manpage_contents);

        let mut count = 1;
        let total = chunks.count();

        let chunks = splitter.chunks(&manpage_contents);

        for chunk in chunks {
            println!("Processing chunk {count}/{total}");

            let resource_details = format!(
                "You are playing the role of a college professor. Here is some text copied from the manpages of the macOS resource `{resource}`.\n\n"
            );
            let question = format!("{resource_details}\n\nManpage text:\n\n{chunk}");

            let response_string = fetch_with_backoff(config, &question, template).await?;

            let manpage_questions: Result<ManpageQuestions, serde_json::Error> =
                serde_json::from_str(&response_string);

            match manpage_questions {
                Ok(mut questions) => {
                    questions.prompt = Some(question);

                    // Serialize as single-item YAML
                    let yaml_entry = serde_yaml::to_string(&vec![questions])?; // serialize as 1-item array
                    let out_path = format!("{output_dir_name}/{resource}_questions.yaml");

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

    Ok(())
}

async fn run_for_books(
    input_dir: &PathBuf,
    output_dir: &Path,
    config: &AwfulJadeConfig,
    template: &ChatTemplate,
) -> Result<(), Box<dyn std::error::Error>> {
    if input_dir.is_file() && input_dir.extension().and_then(|s| s.to_str()) == Some("yaml") {
        let filename = input_dir.file_name().unwrap().to_string_lossy();
        let output_dir_name = output_dir.to_string_lossy();
        let contents = fs::read_to_string(input_dir)?;

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

                for chunk in &book.chunks {
                    println!("Processing chunk {count}/{total}");

                    let book_details = format!("The text is from {title} by {author}.");
                    let question = format!("{chunk}\n\n{book_details}");

                    let response_string = fetch_with_backoff(config, &question, template).await?;

                    let final_exam_questions: Result<ExamQuestions, serde_json::Error> =
                        serde_json::from_str(&response_string);

                    match final_exam_questions {
                        Ok(mut questions) => {
                            questions.prompt = Some(question);

                            // Serialize as single-item YAML
                            let yaml_entry = serde_yaml::to_string(&vec![questions])?; // serialize as 1-item array
                            let out_path = format!("{output_dir_name}/{title}_questions.yaml");

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

            eprintln!("Retrying in {backoff} ms...");

            sleep(Duration::from_millis(backoff)).await;
        }
    }

    Err("Hyper timeout".into())
}
