//! # awful_knowledge_synthesizer
//!
//! A command-line tool that walks books, manpages, mdBooks, tealdeer pages, or
//! source-code repositories, **splits content into chunks**, and asks an LLM to
//! synthesize *final-exam* / *documentation* / *manpage* / *tldr* / *code-review*
//! questions. Each model response is parsed into a typed question record and
//! **appended** to YAML output files for downstream dataset building.
//!
//! ## How it works
//! 1. Select a chat template based on [`SourceType`].
//! 2. Recursively or shallowly scan the input directory (varies by source type).
//! 3. Split each eligible file into size-bounded chunks using `text_splitter`:
//!    - Books/manpages: [`TextSplitter`] (plain text)
//!    - mdBook/tealdeer: [`MarkdownSplitter`] (markdown aware)
//!    - Code: [`CodeSplitter`] (Tree-sitter aware, per-language)
//! 4. For each chunk, build a source-specific prompt and call the model with
//!    [`fetch_with_backoff`] (exponential backoff).
//! 5. Parse the response JSON into a typed `*Questions` struct and **append** a
//!    one-item YAML array to the output file in `output_dir`.
//!
//! ## Input types & expected outputs
//! - `--source-type book` → emits [`ExamQuestions`] to `{title}_questions.yaml`
//! - `--source-type manpage` → emits [`ManpageQuestions`] to `{resource}_questions.yaml`
//! - `--source-type mdbook` → emits [`MdbookQuestions`] to `{mdbook_name}_questions.yaml`
//! - `--source-type tealdeer` → emits [`TealdeerQuestions`] to `Tealdeer_questions.yaml`
//! - `--source-type code` → emits [`CodeQuestions`] to `{project_name}_questions.yaml`
//!
//! The model is expected to return a JSON object matching the corresponding
//! `*Questions` struct (fields documented below). The tool injects the exact
//! prompt it sent into the `.prompt` field before writing.
//!
//! ## CLI
//! ```text
//! awful_knowledge_synthesizer \
//!   --input-dir ./inputs \
//!   --output-dir ./out \
//!   --config ./awfuljade.yaml \
//!   --source-type <book|manpage|mdbook|tealdeer|code> \
//!   [--mdbook-name MyBook] \
//!   [--language <asm|c|rust> --project-name MyRepo]
//! ```
//!
//! - `--input-dir`: Folder containing the raw sources to process.
//! - `--output-dir`: Where YAML outputs are appended. Files are created if missing.
//! - `--config`: Configuration file consumed by `awful_aj::config::load_config`.
//! - `--source-type`: Selects traversal mode, splitter, prompt prefix, and output schema.
//! - `--mdbook-name`: Required when `--source-type mdbook`; used in prompts and filename.
//! - `--language` & `--project-name`: Required when `--source-type code`; selects parser and filename.
//!
//! ## Splitting behavior
//! The tool uses bounded-character chunkers from `text_splitter`. For code, it uses
//! Tree-sitter grammars (ASM, C, Rust) to keep chunks syntactically coherent. For
//! Markdown and plain text, it tries to respect structure while fitting within the
//! configured character ranges.
//!
//! ## Failure behavior
//! - Model calls: retried with exponential backoff up to [`MAX_RETRIES`] (see
//!   [`fetch_with_backoff`]); if all fail, a `"Hyper timeout"` error is returned.
//! - Parse errors: if the model response cannot be deserialized to the expected
//!   `*Questions` struct, the error is printed and the chunk is skipped.
//! - IO/YAML errors: surfaced and printed; the process continues to next file/chunk.
//!
//! ## Examples
//! ```bash
//! # Synthesize questions from a recursive mdBook tree:
//! awful_knowledge_synthesizer \
//!   --input-dir ./book-md \
//!   --output-dir ./out \
//!   --config ./aj.yaml \
//!   --source-type mdbook \
//!   --mdbook-name "Rust Book"
//!
//! # Generate code-review questions for a Rust repo:
//! awful_knowledge_synthesizer \
//!   --input-dir ./repo \
//!   --output-dir ./out \
//!   --config ./aj.yaml \
//!   --source-type code \
//!   --language rust \
//!   --project-name my_crate
//! ```

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

/// The semantic source to be processed; selects traversal, template, schema, and output.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, clap::ValueEnum, Ord, Debug)]
enum SourceType {
    /// Book-like sources (YAML-encoded `Book` with `chunks`).
    Book,
    /// Plain-text manpages (e.g., `.txt` dumps).
    Manpage,
    /// mdBook projects (Markdown).
    Mdbook,
    /// tealdeer/tldr pages (Markdown).
    Tealdeer,
    /// Source repositories (code files).
    Code,
}

/// Code-language selection for Tree-sitter aware splitting and prompt hints.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, clap::ValueEnum, Ord, Debug)]
enum Language {
    /// Assembly (`.s`).
    Asm,
    /// C (`.c`, `.h`).
    C,
    /// Rust (`.rs`).
    Rust,
}

/// Command-line interface for `awful_knowledge_synthesizer`.
#[derive(Parser, Debug)]
#[command(name = "awful_knowledge_synthesizer")]
#[command(about = "Generate final exam questions from YAML book chunks", long_about = None)]
struct Args {
    /// Path to directory of inputs.
    #[arg(short, long)]
    input_dir: PathBuf,

    /// Configuration file for the model backend (see `awful_aj` crate).
    #[arg(short, long)]
    config: PathBuf,

    /// Source type that determines parsing/splitting/output schema.
    #[clap(value_enum)]
    #[arg(short, long)]
    source_type: SourceType,

    /// mdBook project name (required when `--source-type mdbook`).
    #[arg(short, long, requires_if("mdbook", "source_type"))]
    mdbook_name: Option<String>,

    /// Path to directory where YAML outputs are appended.
    #[arg(short, long)]
    output_dir: PathBuf,

    /// Language of the code repository (required when `--source-type code`).
    #[arg(short, long, requires_if("code", "source_type"))]
    language: Option<Language>,

    /// Code repo project name (required when `--source-type code`).
    #[arg(short, long, requires_if("code", "source_type"))]
    project_name: Option<String>,
}

/// Book metadata plus pre-chunked content. Usually deserialized from YAML files.
#[derive(Debug, Deserialize, Serialize)]
struct Book {
    /// Optional book title; will be derived from filename if absent.
    pub title: Option<String>,
    /// Optional author; will be derived from filename if absent.
    pub author: Option<String>,
    /// Sequential textual chunks to be sent to the model.
    pub chunks: Vec<String>,
}

/// Book-style question payload returned by the model.
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct ExamQuestions {
    /// The exact prompt text sent to the model (injected by the tool).
    pub prompt: Option<String>,
    /// First exam question synthesized by the model.
    pub finalExamQuestion1: Option<String>,
    /// Second exam question synthesized by the model.
    pub finalExamQuestion2: Option<String>,
    /// Third exam question synthesized by the model.
    pub finalExamQuestion3: Option<String>,
}

/// Manpage-style question payload returned by the model.
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct ManpageQuestions {
    /// The exact prompt text sent to the model (injected by the tool).
    pub prompt: Option<String>,
    /// First question for the manpage chunk.
    pub manpageQuestion1: Option<String>,
    /// Second question for the manpage chunk.
    pub manpageQuestion2: Option<String>,
    /// Third question for the manpage chunk.
    pub manpageQuestion3: Option<String>,
}

/// mdBook-style question payload returned by the model.
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct MdbookQuestions {
    /// The exact prompt text sent to the model (injected by the tool).
    pub prompt: Option<String>,
    /// First documentation question for the mdBook chunk.
    pub documentationQuestion1: Option<String>,
    /// Second documentation question for the mdBook chunk.
    pub documentationQuestion2: Option<String>,
    /// Third documentation question for the mdBook chunk.
    pub documentationQuestion3: Option<String>,
}

/// tealdeer/tldr-style question payload returned by the model.
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct TealdeerQuestions {
    /// The exact prompt text sent to the model (injected by the tool).
    pub prompt: Option<String>,
    /// First question for the tealdeer page chunk.
    pub tealdeerQuestion1: Option<String>,
    /// Second question for the tealdeer page chunk.
    pub tealdeerQuestion2: Option<String>,
    /// Third question for the tealdeer page chunk.
    pub tealdeerQuestion3: Option<String>,
}

/// Code-review-style question payload returned by the model.
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct CodeQuestions {
    /// The exact prompt text sent to the model (injected by the tool).
    pub prompt: Option<String>,
    /// First question for the code chunk.
    pub codeQuestion1: Option<String>,
    /// Second question for the code chunk.
    pub codeQuestion2: Option<String>,
    /// Third question for the code chunk.
    pub codeQuestion3: Option<String>,
}

/// Entry point: parses arguments, chooses the template, walks inputs,
/// splits into chunks, queries the model, and appends YAML question records.
///
/// Returns `Ok(())` on success; otherwise surfaces IO/parse/model errors.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let input_dir = args.input_dir;
    let conf_file = args.config;
    let source_type = args.source_type;
    let output_dir = args.output_dir;
    let language = args.language;
    let project_name = args.project_name;

    // Select a source-appropriate prompt template.
    let template = match source_type {
        SourceType::Book => template::load_template("book_knowledge_synthesizer").await?,
        SourceType::Manpage => template::load_template("manpage_knowledge_synthesizer").await?,
        SourceType::Mdbook => template::load_template("mdbook_knowledge_synthesizer").await?,
        SourceType::Tealdeer => template::load_template("tealdeer_knowledge_synthesizer").await?,
        SourceType::Code => template::load_template("code_knowledge_synthesizer").await?,
    };

    // Load runtime configuration for the model backend.
    let config =
        config::load_config(conf_file.to_str().expect("Not a valid config filename")).unwrap();

    println!("Reading {input_dir:?}");

    match source_type {
        SourceType::Book => {
            // Shallow directory scan for YAML book files.
            for entry in fs::read_dir(&input_dir)? {
                let entry = entry?;
                let path = entry.path();

                run_for_books(&path, &output_dir, &config, &template).await?;
            }
        }
        SourceType::Manpage => {
            // Shallow directory scan for plain-text manpage files.
            for entry in fs::read_dir(&input_dir)? {
                let entry = entry?;
                let path = entry.path();

                run_for_manpages(&path, &output_dir, &config, &template).await?;
            }
        }
        SourceType::Mdbook => {
            // Recursive walk for mdBook markdown.
            let mdbook_name = args.mdbook_name.unwrap();

            for entry in WalkDir::new(&input_dir) {
                let entry = entry?;
                let path = entry.path();

                run_for_mdbook(&mdbook_name, path, &output_dir, &config, &template).await?;
            }
        }
        SourceType::Tealdeer => {
            // Recursive walk for tealdeer markdown.
            for entry in WalkDir::new(&input_dir) {
                let entry = entry?;
                let path = entry.path();

                run_for_tealdeer(path, &output_dir, &config, &template).await?;
            }
        }
        SourceType::Code => {
            // Recursive walk for code files; requires language & project name.
            let project_name = project_name.unwrap();

            for entry in WalkDir::new(&input_dir) {
                let entry = entry?;
                let path = entry.path();

                run_for_code(
                    &language.unwrap(),
                    &project_name,
                    path,
                    &output_dir,
                    &config,
                    &template,
                )
                .await?;
            }
        }
    };

    Ok(())
}

/// Process a code file (if extension matches the selected [`Language`]).
///
/// - Detects eligible files by extension: `asm` → `.s`, `c` → `.c`/`.h`, `rust` → `.rs`.
/// - Splits with a Tree-sitter powered [`CodeSplitter`] into bounded chunks.
/// - For each chunk, builds a code-review style prompt, calls the model with
///   [`fetch_with_backoff`], injects `.prompt`, and appends a one-item YAML array
///   of [`CodeQuestions`] to `{output_dir}/{project_name}_questions.yaml`.
///
/// Non-matching files are silently skipped.
///
/// # Errors
/// Returns any IO/model/serialization errors encountered during processing.
async fn run_for_code(
    language: &Language,
    project_name: &String,
    input_dir: &Path,
    output_dir: &Path,
    config: &AwfulJadeConfig,
    template: &ChatTemplate,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_extension = if input_dir.is_file() {
        input_dir
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
    } else {
        ""
    };

    if ["s", "h", "c", "rs"].contains(&file_extension) {
        let filename = input_dir.file_name().unwrap().to_string_lossy();
        let output_dir_name = output_dir.to_string_lossy();

        println!("File: {filename}\n");

        // Character bounds for semantic code chunking.
        let max_characters = 5_000..10_000;

        let splitter = match language {
            Language::Asm => CodeSplitter::new(tree_sitter_asm::LANGUAGE, max_characters)
                .expect("Invalid tree-sitter language"),
            Language::C => CodeSplitter::new(tree_sitter_c::LANGUAGE, max_characters)
                .expect("Invalid tree-sitter language"),
            Language::Rust => CodeSplitter::new(tree_sitter_rust::LANGUAGE, max_characters)
                .expect("Invalid tree-sitter language"),
        };

        // Load file contents only when the extension matches the chosen language.
        let (lang_str, file_contents) = match language {
            Language::Asm => {
                if file_extension == "s" {
                    ("asm", fs::read_to_string(input_dir)?)
                } else {
                    ("asm", "".to_string())
                }
            }
            Language::C => {
                if ["c", "h"].contains(&file_extension) {
                    ("c", fs::read_to_string(input_dir)?)
                } else {
                    ("c", "".to_string())
                }
            }
            Language::Rust => {
                if file_extension == "rs" {
                    ("rust", fs::read_to_string(input_dir)?)
                } else {
                    ("rust", "".to_string())
                }
            }
        };

        let chunks = splitter.chunks(&file_contents);

        let mut count = 1;
        let total = chunks.count();

        let chunks = splitter.chunks(&file_contents);

        for chunk in chunks {
            println!("Processing chunk {count}/{total}");

            let input_dir_string = input_dir.to_string_lossy();
            let command_details = format!(
                "You are playing the role of a senior software engineer developing questions for a code review. Here is some source code from {input_dir_string}. It is part of the {project_name} project.\n\n"
            );
            let question =
                format!("{command_details}\n\nSource Code:\n\n```{lang_str}\n{chunk}\n```");

            let response_string = fetch_with_backoff(config, &question, template).await?;

            let aarch64_questions: Result<CodeQuestions, serde_json::Error> =
                serde_json::from_str(&response_string);

            match aarch64_questions {
                Ok(mut questions) => {
                    questions.prompt = Some(question);

                    // Serialize as single-item YAML
                    let yaml_entry = serde_yaml::to_string(&vec![questions])?; // serialize as 1-item array
                    let out_path = format!("{output_dir_name}/{project_name}_questions.yaml");

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

/// Process a tealdeer (tldr) Markdown file.
///
/// - Filters for `*.md`.
/// - Splits with [`MarkdownSplitter`], prompts as a professor synthesizing documentation
///   questions, and appends [`TealdeerQuestions`] to `Tealdeer_questions.yaml`.
///
/// # Errors
/// Returns IO/model/serialization errors for the given file, if any.
async fn run_for_tealdeer(
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

        let command_and_extension = filename.split_terminator('.').collect::<Vec<&str>>();
        let command_name = command_and_extension[0].trim();

        // Character bounds for markdown-aware chunking.
        let max_characters = 10_00..20_000;
        let splitter = MarkdownSplitter::new(max_characters);
        let chunks = splitter.chunks(&page_contents);

        let mut count = 1;
        let total = chunks.count();

        let chunks = splitter.chunks(&page_contents);

        for chunk in chunks {
            println!("Processing chunk {count}/{total}");

            let command_details = format!(
                "You are playing the role of a college professor. Here is some output of the `tldr {command_name}` commmand provided by the open source library tealdeer.\n\n"
            );
            let question = format!("{command_details}\n\nTeeldear text:\n\n{chunk}");

            let response_string = fetch_with_backoff(config, &question, template).await?;

            let tealdeer_questions: Result<TealdeerQuestions, serde_json::Error> =
                serde_json::from_str(&response_string);

            match tealdeer_questions {
                Ok(mut questions) => {
                    questions.prompt = Some(question);

                    // Serialize as single-item YAML
                    let yaml_entry = serde_yaml::to_string(&vec![questions])?; // serialize as 1-item array
                    let out_path = format!("{output_dir_name}/Tealdeer_questions.yaml");

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

/// Process an mdBook Markdown file.
///
/// - Filters for `*.md`.
/// - Splits with [`MarkdownSplitter`], prompts as a professor citing the page and
///   mdBook name, and appends [`MdbookQuestions`] to `{mdbook_name}_questions.yaml`.
///
/// # Errors
/// Returns IO/model/serialization errors for the given file, if any.
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

        // Character bounds for markdown-aware chunking.
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

            let mdbook_questions: Result<MdbookQuestions, serde_json::Error> =
                serde_json::from_str(&response_string);

            match mdbook_questions {
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

/// Process a manpage text file.
///
/// - Filters for `*.txt`.
/// - Splits with [`TextSplitter`] (plain text), prompts as a professor citing
///   the macOS resource name, and appends [`ManpageQuestions`] to
///   `{resource}_questions.yaml`.
///
/// # Errors
/// Returns IO/model/serialization errors for the given file, if any.
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

        // Character bounds for plain-text chunking.
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

/// Process a YAML-encoded book file into exam questions.
///
/// - Filters for `*.yaml`.
/// - Deserializes to [`Book`], derives `author` and `title` from the filename
///   (`{author}|{title}.yaml`), iterates `book.chunks`, prompts the model, and
///   appends [`ExamQuestions`] to `{title}_questions.yaml`.
///
/// # Errors
/// Returns IO/model/serialization errors for the given file, if any; if the book
/// YAML fails to parse, logs the filename and continues.
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

                let mut count = 0;
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

/// Maximum number of retries for a model request.
const MAX_RETRIES: u32 = 5;
/// Initial backoff in milliseconds; doubles for each successive retry.
const BASE_DELAY_MS: u64 = 500;

/// Call the model with exponential backoff.
///
/// Tries the request up to `MAX_RETRIES + 1` times, waiting
/// `BASE_DELAY_MS * 2^attempt` milliseconds between attempts. Logs per-attempt
/// errors and delay information.
///
/// # Errors
/// Returns `"Hyper timeout"` if all attempts fail.
///
/// # Examples
/// ```no_run
/// # async fn demo(cfg: &awful_aj::config::AwfulJadeConfig, t: &awful_aj::template::ChatTemplate)
/// # -> Result<(), Box<dyn std::error::Error>> {
/// let answer = fetch_with_backoff(cfg, "Explain iterators vs generators", t).await?;
/// println!("{answer}");
/// # Ok(()) }
/// ```
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
