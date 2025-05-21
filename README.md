# Awful Knowledge Synthesizer

Turn books that were parsed into chunks and placed in YAML files into exam questions meant to prompt an LLM for question/answer pairs.

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

`awful_knowledge_synthesizer` can work with books (yaml), manpages (txt), mdbook (nested directories of markdown), tealdeer (markdown).

The `yaml` file inputs for the `--source-type books` option are expected to conform to the following schema:

```yaml
chunks:
  - Some sanitized text
  - From a book
  - Or any corpus really
```

All of the example inputs are XNU related. We would like to fine tune a model from basic grammar to calculus and trigonometry, followed by the systems programming building blocks I prefer. These are aarch64 assembly, C, Rust, and Nushell. Someday I pray these will be the only dependencies my system is built on. We follow that up with userland MacOS, iOS, *OS knowledge.

Hopefully at the end of this we'll have a knowledgable, useful, subject matter expert, that lives on the device it knows deeply. Like a Clippy that doesn't suck (Microsoft Word's animated atrocity, not Rust's incredibly useful linter).

An iOS app to run the resulting model, [**Jade** is already available](https://github.com/graves/Jade).
A command line interface to OpenAI-compatible APIs, for running the resulting model (with a built in vector database for persistent memories), [**Awful Jade** is already available](https://github.com/awful_aj)

## Configuration

Under the hood, this tool uses [aj](https://github.com/graves/aj) to send the prompts for inference. The configuration requires a `config.yaml` compatible with `aj` and `*_knowledge_synthsizer.yaml` files for each source type placed in the configuration directory `aj` expects.

```sh
λ ls template/
╭───┬──────────────────────────────────────────────┬──────┬────────┬────────────╮
│ # │                     name                     │ type │  size  │  modified  │
├───┼──────────────────────────────────────────────┼──────┼────────┼────────────┤
│ 0 │ template/book_knowledge_synthesizer.yaml     │ file │ 7.4 kB │ a week ago │
│ 1 │ template/code_knowledge_synthesizer.yaml     │ file │ 5.9 kB │ 2 days ago │
│ 2 │ template/manpage_knowledge_synthesizer.yaml  │ file │ 2.9 kB │ 4 days ago │
│ 3 │ template/mdbook_knowledge_synthesizer.yaml   │ file │ 2.5 kB │ 3 days ago │
│ 4 │ template/tealdeer_knowledge_synthesizer.yaml │ file │ 2.5 kB │ 2 days ago │
╰───┴──────────────────────────────────────────────┴──────┴────────┴────────────╯
```

```sh
λ cp template/* `~/Library/Application Support/com.awful-sec.aj/templates/`
```

```sh
λ cat config/config.yaml
api_key:
api_base: http://127.0.0.1:1234/v1
model: qwen3-4B-mlx
context_max_tokens: 32768
assistant_minimum_context_tokens: 2048
stop_words:
- |2-

  <|im_start|>
- <|im_end|>
session_db_url: "/Users/tg/Library/Application Support/com.awful-sec.aj/aj.db"
session_name: None
```

## Usage

```sh
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

```sh
λ awful_knowledge_synthesizer --input-dir inputs/code/sqlite --config config.yaml --source-type code --language c --output-dir . --project-name "SQLite"
Reading "inputs/code/sqlite"
File: jimsh0.c

Processing chunk 1/116
Wrote to ./SQLite_questions.yaml
Processing chunk 2/116
```

`SQLite_questions.yaml`:

```yaml
- prompt: "You are playing the role of a senior software engineer developing questions for a code review. Here is some source code from inputs/code/sqlite/autosetup/jimsh0.c. It is part of the SQLite project.\n\n\n\nSource Code:\n\n```c\n/* This is single source file, bootstrap version of Jim Tcl. See http://jim.tcl.tk/ */\n#define JIM_COMPAT\n#define JIM_ANSIC\n#define JIM_REGEXP\n#define HAVE_NO_AUTOCONF\n#define JIM_TINY\n#define _JIMAUTOCONF_H\n#define TCL_LIBRARY \".\"\n#define jim_ext_bootstrap\n#define jim_ext_aio\n#define jim_ext_readdir\n#define jim_ext_regexp\n#define jim_ext_file\n#define jim_ext_glob\n#define jim_ext_exec\n#define jim_ext_clock\n#define jim_ext_array\n#define jim_ext_stdlib\n#define jim_ext_tclcompat\n#if defined(_MSC_VER)\n#define TCL_PLATFORM_OS \"windows\"\n#define TCL_PLATFORM_PLATFORM \"windows\"\n#define TCL_PLATFORM_PATH_SEPARATOR \";\"\n#define HAVE_MKDIR_ONE_ARG\n#define HAVE_SYSTEM\n#elif defined(__MINGW32__)\n#define TCL_PLATFORM_OS \"mingw\"\n#define TCL_PLATFORM_PLATFORM \"windows\"\n#define TCL_PLATFORM_PATH_SEPARATOR \";\"\n#define HAVE_MKDIR_ONE_ARG\n#define HAVE_SYSTEM\n#define HAVE_SYS_TIME_H\n#define HAVE_DIRENT_H\n#define HAVE_UNISTD_H\n#define HAVE_UMASK\n#include <sys/stat.h>\n#ifndef S_IRWXG\n#define S_IRWXG 0\n#endif\n#ifndef S_IRWXO\n#define S_IRWXO 0\n#endif\n#else\n#define TCL_PLATFORM_OS \"unknown\"\n#define TCL_PLATFORM_PLATFORM \"unix\"\n#define TCL_PLATFORM_PATH_SEPARATOR \":\"\n#ifdef _MINIX\n#define vfork fork\n#define _POSIX_SOURCE\n#else\n#define _GNU_SOURCE\n#endif\n#define HAVE_FORK\n#define HAVE_WAITPID\n#define HAVE_ISATTY\n#define HAVE_MKSTEMP\n#define HAVE_LINK\n#define HAVE_SYS_TIME_H\n#define HAVE_DIRENT_H\n#define HAVE_UNISTD_H\n#define HAVE_UMASK\n#define HAVE_PIPE\n#define _FILE_OFFSET_BITS 64\n#endif\n#define JIM_VERSION 84\n#ifndef JIM_WIN32COMPAT_H\n#define JIM_WIN32COMPAT_H\n\n\n\n#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n\n#if defined(_WIN32) || defined(WIN32)\n\n#define HAVE_DLOPEN\nvoid *dlopen(const char *path, int mode);\nint dlclose(void *handle);\nvoid *dlsym(void *handle, const char *symbol);\nchar *dlerror(void);\n\n\n#if defined(__MINGW32__)\n    #define JIM_SPRINTF_DOUBLE_NEEDS_FIX\n#endif\n\n#ifdef _MSC_VER\n\n\n#if _MSC_VER >= 1000\n\t#pragma warning(disable:4146)\n#endif\n\n#include <limits.h>\n#define jim_wide _int64\n#ifndef HAVE_LONG_LONG\n#define HAVE_LONG_LONG\n#endif\n#ifndef LLONG_MAX\n\t#define LLONG_MAX    9223372036854775807I64\n#endif\n#ifndef LLONG_MIN\n\t#define LLONG_MIN    (-LLONG_MAX - 1I64)\n#endif\n#define JIM_WIDE_MIN LLONG_MIN\n#define JIM_WIDE_MAX LLONG_MAX\n#define JIM_WIDE_MODIFIER \"I64d\"\n#define strcasecmp _stricmp\n#define strtoull _strtoui64\n\n#include <io.h>\n\n#include <winsock.h>\nint gettimeofday(struct timeval *tv, void *unused);\n\n#define HAVE_OPENDIR\nstruct dirent {\n\tchar *d_name;\n};\n\ntypedef struct DIR {\n\tlong                handle;\n\tstruct _finddata_t  info;\n\tstruct dirent       result;\n\tchar                *name;\n} DIR;\n\nDIR *opendir(const char *name);\nint closedir(DIR *dir);\nstruct dirent *readdir(DIR *dir);\n\n#endif\n\n#endif\n\n#ifdef __cplusplus\n}\n#endif\n\n#endif\n#ifndef UTF8_UTIL_H\n#define UTF8_UTIL_H\n\n#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n\n\n#define MAX_UTF8_LEN 4\n\nint utf8_fromunicode(char *p, unsigned uc);\n\n#ifndef JIM_UTF8\n#include <ctype.h>\n\n\n#define utf8_strlen(S, B) ((B) < 0 ? (int)strlen(S) : (B))\n#define utf8_strwidth(S, B) utf8_strlen((S), (B))\n#define utf8_tounicode(S, CP) (*(CP) = (unsigned char)*(S), 1)\n#define utf8_getchars(CP, C) (*(CP) = (C), 1)\n#define utf8_upper(C) toupper(C)\n#define utf8_title(C) toupper(C)\n#define utf8_lower(C) tolower(C)\n#define utf8_index(C, I) (I)\n#define utf8_charlen(C) 1\n#define utf8_prev_len(S, L) 1\n#define utf8_width(C) 1\n\n#else\n\n#endif\n\n#ifdef __cplusplus\n}\n#endif\n\n#endif\n\n#ifndef __JIM__H\n#define __JIM__H\n\n#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n#include <time.h>\n#include <limits.h>\n#include <stdlib.h>\n#include <stdarg.h>\n\n\n#ifndef HAVE_NO_AUTOCONF\n#endif\n\n\n\n#ifndef jim_wide\n#  ifdef HAVE_LONG_LONG\n#    define jim_wide long long\n#    ifndef LLONG_MAX\n#      define LLONG_MAX    9223372036854775807LL\n#    endif\n#    ifndef LLONG_MIN\n#      define LLONG_MIN    (-LLONG_MAX - 1LL)\n#    endif\n#    define JIM_WIDE_MIN LLONG_MIN\n#    define JIM_WIDE_MAX LLONG_MAX\n#  else\n#    define jim_wide long\n#    define JIM_WIDE_MIN LONG_MIN\n#    define JIM_WIDE_MAX LONG_MAX\n#  endif\n\n\n#  ifdef HAVE_LONG_LONG\n#    define JIM_WIDE_MODIFIER \"lld\"\n#  else\n#    define JIM_WIDE_MODIFIER \"ld\"\n#    define strtoull strtoul\n#  endif\n#endif\n\n#define UCHAR(c) ((unsigned char)(c))\n\n\n\n#define JIM_ABI_VERSION 101\n\n#define JIM_OK 0\n#define JIM_ERR 1\n#define JIM_RETURN 2\n#define JIM_BREAK 3\n#define JIM_CONTINUE 4\n#define JIM_SIGNAL 5\n#define JIM_EXIT 6\n\n#define JIM_EVAL 7\n\n#define JIM_MAX_CALLFRAME_DEPTH 1000\n#define JIM_MAX_EVAL_DEPTH 2000\n\n\n#define JIM_PRIV_FLAG_SHIFT 20\n\n#define JIM_NONE 0\n#define JIM_ERRMSG 1\n#define JIM_ENUM_ABBREV 2\n#define JIM_UNSHARED 4\n#define JIM_MUSTEXIST 8\n#define JIM_NORESULT 16\n\n\n#define JIM_SUBST_NOVAR 1\n#define JIM_SUBST_NOCMD 2\n#define JIM_SUBST_NOESC 4\n#define JIM_SUBST_FLAG 128\n\n\n#define JIM_CASESENS    0\n#define JIM_NOCASE      1\n#define JIM_OPT_END     2\n\n\n#define JIM_PATH_LEN 1024\n\n\n#define JIM_NOTUSED(V) ((void) V)\n\n#define JIM_LIBPATH \"auto_path\"\n#define JIM_INTERACTIVE \"tcl_interactive\"\n\n\ntypedef struct Jim_Stack {\n    int len;\n    int maxlen;\n    void **vector;\n} Jim_Stack;\n```"
  codeQuestion1: What is the purpose of this code?
  codeQuestion2: How can a user initiate a new game after losing, and what system calls are involved in handling the input for this action?
  codeQuestion3: What steps are taken to handle terminal input and output settings?
```

I've left all of the corpora inputs in [inputs](./inputs) and all of the completed question/prompt items in [complete](./complete).

### Example Inputs

```sh
λ tree inputs
inputs
├── books
│   ├── GrammarLogicRhetoricMath
│   │   ├── Adler, Mortimer J. & Gorman, William | A Syntopicon of Great Books of the Western World.pdf.txt.yaml
│   │   ├── Barbara Oakley | A Mind for Numbers.pdf.txt.yaml
│   │   ├── Charles P. McKeague, Mark D. Turner | Trigonometry.pdf.txt.yaml
│   │   ├── Corbett, Edward P. J. | Classical rhetoric for the modern student-Oxford University Press.pdf.txt.yaml
│   │   ├── Irving M. Copi, Carl Cohen, Victor Rodych | Introduction to Logic.pdf.txt.yaml
│   │   ├── Jeremy Kun | A Programmer’s Introduction to Mathematics.pdf.txt.yaml
│   │   ├── Michael Spivak | Calculus.pdf.txt.yaml
│   │   ├── Mortimer J. Adler, Charles Van Doren | How to Read a Book.pdf.txt.yaml
│   │   ├── Neuronwaves | Critical Thinking Logic and Problem Solving.pdf.txt.yaml
│   │   ├── Robert F. Blitzer | Algebra and Trigonometry.pdf.txt.yaml
│   │   ├── Sidney Greenbaum, Gerald Nelson | An Introduction to English Grammar-Longman.pdf.txt.yaml
│   │   ├── Sister Miriam Joseph, Marguerite McGlinn | The Trivium The Liberal Arts of Logic, Grammar, and Rhetoric.pdf.txt.yaml
│   │   ├── Stephen E. Nadeau | The Neural Architecture of Grammar.pdf.txt.yaml
│   │   └── Thomas S. Kane | The New Oxford Guide to Writing.pdf.txt.yaml
│   ├── SystemsProgramming
│   │   ├── Brian Kernighan and Dennis Ritchie | The C Programming Language.txt.yaml
│   │   ├── Kodeco Team, Walter Tyree | Advanced Apple Debugging & Reverse Engineering (Fourth Edition) Exploring Apple Code Through LLDB, Python & DTrace.txt.yaml
│   │   ├── Sagar Rastogi, Jasdeep Singh | Exploring macOS A Journey Through the Mac Ecosystem.txt.yaml
│   │   └── Sufyan bin Uzayr  | Objective-C The Ultimate Guide.txt.yaml
│   └── remove_ctrl_chars.nu
├── code
│   ├── AArch64_Assembly
│   │   ...
|   |   ...
│   ├── reqwest
|   |   ...
|   |   ...
│   ├── ripgrep
|   |   ...
|   |   ...
│   └── sqlite
|   |   ...
|   |   ...
├── manpages
│   ├── 4ccconv.txt
    ...
    ...
│   └── zshzle.txt
└── mdbooks
    ├── cargo
    |   ...
    |   ...
    ├── helix
    |   ...
    |   ...
    ├── nushell
    |   ...
    |   ...
    └── rust
    |   ...
    |   ...
└── tealdeer
    ├── aa.md
    ...
    ...
    └── yabai.md

164 directories, 6307 files
```

### Example Outputs

```sh
λ tree complete
complete
├── books
│   ├── GrammarLogicRhetoricMath
│   │   ├── A Mind for Numbers_questions.yaml
│   │   ├── A Programmer’s Introduction to Mathematics_questions.yaml
│   │   ├── Algebra and Trigonometry_questions.yaml
│   │   ├── An Introduction to English Grammar-Longman_questions.yaml
│   │   ├── Calculus_questions.yaml
│   │   ├── Classical rhetoric for the modern student-Oxford University Press_questions.yaml
│   │   ├── Critical Thinking Logic and Problem Solving_questions.yaml
│   │   ├── How to Read a Book_questions.yaml
│   │   ├── Introduction to Logic_questions.yaml
│   │   ├── The Neural Architecture of Grammar_questions.yaml
│   │   ├── The New Oxford Guide to Writing_questions.yaml
│   │   ├── The Trivium The Liberal Arts of Logic, Grammar, and Rhetoric_questions.yaml
│   │   └── Trigonometry_questions.yaml
│   └── SystemsProgramming
│       ├── Advanced Apple Debugging & Reverse Engineering (Fourth Edition) Exploring Apple Code Through LLDB, Python & DTrace_questions.yaml
│       ├── Exploring macOS A Journey Through the Mac Ecosystem_questions.yaml
│       ├── Foundations of ARM64 Linux Debugging, Disassembling, and Reversing Analyze Code, Understand Stack Memory Usage, and Reconstruct Original C_C++ Code with ARM64_questions.yaml
│       ├── Objective-C The Ultimate Guide_questions.yaml
│       ├── The C Programming Language_questions.yaml
│       └── The Rust Programming Language_questions.yaml
├── code
│   ├── AArch64_Assembly Algorithms_questions.yaml
│   ├── Hyper_questions.yaml
│   ├── Ripgrep_questions.yaml
│   └── SQLite_questions.yaml
├── manpages
│   ├── 4ccconv_questions.yaml
    | ...
    | ...
│   └── zshzle_questions.yaml
├── mdbooks
│   ├── Cargo_questions.yaml
│   ├── Helix_questions.yaml
│   ├── Nushell_questions.yaml
│   └── Rust_questions.yaml
└── tealdeer
    └── Tealdeer_questions.yaml

8 directories, 2734 files
