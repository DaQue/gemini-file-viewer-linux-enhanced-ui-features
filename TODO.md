# TODO

- [ ] Integrate syntect-based syntax highlighting
  - Wire `syntect` for tokenization/coloring; keep current lightweight highlighter as fallback for very large files
  - Load syntaxes/themes (dump-load) at startup; map to existing `CodeTheme`
  - Gate by file size and extension; add a setting to toggle syntect on/off
  - Remove bespoke highlighter paths once parity is verified

- [ ] Multi-tab UI for text and images
  - Render a tab strip; show file name and dirty state (future)
  - Open subsequent files in new tabs; click to switch; middle-click/shortcut to close
  - Persist and restore open tabs and active tab via settings
  - Keyboard shortcuts: Next/Prev Tab, Close Tab, Reopen Closed Tab

- [ ] Clippy: remove remaining allows and refactor highlight module
  - Introduce a `HighlightContext` struct to reduce argument counts
  - Split `append_highlighted`/`token_highlight` into smaller helpers
  - Replace nested conditionals with clearer flows; remove identical branches
  - Convert iterator `while let` loops to `for` where appropriate

- [ ] Global Search improvements
  - Optional regex mode; toggleable case and whole-word already exist
  - Virtualize result list for very large outputs; show per-file counts and grouping
  - Option to search disk (folder) in addition to open tabs

- [ ] File open UX
  - Drag-and-drop files onto window to open (and into new tab)
  - Keep non-blocking open; indicate when a dialog is in-flight

- [ ] UI/UX polish
  - Responsive toolbar layout; configurable placement of Global Search/Recent
  - Compact Recent list items with icons; keyboard navigation
  - Show file type icon and size in status bar for text files

- [ ] Performance
  - Background loading for very large text files; incremental rendering
  - Cache line layouts for faster scrolling on huge files

- [ ] Packaging and versioning
  - Bump crate version; align window title and Cargo.toml
  - Add Linux/AppImage packaging script, Windows icon/RC, macOS bundle metadata

- [ ] Testing
  - Unit tests for `search::find_target_line` and count recomputation
  - Integration tests for tab switching and global search navigation
  - Benchmarks for highlighter performance on large files
