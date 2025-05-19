for entry in (ls) {
  let fname = $entry.name | split column "." | get column1.0 | str replace "_questions" ""
  mv $"/Users/tg/Training Docs/macOS/manpages/($fname).txt" $"/Users/tg/Training Docs/macOS/manpages/($fname).bak"
}
