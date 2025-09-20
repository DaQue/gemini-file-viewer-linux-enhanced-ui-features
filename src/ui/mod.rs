pub(crate) mod toolbar;
pub(crate) mod search_bar;
pub(crate) mod status;
pub(crate) mod tab_strip;
pub(crate) mod windows;

// Re-exports to preserve crate::ui::{fn} paths
pub(crate) use toolbar::toolbar;
pub(crate) use search_bar::search_bar;
pub(crate) use status::{status_bar, status_extra};
pub(crate) use tab_strip::tab_strip;
pub(crate) use windows::{recent_files_window, global_search_window, settings_window};

