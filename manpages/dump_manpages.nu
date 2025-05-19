#!/opt/homebrew/bin/nu

# Nushell script to extract filenames and run `man <command> | col -b | save <command>.txt`

let man_dirs = [
  "/usr/share/man/man1",
  "/usr/share/man/man4",
  "/usr/share/man/man5",
  "/usr/share/man/man6",
  "/usr/share/man/man7",
  "/usr/share/man/man8",
  "/usr/share/man/man9",
  "/usr/share/man/mann"
]

for dir in $man_dirs {
  ls $dir
  | where type == "file"
  | each { |file|
      let full_path = $file.name
      let base = ($full_path | path basename)
      let command = ($base | split row "." | get 0)
      man $command | col -b | save -f $"($command).txt"
  }
}
