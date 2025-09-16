# TODO

- [x] Integrate syntect-based syntax highlighting
  - Wire `syntect` for tokenization/coloring; keep current lightweight highlighter as fallback for very large files
  - Load syntaxes/themes (dump-load) at startup; map to existing `CodeTheme`
  - Gate by file size and extension; add a setting to toggle syntect on/off
  - Remove bespoke highlighter paths once parity is verified
  - Status: syntect session integrated; uses defaults; toggle in Settings; remove “beta” label

- [x] Multi-tab UI for text and images
  - Render a tab strip; show file name and dirty state (future)
  - Open subsequent files in new tabs; click to switch; middle-click/shortcut to close
  - Persist and restore open tabs and active tab via settings (opt-in)
  - Unified strip for text and image tabs; image tabs tracked by path
  - One-shot "Reopen Session" action (toolbar and Recent window)
  - Keyboard shortcuts: Next/Prev Tab, Close Tab, Reopen Closed Tab
  - Status: text+image tab strip with close buttons; session restore (opt-in) implemented; one-shot reopen added

- [ ] Clippy: remove remaining allows and refactor highlight module
  - Introduce a `HighlightContext` struct to reduce argument counts
  - Split `append_highlighted`/`token_highlight` into smaller helpers
  - Replace nested conditionals with clearer flows; remove identical branches
  - Convert iterator `while let` loops to `for` where appropriate
  - Status: comment formatting deduped; `#[allow]` removed in highlight.rs; clippy nits remain in ui/app

- [ ] Global Search improvements
  - Optional regex mode; toggleable case and whole-word already exist
  - Virtualize result list for very large outputs; show per-file counts and grouping
  - Option to search disk (folder) in addition to open tabs
  - Status: Regex mode implemented (with error display; whole-word disabled in regex). Virtualization and folder search pending

- [ ] File open UX
  - Drag-and-drop files onto window to open (adds background text tabs; tracks image tabs)
  - Keep non-blocking open; indicate when a dialog is in-flight (spinner). Toolbar button is blocking; Ctrl+O uses non-blocking

- [ ] UI/UX polish
  - Responsive toolbar layout; configurable placement of Global Search/Recent
  - Compact Recent list items with icons; keyboard navigation
  - Show file type icon and size in status bar for text files

- [x] Window sizing and persistence
  - Enforce a minimum width to fit all toolbar buttons
  - Restore last window size on startup; don't go below minimum
  - Status: min size enforced; last size fields stored; applied on startup

- [x] About dialog content
  - Include license: free to use with no warranty of usability or responsibility
  - List authors: David Queen, Allison Bayless
  - Status: text added in Settings → About

- [x] Keybindings window
  - Add button inside Settings to show keybindings; ensure minimum width but resizable
  - Add keyboard shortcuts: Ctrl+, for Settings; F1 for About
  - Status: expanded to include Alt variants

- [ ] Performance
  - Background loading for very large text files; incremental rendering
  - Cache line layouts for faster scrolling on huge files

- [ ] Packaging and versioning
  - Bump crate version; align window title and Cargo.toml
  - Add Linux/AppImage packaging script, Windows icon/RC, macOS bundle metadata
  - Status: Windows EXE icon embedding via build.rs; Linux .desktop + icon wiring documented and installable

- [ ] Testing
  - Unit tests for `search::find_target_line` and count recomputation
  - Integration tests for tab switching and global search navigation
  - Benchmarks for highlighter performance on large files

- [x] Temporary app icon
  - Add a placeholder app icon for window/taskbar and launcher entries
  - Linux: `.desktop` file + PNG under `~/.local/share/icons`, set window icon via eframe options
  - Status: procedural placeholder set at runtime; PNG exported at `assets/icon_256.png`; launcher wired; Windows EXE embeds icon on build
