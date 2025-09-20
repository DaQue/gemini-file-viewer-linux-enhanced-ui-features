# TODO

## ✅ COMPLETED FEATURES

- [x] **Syntect-based syntax highlighting** - FULLY IMPLEMENTED
  - ✅ `syntect` integrated with tokenization/coloring
  - ✅ Fallback to lightweight highlighter for large files
  - ✅ Syntax themes loaded at startup; mapped to `CodeTheme`
  - ✅ File size gating and toggle in Settings
  - ✅ Toggle setting (`use_syntect`) available in UI

- [x] **Multi-tab UI for text and images** - FULLY IMPLEMENTED
  - ✅ Tab strip with file names and close buttons
  - ✅ Click to switch tabs; close buttons work
  - ✅ Session restore functionality (opt-in via settings)
  - ✅ Unified strip for text and image tabs
  - ✅ "Reopen Session" button in toolbar
  - ✅ Text and image tabs tracked separately

- [x] **Window sizing and persistence** - FULLY IMPLEMENTED
  - ✅ Minimum width enforced to fit toolbar buttons
  - ✅ Last window size restored on startup
  - ✅ Size persistence across app restarts

- [x] **About dialog content** - FULLY IMPLEMENTED
  - ✅ License text included
  - ✅ Authors listed: David Queen, Allison Bayless
  - ✅ Comprehensive keyboard shortcuts displayed
  - ✅ Rich formatting with icons and sections

- [x] **Keybindings window** - FULLY IMPLEMENTED
  - ✅ Settings button opens keybindings window
  - ✅ Comprehensive keyboard shortcuts listed
  - ✅ Alt variants included for all shortcuts
  - ✅ Resizable window with minimum width

- [x] **App icon** - FULLY IMPLEMENTED
  - ✅ Procedural icon generation at runtime
  - ✅ PNG assets in `assets/icons/` directory
  - ✅ Windows EXE icon embedding via `build.rs`
  - ✅ Linux `.desktop` file integration

- [x] **Drag-and-drop file opening** - FULLY IMPLEMENTED
  - ✅ Drag files onto window to open
  - ✅ Supports both text and image files
  - ✅ Adds text files as background tabs
  - ✅ Tracks image tabs without switching
  - ✅ File type validation and error handling

## 🔄 IN PROGRESS / PARTIAL

- [ ] **Global Search improvements** - PARTIALLY IMPLEMENTED
  - ✅ Regex mode with error display
  - ✅ Case sensitive and whole word options
  - ✅ Global search window with results
  - ❌ Virtualize result list for large outputs
  - ❌ Per-file counts and grouping
  - ❌ Option to search disk (folder) in addition to open tabs

## ❌ PENDING FEATURES

- [ ] **Code cleanup and optimization**
  - [x] Remove remaining clippy warnings
  - [x] Refactor highlight module for better maintainability
  - [x] Introduce `HighlightContext` struct to reduce argument counts
  - [x] Split large functions into smaller helpers

- [ ] **UI/UX polish**
  - Responsive toolbar layout
  - Configurable placement of Global Search/Recent buttons
  - Compact Recent list items with icons
  - Keyboard navigation for Recent files
  - Show file type icon and size in status bar for text files

- [ ] **Performance improvements**
  - Background loading for very large text files
  - Incremental rendering for huge files
  - Cache line layouts for faster scrolling
  - Optimize memory usage for large files

- [ ] **Enhanced packaging**
  - Linux AppImage packaging script
  - macOS bundle metadata
  - Automated release packaging
  - Cross-platform build scripts

- [ ] **Testing infrastructure**
  - [x] Unit tests for `search::find_target_line` and count recomputation
  - Integration tests for tab switching and global search navigation
  - Benchmarks for highlighter performance on large files
  - UI automation tests

## 🆕 NEW FEATURES TO CONSIDER

- [ ] **Enhanced file navigation**
  - File tree/sidebar for directory browsing
  - Recent directories alongside recent files
  - Bookmark favorite files/folders

- [ ] **Advanced search capabilities**
  - Search within specific file types
  - Search history and saved searches
  - Advanced regex patterns and filters

- [ ] **Editor-like features**
  - Basic text editing capabilities
  - Find and replace functionality
  - Line number jumping (Go to line)
  - Text selection and copying

- [ ] **Accessibility improvements**
  - High contrast themes
  - Font size scaling
  - Keyboard-only navigation
  - Screen reader support
