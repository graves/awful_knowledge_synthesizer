# 🧠 Awful Knowledge Synthesizer: Transforming Text into Exam Questions  

> *A tool to generate LLM-powered exam questions from YAML books, manpages, mdbooks, and more.*

```
    _______________________________________________________
   |:::::: o o o o . |..... . .. . | [45]  o o o o o ::::::|
   |:::::: o o o o   | ..  . ..... |       o o o o o ::::::|
   |::::::___________|__..._...__._|_________________::::::|
   | # # | # # # | # # | # # # | # # | # # # | # # | # # # |
   | # # | # # # | # # | # # # | # # | # # # | # # | # # # |
   | # # | # # # | # # | # # # | # # | # # # | # # | # # # |
   | | | | | | | | | | | | | | | | | | | | | | | | | | | | |
   |_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|_|

                                 -Mr R J Craggs-
```

```
λ awful_knowledge_synthesizer --help
Generate final exam questions from YAML book chunks

Usage: awful_knowledge_synthesizer [OPTIONS] --input-dir <INPUT_DIR> --config <CONFIG> --source-type <SOURCE_TYPE> --output-dir <OUTPUT_DIR>

Options:
  -i, --input-dir <INPUT_DIR>        Path to directory of inputs
  -c, --config <CONFIG>              Configuration file
  -s, --source-type <SOURCE_TYPE>    Source type [possible values: book, manpage, mdbook, tealdeer, code]
  -m, --mdbook-name <MDBOOK_NAME>    mdbook project name
  -o, --output-dir <OUTPUT_DIR>      Path to directory to output files
  -l, --language <LANGUAGE>          Language of the code repository [possible values: asm, c, rust]
  -p, --project-name <PROJECT_NAME>  Code repo project name
  -h, --help                         Print help
```

---

## 📚 What Is This?  

**`awful_knowledge_synthesizer`** is a command-line tool that takes YAML files (and other text formats) containing book excerpts, manpages, or code snippets and generates **exam questions** for Large Language Models (LLMs).  

### 🔍 Key Features  
- Supports multiple source types: `yaml`, `manpage`, `mdbook`, `tealdeer`, and `code`.  
- Uses templates to format prompts for LLMs (e.g., "You are a senior software engineer...").  
- Outputs YAML files with question-answer pairs (e.g., `SQLite_questions.yaml`).  
- Integrates with [Awful Jade](https://github.com/awful_aj) for LLM inference and conversation persistence.  
- Was used to generate finetuning datasets for the [Jade](https://github.com/graves/Jade) iOS app.  

---


### 🧩 **How It Works**  

This tool transforms text from various sources into exam questions using Large Language Models (LLMs). Here’s a breakdown of how each input type is processed:

---

#### 📄 **Books (YAML Files)**  
- **Input**: YAML files with structured text chunks (e.g., `GrammarLogicRhetoricMath.yaml`).  
- **Process**:  
  - Parses YAML files to extract text chunks.  
  - Splits the content into manageable fragments (with default chunk size).  
  - Uses a LLM template to generate exam questions based on the text.  
- **Output**: Questions are saved in `_questions.yaml` files (e.g., `GrammarLogicRhetoricMath_questions.yaml`).  

---

#### 📜 **Manpages (.txt Files)**  
- **Input**: `.txt` files containing macOS manpage content (e.g., `4ccconv.txt`).  
- **Process**:  
  - Reads `.txt` files and splits them into chunks.  
  - Uses a LLM template to create questions about the text.  
- **Output**: Questions are saved in `_questions.yaml` files (e.g., `4ccconv_questions.yaml`).  

---

#### 📄 **MDBooks (.md Files in Nested Directories)**  
- **Input**: Markdown files under a directory structure (e.g., `cargo/` for Cargo documentation).  
- **Process**:  
  - Recursively scans directories for `.md` files.  
  - Splits markdown content into chunks and generates questions about the text.  
- **Output**: Questions are saved in format `mdbook_name_questions.yaml` (e.g., `Cargo_questions.yaml`).  

---

#### 🦌 **Tealdeer (.md Files with `tldr` Commands)**  
- **Input**: Markdown files containing `tldr` command outputs (e.g., `aa.md`).  
- **Process**:  
  - Extracts the command name from filenames (e.g., `aa.md → tldr aa`).  
  - Splits markdown content into chunks and generates questions about the `tldr` output.  
- **Output**: Questions are saved in `Tealdeer_questions.yaml`.  

---

#### 👾 **Code Files (C, Rust, or Assembly)**  
- **Input**: Source code files with extensions like `.c`, `.rs`, or `.asm`.  
- **Process**:  
  - Uses the command line flag to determine its language (C, Rust, or Assembly).  
  - Uses a code-specific splitter to divide the content into chunks.  
  - Generates questions tailored for developers (e.g., "What is this function doing?").  
- **Output**: Questions are saved in `project_name_questions.yaml` (e.g., `SQLite_questions.yaml`).  

---

### 🧪 **Key Workflow**  
1. **Input Parsing**:  
   - YAML files (books), `.txt`/`.md` files, or source code.  
   - Each type is handled by a dedicated function (`run_for_books`, `run_for_manpages`, etc.).  

2. **Chunking**:  
   - Text is split into manageable fragments (e.g., 1000–20,000 characters).  
   - Code files are split based on language (e.g., `tree-sitter` parsers for C/Rust).  

3. **LLM Prompting**:  
   - All inputs are converted into questions using a LLM template (e.g., "You are a professor...").  

4. **Output**:  
   - Questions are saved in YAML files with structured formatting (e.g., `project_name_questions.yaml`).  

---

## 📦 Example Usage  

### ✅ Basic Command  

```bash
λ awful_knowledge_synthesizer --help
Generate final exam questions from YAML book chunks

Usage: awful_knowledge_synthesizer [OPTIONS] --input-dir <INPUT_DIR> --config <CONFIG> --source-type <SOURCE_TYPE> --output-dir <OUTPUT_DIR>

Options:
  -i, --input-dir <INPUT_DIR>        Path to directory of inputs
  -c, --config <CONFIG>              Configuration file
  -s, --source-type <SOURCE_TYPE>    Source type [possible values: book, manpage, mdbook, tealdeer, code]
  -m, --mdbook-name <MDBOOK_NAME>    mdbook project name
  -o, --output-dir <OUTPUT_DIR>      Path to directory to output files
  -l, --language <LANGUAGE>          Language of the code repository [possible values: asm, c, rust]
  -p, --project-name <PROJECT_NAME>  Code repo project name
  -h, --help                         Print help
```

```bash
λ awful_knowledge_synthesizer --input-dir inputs/code/sqlite --config config.yaml --source-type code --language c --output-dir . --project-name "SQLite"
Reading "inputs/code/sqlite"
File: jimsh0.c

Processing chunk 1/116
Wrote to ./SQLite_questions.yaml
Processing chunk 2/116
```

### ✅ Command Output
  
`SQLite_questions.yaml`:

```yaml
- prompt: "You are playing the role of a senior software engineer developing questions for a code review. Here is some source code from inputs/code/sqlite/autosetup/jimsh0.c. It is part of the SQLite project.\n\n\n\nSource Code:\n\n```c\n/* This is single source file, bootstrap version of Jim Tcl. See http://jim.tcl.tk/ */\n#define JIM_COMPAT\n#define JIM_ANSIC\n#define JIM_REGEXP\n#define HAVE_NO_AUTOCONF\n#define JIM_TINY\n#define _JIMAUTOCONF_H\n#define TCL_LIBRARY \".\"\n#define jim_ext_bootstrap\n#define jim_ext_aio\n#define jim_ext_readdir\n#define jim_ext_regexp\n#define jim_ext_file\n#define jim_ext_glob\n#define jim_ext_exec\n#define jim_ext_clock\n#define jim_ext_array\n#define jim_ext_stdlib\n#define jim_ext_tclcompat\n#if defined(_MSC_VER)\n#define TCL_PLATFORM_OS \"windows\"\n#define TCL_PLATFORM_PLATFORM \"windows\"\n#define TCL_PLATFORM_PATH_SEPARATOR \";\"\n#define HAVE_MKDIR_ONE_ARG\n#define HAVE_SYSTEM\n#elif defined(__MINGW32__)\n#define TCL_PLATFORM_OS \"mingw\"\n#define TCL_PLATFORM_PLATFORM \"windows\"\n#define TCL_PLATFORM_PATH_SEPARATOR \";\"\n#define HAVE_MKDIR_ONE_ARG\n#define HAVE_SYSTEM\n#define HAVE_SYS_TIME_H\n#define HAVE_DIRENT_H\n#define HAVE_UNISTD_H\n#define HAVE_UMASK\n#include <sys/stat.h>\n#ifndef S_IRWXG\n#define S_IRWXG 0\n#endif\n#ifndef S_IRWXO\n#define S_IRWXO 0\n#endif\n#else\n#define TCL_PLATFORM_OS \"unknown\"\n#define TCL_PLATFORM_PLATFORM \"unix\"\n#define TCL_PLATFORM_PATH_SEPARATOR \":\"\n#ifdef _MINIX\n#define vfork fork\n#define _POSIX_SOURCE\n#else\n#define _GNU_SOURCE\n#endif\n#define HAVE_FORK\n#define HAVE_WAITPID\n#define HAVE_ISATTY\n#define HAVE_MKSTEMP\n#define HAVE_LINK\n#define HAVE_SYS_TIME_H\n#define HAVE_DIRENT_H\n#define HAVE_UNISTD_H\n#define HAVE_UMASK\n#define HAVE_PIPE\n#define _FILE_OFFSET_BITS 64\n#endif\n#define JIM_VERSION 84\n#ifndef JIM_WIN32COMPAT_H\n#define JIM_WIN32COMPAT_H\n\n\n\n#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n\n#if defined(_WIN32) || defined(WIN32)\n\n#define HAVE_DLOPEN\nvoid *dlopen(const char *path, int mode);\nint dlclose(void *handle);\nvoid *dlsym(void *handle, const char *symbol);\nchar *dlerror(void);\n\n\n#if defined(__MINGW32__)\n    #define JIM_SPRINTF_DOUBLE_NEEDS_FIX\n#endif\n\n#ifdef _MSC_VER\n\n\n#if _MSC_VER >= 1000\n\t#pragma warning(disable:4146)\n#endif\n\n#include <limits.h>\n#define jim_wide _int64\n#ifndef HAVE_LONG_LONG\n#define HAVE_LONG_LONG\n#endif\n#ifndef LLONG_MAX\n\t#define LLONG_MAX    9223372036854775807I64\n#endif\n#ifndef LLONG_MIN\n\t#define LLONG_MIN    (-LLONG_MAX - 1I64)\n#endif\n#define JIM_WIDE_MIN LLONG_MIN\n#define JIM_WIDE_MAX LLONG_MAX\n#define JIM_WIDE_MODIFIER \"I64d\"\n#define strcasecmp _stricmp\n#define strtoull _strtoui64\n\n#include <io.h>\n\n#include <winsock.h>\nint gettimeofday(struct timeval *tv, void *unused);\n\n#define HAVE_OPENDIR\nstruct dirent {\n\tchar *d_name;\n};\n\ntypedef struct DIR {\n\tlong                handle;\n\tstruct _finddata_t  info;\n\tstruct dirent       result;\n\tchar                *name;\n} DIR;\n\nDIR *opendir(const char *name);\nint closedir(DIR *dir);\nstruct dirent *readdir(DIR *dir);\n\n#endif\n\n#endif\n\n#ifdef __cplusplus\n}\n#endif\n\n#endif\n#ifndef UTF8_UTIL_H\n#define UTF8_UTIL_H\n\n#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n\n\n#define MAX_UTF8_LEN 4\n\nint utf8_fromunicode(char *p, unsigned uc);\n\n#ifndef JIM_UTF8\n#include <ctype.h>\n\n\n#define utf8_strlen(S, B) ((B) < 0 ? (int)strlen(S) : (B))\n#define utf8_strwidth(S, B) utf8_strlen((S), (B))\n#define utf8_tounicode(S, CP) (*(CP) = (unsigned char)*(S), 1)\n#define utf8_getchars(CP, C) (*(CP) = (C), 1)\n#define utf8_upper(C) toupper(C)\n#define utf8_title(C) toupper(C)\n#define utf8_lower(C) tolower(C)\n#define utf8_index(C, I) (I)\n#define utf8_charlen(C) 1\n#define utf8_prev_len(S, L) 1\n#define utf8_width(C) 1\n\n#else\n\n#endif\n\n#ifdef __cplusplus\n}\n#endif\n\n#endif\n\n#ifndef __JIM__H\n#define __JIM__H\n\n#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n#include <time.h>\n#include <limits.h>\n#include <stdlib.h>\n#include <stdarg.h>\n\n\n#ifndef HAVE_NO_AUTOCONF\n#endif\n\n\n\n#ifndef jim_wide\n#  ifdef HAVE_LONG_LONG\n#    define jim_wide long long\n#    ifndef LLONG_MAX\n#      define LLONG_MAX    9223372036854775807LL\n#    endif\n#    ifndef LLONG_MIN\n#      define LLONG_MIN    (-LLONG_MAX - 1LL)\n#    endif\n#    define JIM_WIDE_MIN LLONG_MIN\n#    define JIM_WIDE_MAX LLONG_MAX\n#  else\n#    define jim_wide long\n#    define JIM_WIDE_MIN LONG_MIN\n#    define JIM_WIDE_MAX LONG_MAX\n#  endif\n\n\n#  ifdef HAVE_LONG_LONG\n#    define JIM_WIDE_MODIFIER \"lld\"\n#  else\n#    define JIM_WIDE_MODIFIER \"ld\"\n#    define strtoull strtoul\n#  endif\n#endif\n\n#define UCHAR(c) ((unsigned char)(c))\n\n\n\n#define JIM_ABI_VERSION 101\n\n#define JIM_OK 0\n#define JIM_ERR 1\n#define JIM_RETURN 2\n#define JIM_BREAK 3\n#define JIM_CONTINUE 4\n#define JIM_SIGNAL 5\n#define JIM_EXIT 6\n\n#define JIM_EVAL 7\n\n#define JIM_MAX_CALLFRAME_DEPTH 1000\n#define JIM_MAX_EVAL_DEPTH 2000\n\n\n#define JIM_PRIV_FLAG_SHIFT 20\n\n#define JIM_NONE 0\n#define JIM_ERRMSG 1\n#define JIM_ENUM_ABBREV 2\n#define JIM_UNSHARED 4\n#define JIM_MUSTEXIST 8\n#define JIM_NORESULT 16\n\n\n#define JIM_SUBST_NOVAR 1\n#define JIM_SUBST_NOCMD 2\n#define JIM_SUBST_NOESC 4\n#define JIM_SUBST_FLAG 128\n\n\n#define JIM_CASESENS    0\n#define JIM_NOCASE      1\n#define JIM_OPT_END     2\n\n\n#define JIM_PATH_LEN 1024\n\n\n#define JIM_NOTUSED(V) ((void) V)\n\n#define JIM_LIBPATH \"auto_path\"\n#define JIM_INTERACTIVE \"tcl_interactive\"\n\n\ntypedef struct Jim_Stack {\n    int len;\n    int maxlen;\n    void **vector;\n} Jim_Stack;\n```"
  codeQuestion1: What is the purpose of this code?
  codeQuestion2: How can a user initiate a new game after losing, and what system calls are involved in handling the input for this action?
  codeQuestion3: What steps are taken to handle terminal input and output settings?
```

I've left all of the corpora inputs in [inputs](./inputs) and all of the completed question/prompt items in [complete](./complete).


### 🧾 Output Structure  
```bash
complete/
├── books/
│   ├── GrammarLogicRhetoricMath/
│   │   └── SQLite_questions.yaml
├── code/
│   ├── SQLite_questions.yaml
└── mdbooks/
    └── Rust_questions.yaml
```

---

## 📎 Configuration (config.yaml)  

```yaml
api_key: your-openai-api-key
api_base: http://127.0.0.1:1234/v1
model: qwen3-4B-mlx
context_max_tokens: 32768
assistant_minimum_context_tokens: 2048
stop_words:
  - |-
    This is a sample text...
session_db_url: /path/to/aj.db
```

### 📁 Template Files

Place these in a directory like `~/Library/Application Support/com.awful-sec.aj/templates/`:  
```bash
templates/book_knowledge_synthesizer.yaml
templates/code_knowledge_synthesizer.yaml
templates/manpage_knowledge_synthesizer.yaml
templates/mdbook_knowledge_synthesizer.yaml
templates/tealdeer_knowledge_synthesizer.yaml
```

---

## 🧠 Supported Source Types  

| Type               | Description                                      |
|-------------------|--------------------------------------------------|
| `yaml`            | Sanitized text chunks (e.g., from books).        |
| `manpage`         | Manpages or system docs (txt files).             |
| `mdbook`          | Nested markdown directories (e.g., `Cargo`, `Rust`). |
| `tealdeer`        | Markdown files (e.g., `AArch64_Assembly.md`).    |
| `code`            | Code snippets (e.g., C, Rust).                   |

---

## 📈 Example Output  

```yaml
- prompt: "What is the purpose of this code?"
  answer: "To implement a database engine..."
```

> *Note: The actual questions depend on the LLM and template used. Use [Awful Jade](https://github.com/awful_aj) to test the results.*  

---

## 🙋🏿‍♂️ Contributing & Feedback  

- **Report bugs**: We welcome all questions ad contributions With Arms Wide Open. It's a Creed really.
- **Suggest improvements**: We were aiming to build a user friendly, simple, fast CLI but if you are having \~big ideas\~ that require simple solutions, holler.  
- **Share your data**: Both with us and in general. Here's are the Open Source datasets built using this tool: https://huggingface.co/dougiefresh/datasets

---

## 🤔 Why Use This?  

- **No code changes**: Just run it and let the LLM handle the heavy lifting.  
- **Customizable**: Choose between `code`, `manpage`, `mdbook`, `book`, `tealdeer`, or `yaml` sources.  
- **Persistent converstaions (Optional)**: Use a sqlite database to store LLM responses with `config.yaml`.  

---

## 🫩 Final Thoughts  

**`awful_knowledge_synthesizer`** is a begrudging love letter to the power of LLMs and the chaos of text. Whether you're studying for finals, building interesting NPCs, or simply trying to f*ck your computer, this tool is your dutiful assistant.  

> *Remember: The goal isn’t to make every question perfect. It’s to ensure our knowledge *is used to refine*.*  

--- 

### 🧐 Want to Try It?  

1. **Install dependencies**:  
   ```bash
   cargo install awful_knowledge_synthesizer
   ```

2. **Run it**:  
   ```bash
   awful_knowledge_synthesizer --help
   ```

3. **Explore the examples**:  
   ```bash
   tree inputs
   tree complete
   ```

> *Now go forth and synthesize!* 🧠📚
