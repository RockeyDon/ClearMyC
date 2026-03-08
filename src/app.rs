use crate::data::{AppData, FolderEntry};
use crate::file_ops::{DeleteResult, calculate_dir_size, move_to_recycle_bin, permanently_delete};
use crate::i18n::{self, Locale, Texts};
use crate::llm;

use iced::{Element, Task, Theme};

// ─── Types shared across UI modules ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigTab {
    Folder,
    Strategy,
    Theme,
    About,
}

#[derive(Debug, Clone)]
pub struct ConfigFolderRow {
    pub path: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Main page
    ToggleSelected(usize),
    Refresh,
    ScanComplete(Vec<Option<u64>>),
    DeleteClicked,
    PermanentDeleteClicked,
    OpenConfig,

    // Delete confirm dialog
    ConfirmDelete,
    CancelDelete,
    DeleteDone(Vec<Option<i8>>),

    // Permanent delete confirm dialog
    ConfirmPermanentDelete,
    CancelPermanentDelete,
    PermanentDeleteDone(Vec<Option<i8>>),

    // Config page
    ConfigTabChanged(ConfigTab),
    ConfigFolderPathChanged(usize, String),
    ConfigFolderDescChanged(usize, String),
    AddFolderRow,
    RemoveFolderRow(usize),
    ConfigThemeChanged(String),
    ThemeApplied,
    CloseConfig,

    // Strategy config
    StrategyBaseUrlChanged(String),
    StrategyModelChanged(String),
    StrategyApiKeyChanged(String),
    StrategySkipListChanged(String),
    StrategyMinSizeChanged(String),
    StrategySkipRecentChanged(String),

    // Connect test
    ConnectClicked,
    ConnectResult(String),
    CloseConnectDialog,

    // Auto Discovery
    AutoDiscoverClicked,
    AutoDiscoverResult(String),
    CloseAutoDiscoverDialog,

    // Language
    LanguageChanged(String),

    // Clear all folders
    ClearFoldersClicked,
    ConfirmClearFolders,
    CancelClearFolders,
}

// ─── Application State ──────────────────────────────────────────────

pub struct App {
    pub data: AppData,
    pub folder_sizes: Vec<Option<u64>>,
    pub folder_statuses: Vec<Option<i8>>,
    pub show_config: bool,
    pub config_tab: ConfigTab,
    pub config_folders: Vec<ConfigFolderRow>,
    pub config_theme: String,
    pub show_delete_confirm: bool,
    pub show_permanent_confirm: bool,
    pub scanning: bool,

    // Strategy config fields
    pub strategy_base_url: String,
    pub strategy_model: String,
    pub strategy_api_key: String,
    pub strategy_skip_list: String,
    pub strategy_min_size: String,
    pub strategy_skip_recent: String,

    // Dialog states
    pub show_connect_dialog: bool,
    pub connect_dialog_text: String,
    pub show_auto_discover_dialog: bool,
    pub auto_discover_dialog_text: String,

    // i18n
    pub locale: Locale,

    // Clear confirm
    pub show_clear_confirm: bool,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let data = AppData::load();
        let folder_count = data.folders.len();
        let llm = data.read_llm_config();
        let strategy = data.read_strategy_config();
        let locale = Locale::from_str(&data.settings.language);

        let app = Self {
            config_folders: data
                .folders
                .iter()
                .map(|f| ConfigFolderRow {
                    path: f.path.clone(),
                    description: f.description.clone(),
                })
                .collect(),
            config_theme: data.settings.theme.clone(),
            strategy_base_url: llm.base_url,
            strategy_model: llm.model,
            strategy_api_key: llm.api_key,
            strategy_skip_list: strategy.skip_list,
            strategy_min_size: format_mb_to_display(strategy.min_size),
            strategy_skip_recent: strategy.skip_recent.to_string(),
            data,
            folder_sizes: vec![None; folder_count],
            folder_statuses: vec![None; folder_count],
            show_config: false,
            config_tab: ConfigTab::Folder,
            show_delete_confirm: false,
            show_permanent_confirm: false,
            scanning: false,
            show_connect_dialog: false,
            connect_dialog_text: String::new(),
            show_auto_discover_dialog: false,
            auto_discover_dialog_text: String::new(),
            locale,
            show_clear_confirm: false,
        };

        let folders: Vec<String> = app.data.folders.iter().map(|f| f.path.clone()).collect();
        (app, Task::perform(scan_folders(folders), Message::ScanComplete))
    }

    pub fn theme(&self) -> Theme {
        match self.data.settings.theme.as_str() {
            "dark" => Theme::Dark,
            "system" => {
                if detect_system_dark_mode() {
                    Theme::Dark
                } else {
                    Theme::Light
                }
            }
            _ => Theme::Light,
        }
    }

    /// Return the current locale's text table.
    pub fn t(&self) -> &'static Texts {
        i18n::texts(self.locale)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let t = self.t();
        if self.show_config {
            // Show connect/auto-discover/clear-confirm dialogs on top of config
            if self.show_connect_dialog {
                return crate::ui::dialogs::view_connect_dialog(self);
            }
            if self.show_auto_discover_dialog {
                return crate::ui::dialogs::view_auto_discover_dialog(self);
            }
            if self.show_clear_confirm {
                return crate::ui::dialogs::view_clear_confirm(self);
            }
            return crate::ui::config_view::view(self);
        }
        let theme = self.theme();
        let tc = crate::ui::style::text_color(&theme);
        let mc = crate::ui::style::text_muted_color(&theme);
        if self.show_delete_confirm {
            return crate::ui::dialogs::view_delete_confirm(t, tc, mc);
        }
        if self.show_permanent_confirm {
            return crate::ui::dialogs::view_permanent_confirm(t, tc, mc);
        }
        crate::ui::main_view::view(self)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleSelected(index) => {
                self.data.toggle_selected(index);
                Task::none()
            }
            Message::Refresh => {
                self.folder_statuses = vec![None; self.data.folders.len()];
                self.scanning = true;
                let folders: Vec<String> =
                    self.data.folders.iter().map(|f| f.path.clone()).collect();
                Task::perform(scan_folders(folders), Message::ScanComplete)
            }
            Message::ScanComplete(sizes) => {
                self.folder_sizes = sizes;
                self.scanning = false;
                Task::none()
            }
            Message::DeleteClicked => {
                self.show_delete_confirm = true;
                Task::none()
            }
            Message::PermanentDeleteClicked => {
                self.show_permanent_confirm = true;
                Task::none()
            }
            Message::OpenConfig => {
                self.config_folders = self
                    .data
                    .folders
                    .iter()
                    .map(|f| ConfigFolderRow {
                        path: f.path.clone(),
                        description: f.description.clone(),
                    })
                    .collect();
                self.config_theme = self.data.settings.theme.clone();

                // Load LLM config (decrypted)
                let llm = self.data.read_llm_config();
                self.strategy_base_url = llm.base_url;
                self.strategy_model = llm.model;
                self.strategy_api_key = llm.api_key;

                // Load strategy config
                let strategy = self.data.read_strategy_config();
                self.strategy_skip_list = strategy.skip_list;
                self.strategy_min_size = format_mb_to_display(strategy.min_size);
                self.strategy_skip_recent = strategy.skip_recent.to_string();

                self.show_config = true;
                Task::none()
            }
            Message::ConfirmDelete => {
                self.show_delete_confirm = false;
                let folders: Vec<(usize, String)> = self
                    .data
                    .folders
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.selected)
                    .map(|(i, f)| (i, f.path.clone()))
                    .collect();
                let total = self.data.folders.len();
                Task::perform(do_delete(folders, total, false), Message::DeleteDone)
            }
            Message::CancelDelete => {
                self.show_delete_confirm = false;
                Task::none()
            }
            Message::DeleteDone(results) => {
                self.folder_statuses = results;
                let folders: Vec<String> =
                    self.data.folders.iter().map(|f| f.path.clone()).collect();
                Task::perform(scan_folders(folders), Message::ScanComplete)
            }
            Message::ConfirmPermanentDelete => {
                self.show_permanent_confirm = false;
                let folders: Vec<(usize, String)> = self
                    .data
                    .folders
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.selected)
                    .map(|(i, f)| (i, f.path.clone()))
                    .collect();
                let total = self.data.folders.len();
                Task::perform(do_delete(folders, total, true), Message::PermanentDeleteDone)
            }
            Message::CancelPermanentDelete => {
                self.show_permanent_confirm = false;
                Task::none()
            }
            Message::PermanentDeleteDone(results) => {
                self.folder_statuses = results;
                let folders: Vec<String> =
                    self.data.folders.iter().map(|f| f.path.clone()).collect();
                Task::perform(scan_folders(folders), Message::ScanComplete)
            }
            Message::ConfigTabChanged(tab) => {
                // Save strategy config immediately when leaving the Strategy tab
                if self.config_tab == ConfigTab::Strategy && tab != ConfigTab::Strategy {
                    self.save_strategy_and_llm_config();
                }
                self.config_tab = tab;
                Task::none()
            }
            Message::ConfigFolderPathChanged(index, value) => {
                if let Some(row) = self.config_folders.get_mut(index) {
                    row.path = value;
                }
                Task::none()
            }
            Message::ConfigFolderDescChanged(index, value) => {
                if let Some(row) = self.config_folders.get_mut(index) {
                    row.description = value;
                }
                Task::none()
            }
            Message::AddFolderRow => {
                self.config_folders.push(ConfigFolderRow {
                    path: String::new(),
                    description: String::new(),
                });
                Task::none()
            }
            Message::RemoveFolderRow(index) => {
                if index < self.config_folders.len() {
                    self.config_folders.remove(index);
                }
                Task::none()
            }
            Message::ConfigThemeChanged(theme) => {
                self.config_theme = theme.clone();
                self.data.update_theme(theme);
                Task::done(Message::ThemeApplied)
            }
            Message::ThemeApplied => {
                Task::none()
            }

            // Strategy config changes
            Message::StrategyBaseUrlChanged(val) => {
                self.strategy_base_url = val;
                Task::none()
            }
            Message::StrategyModelChanged(val) => {
                self.strategy_model = val;
                Task::none()
            }
            Message::StrategyApiKeyChanged(val) => {
                self.strategy_api_key = val;
                Task::none()
            }
            Message::StrategySkipListChanged(val) => {
                self.strategy_skip_list = val;
                Task::none()
            }
            Message::StrategyMinSizeChanged(val) => {
                self.strategy_min_size = val;
                Task::none()
            }
            Message::StrategySkipRecentChanged(val) => {
                self.strategy_skip_recent = val;
                Task::none()
            }

            // Connect test
            Message::ConnectClicked => {
                self.show_connect_dialog = true;
                self.connect_dialog_text = self.t().connect_dialog_connecting.to_string();
                let llm = crate::data::LlmConfig {
                    base_url: self.strategy_base_url.clone(),
                    model: self.strategy_model.clone(),
                    api_key: self.strategy_api_key.clone(),
                };
                Task::perform(llm::test_llm_connection(llm), Message::ConnectResult)
            }
            Message::ConnectResult(result) => {
                self.connect_dialog_text = result;
                Task::none()
            }
            Message::CloseConnectDialog => {
                self.show_connect_dialog = false;
                Task::none()
            }

            // Auto Discovery
            Message::AutoDiscoverClicked => {
                // Save current config first so auto_discover reads latest
                self.save_strategy_config();
                self.show_auto_discover_dialog = true;
                self.auto_discover_dialog_text = self.t().auto_discover_analyzing.to_string();
                let data_dir = self.data.data_dir.clone();
                Task::perform(llm::auto_discover(self.locale.as_str().to_string(), data_dir), Message::AutoDiscoverResult)
            }
            Message::AutoDiscoverResult(result) => {
                self.auto_discover_dialog_text = result;
                Task::none()
            }
            Message::CloseAutoDiscoverDialog => {
                self.show_auto_discover_dialog = false;
                // Refresh config folders from disk (auto_discover may have added entries)
                self.data = AppData::load_from(self.data.data_dir.clone());
                self.config_folders = self
                    .data
                    .folders
                    .iter()
                    .map(|f| ConfigFolderRow {
                        path: f.path.clone(),
                        description: f.description.clone(),
                    })
                    .collect();
                Task::none()
            }

            // Language
            Message::LanguageChanged(lang) => {
                self.locale = Locale::from_str(&lang);
                self.data.update_language(lang);
                Task::none()
            }

            // Clear all folders
            Message::ClearFoldersClicked => {
                self.show_clear_confirm = true;
                Task::none()
            }
            Message::ConfirmClearFolders => {
                self.show_clear_confirm = false;
                self.config_folders.clear();
                Task::none()
            }
            Message::CancelClearFolders => {
                self.show_clear_confirm = false;
                Task::none()
            }

            Message::CloseConfig => {
                // Save folders
                // Build a lookup of original selected state by path
                let original_selected: std::collections::HashMap<String, bool> = self
                    .data
                    .folders
                    .iter()
                    .map(|f| (f.path.trim().to_ascii_lowercase(), f.selected))
                    .collect();

                let new_folders: Vec<FolderEntry> = self
                    .config_folders
                    .iter()
                    .filter(|r| !r.path.trim().is_empty())
                    .map(|r| {
                        let key = r.path.trim().to_ascii_lowercase();
                        let selected = original_selected.get(&key).copied().unwrap_or(true);
                        FolderEntry {
                            path: r.path.trim().to_string(),
                            description: r.description.trim().to_string(),
                            selected,
                        }
                    })
                    .collect();
                self.data.update_folders(new_folders);
                self.data.update_theme(self.config_theme.clone());

                // Save LLM + strategy config
                self.save_strategy_and_llm_config();

                self.folder_sizes = vec![None; self.data.folders.len()];
                self.folder_statuses = vec![None; self.data.folders.len()];
                self.show_config = false;

                let folders: Vec<String> =
                    self.data.folders.iter().map(|f| f.path.clone()).collect();
                Task::perform(scan_folders(folders), Message::ScanComplete)
            }
        }
    }

    fn save_strategy_config(&mut self) {
        let min_size = parse_size_to_mb(&self.strategy_min_size);
        let skip_recent = self.strategy_skip_recent.trim().parse::<u32>().unwrap_or(7);
        self.data.update_strategy_config(
            self.strategy_skip_list.clone(),
            min_size,
            skip_recent,
        );
    }

    fn save_strategy_and_llm_config(&mut self) {
        self.data.update_llm_config(
            self.strategy_base_url.clone(),
            self.strategy_model.clone(),
            self.strategy_api_key.clone(),
        );
        self.save_strategy_config();
    }
}

// ─── Helper: parse size string like "500MB" or "1GB" to MB ──────────

fn parse_size_to_mb(s: &str) -> u64 {
    let s = s.trim().to_uppercase();
    if s.is_empty() {
        return 1024; // default 1GB
    }
    if let Some(num_str) = s.strip_suffix("GB") {
        if let Ok(n) = num_str.trim().parse::<f64>() {
            return (n * 1024.0) as u64;
        }
    }
    if let Some(num_str) = s.strip_suffix("MB") {
        if let Ok(n) = num_str.trim().parse::<f64>() {
            return n as u64;
        }
    }
    // Try plain number as MB
    if let Ok(n) = s.parse::<u64>() {
        return n;
    }
    1024 // default
}

fn format_mb_to_display(mb: u64) -> String {
    if mb >= 1024 && mb % 1024 == 0 {
        format!("{}GB", mb / 1024)
    } else {
        format!("{}MB", mb)
    }
}

// ─── System theme detection ─────────────────────────────────────────

#[cfg(windows)]
fn detect_system_dark_mode() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    if let Ok(key) = hkcu.open_subkey(path) {
        if let Ok(val) = key.get_value::<u32, _>("AppsUseLightTheme") {
            return val == 0;
        }
    }
    false
}

#[cfg(not(windows))]
fn detect_system_dark_mode() -> bool {
    false
}

// ─── Async Operations ───────────────────────────────────────────────

async fn scan_folders(folders: Vec<String>) -> Vec<Option<u64>> {
    folders.iter().map(|p| Some(calculate_dir_size(p))).collect()
}

async fn do_delete(
    selected: Vec<(usize, String)>,
    total: usize,
    permanent: bool,
) -> Vec<Option<i8>> {
    let mut results: Vec<Option<i8>> = vec![None; total];
    for (index, path) in &selected {
        let result = if permanent {
            permanently_delete(path)
        } else {
            move_to_recycle_bin(path)
        };
        if result == DeleteResult::Succeed {
            results[*index] = Some(0)
        } else if result == DeleteResult::Failed {
            results[*index] = Some(1)
        } else if result == DeleteResult::PathNotFound {
            results[*index] = Some(2)
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::FolderEntry;
    use std::fs;

    /// Build an App with a temp data dir so tests don't pollute each other.
    fn test_app() -> App {
        let dir = std::env::temp_dir().join(format!("clearmyc_app_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let data = AppData::load_from(dir);
        App {
            config_folders: Vec::new(),
            config_theme: data.settings.theme.clone(),
            strategy_base_url: String::new(),
            strategy_model: String::new(),
            strategy_api_key: String::new(),
            strategy_skip_list: String::new(),
            strategy_min_size: "1GB".to_string(),
            strategy_skip_recent: "7".to_string(),
            data,
            folder_sizes: Vec::new(),
            folder_statuses: Vec::new(),
            show_config: false,
            config_tab: ConfigTab::Folder,
            show_delete_confirm: false,
            show_permanent_confirm: false,
            scanning: false,
            show_connect_dialog: false,
            connect_dialog_text: String::new(),
            show_auto_discover_dialog: false,
            auto_discover_dialog_text: String::new(),
            locale: Locale::EnUs,
            show_clear_confirm: false,
        }
    }

    fn cleanup_app(app: &App) {
        let _ = fs::remove_dir_all(&app.data.data_dir);
    }

    #[test]
    fn test_toggle_selected_updates_state() {
        let mut app = test_app();
        app.data.update_folders(vec![
            FolderEntry { path: "C:\\A".into(), description: "".into(), selected: true },
            FolderEntry { path: "C:\\B".into(), description: "".into(), selected: false },
        ]);
        app.folder_sizes = vec![None; 2];
        app.folder_statuses = vec![None; 2];

        let _ = app.update(Message::ToggleSelected(0));
        assert!(!app.data.folders[0].selected);

        let _ = app.update(Message::ToggleSelected(1));
        assert!(app.data.folders[1].selected);

        cleanup_app(&app);
    }

    #[test]
    fn test_refresh_clears_statuses_and_sets_scanning() {
        let mut app = test_app();
        app.data.update_folders(vec![
            FolderEntry { path: "C:\\X".into(), description: "".into(), selected: true },
        ]);
        app.folder_statuses = vec![Some(0)];

        let _ = app.update(Message::Refresh);
        assert!(app.scanning);
        assert_eq!(app.folder_statuses, vec![None]);

        cleanup_app(&app);
    }

    #[test]
    fn test_scan_complete_stores_sizes() {
        let mut app = test_app();
        app.scanning = true;

        let _ = app.update(Message::ScanComplete(vec![Some(1024), Some(2048)]));
        assert!(!app.scanning);
        assert_eq!(app.folder_sizes, vec![Some(1024), Some(2048)]);

        cleanup_app(&app);
    }

    #[test]
    fn test_delete_clicked_shows_confirm() {
        let mut app = test_app();

        let _ = app.update(Message::DeleteClicked);
        assert!(app.show_delete_confirm);

        cleanup_app(&app);
    }

    #[test]
    fn test_cancel_delete_hides_confirm() {
        let mut app = test_app();
        app.show_delete_confirm = true;

        let _ = app.update(Message::CancelDelete);
        assert!(!app.show_delete_confirm);

        cleanup_app(&app);
    }

    #[test]
    fn test_permanent_delete_clicked_shows_confirm() {
        let mut app = test_app();

        let _ = app.update(Message::PermanentDeleteClicked);
        assert!(app.show_permanent_confirm);

        cleanup_app(&app);
    }

    #[test]
    fn test_cancel_permanent_delete_hides_confirm() {
        let mut app = test_app();
        app.show_permanent_confirm = true;

        let _ = app.update(Message::CancelPermanentDelete);
        assert!(!app.show_permanent_confirm);

        cleanup_app(&app);
    }

    #[test]
    fn test_open_config_populates_config_state() {
        let mut app = test_app();
        app.data.update_folders(vec![
            FolderEntry { path: "C:\\Dir".into(), description: "desc".into(), selected: true },
        ]);

        let _ = app.update(Message::OpenConfig);
        assert!(app.show_config);
        assert_eq!(app.config_folders.len(), 1);
        assert_eq!(app.config_folders[0].path, "C:\\Dir");
        assert_eq!(app.config_folders[0].description, "desc");
        assert_eq!(app.config_theme, "light");

        cleanup_app(&app);
    }

    #[test]
    fn test_config_tab_changed() {
        let mut app = test_app();

        let _ = app.update(Message::ConfigTabChanged(ConfigTab::Theme));
        assert_eq!(app.config_tab, ConfigTab::Theme);

        let _ = app.update(Message::ConfigTabChanged(ConfigTab::About));
        assert_eq!(app.config_tab, ConfigTab::About);

        let _ = app.update(Message::ConfigTabChanged(ConfigTab::Folder));
        assert_eq!(app.config_tab, ConfigTab::Folder);

        let _ = app.update(Message::ConfigTabChanged(ConfigTab::Strategy));
        assert_eq!(app.config_tab, ConfigTab::Strategy);

        cleanup_app(&app);
    }

    #[test]
    fn test_config_folder_editing() {
        let mut app = test_app();
        app.config_folders = vec![
            ConfigFolderRow { path: "".into(), description: "".into() },
        ];

        let _ = app.update(Message::ConfigFolderPathChanged(0, "C:\\New".into()));
        assert_eq!(app.config_folders[0].path, "C:\\New");

        let _ = app.update(Message::ConfigFolderDescChanged(0, "new desc".into()));
        assert_eq!(app.config_folders[0].description, "new desc");

        cleanup_app(&app);
    }

    #[test]
    fn test_add_folder_row() {
        let mut app = test_app();
        assert_eq!(app.config_folders.len(), 0);

        let _ = app.update(Message::AddFolderRow);
        assert_eq!(app.config_folders.len(), 1);
        assert!(app.config_folders[0].path.is_empty());

        cleanup_app(&app);
    }

    #[test]
    fn test_config_theme_changed_applies_immediately() {
        let mut app = test_app();

        let _ = app.update(Message::ConfigThemeChanged("dark".into()));
        assert_eq!(app.config_theme, "dark");
        assert_eq!(app.data.settings.theme, "dark");

        cleanup_app(&app);
    }

    #[test]
    fn test_close_config_saves_and_filters_empty_paths() {
        let mut app = test_app();
        app.show_config = true;
        app.config_folders = vec![
            ConfigFolderRow { path: "C:\\Valid".into(), description: "ok".into() },
            ConfigFolderRow { path: "  ".into(), description: "empty path".into() },
            ConfigFolderRow { path: "D:\\Also".into(), description: "".into() },
        ];
        app.config_theme = "dark".into();

        let _ = app.update(Message::CloseConfig);
        assert!(!app.show_config);
        assert_eq!(app.data.folders.len(), 2);
        assert_eq!(app.data.folders[0].path, "C:\\Valid");
        assert_eq!(app.data.folders[1].path, "D:\\Also");
        assert_eq!(app.data.settings.theme, "dark");

        cleanup_app(&app);
    }

    #[test]
    fn test_delete_done_stores_results() {
        let mut app = test_app();
        app.data.update_folders(vec![
            FolderEntry { path: "C:\\A".into(), description: "".into(), selected: true },
            FolderEntry { path: "C:\\B".into(), description: "".into(), selected: false },
        ]);

        let results = vec![Some(0), None];
        let _ = app.update(Message::DeleteDone(results));
        assert_eq!(app.folder_statuses, vec![Some(0), None]);

        cleanup_app(&app);
    }

    #[test]
    fn test_theme_method() {
        let mut app = test_app();

        app.data.settings.theme = "light".into();
        assert!(matches!(app.theme(), Theme::Light));

        app.data.settings.theme = "dark".into();
        assert!(matches!(app.theme(), Theme::Dark));

        app.data.settings.theme = "system".into();
        let t = app.theme();
        assert!(matches!(t, Theme::Light | Theme::Dark));

        cleanup_app(&app);
    }

    // ─── Strategy config tests ──────────────────────────────────────

    #[test]
    fn test_strategy_fields_update() {
        let mut app = test_app();

        let _ = app.update(Message::StrategyBaseUrlChanged("http://test.com/v1".into()));
        assert_eq!(app.strategy_base_url, "http://test.com/v1");

        let _ = app.update(Message::StrategyModelChanged("Qwen".into()));
        assert_eq!(app.strategy_model, "Qwen");

        let _ = app.update(Message::StrategyApiKeyChanged("sk-123".into()));
        assert_eq!(app.strategy_api_key, "sk-123");

        let _ = app.update(Message::StrategySkipListChanged("C:\\skip".into()));
        assert_eq!(app.strategy_skip_list, "C:\\skip");

        let _ = app.update(Message::StrategyMinSizeChanged("500MB".into()));
        assert_eq!(app.strategy_min_size, "500MB");

        let _ = app.update(Message::StrategySkipRecentChanged("14".into()));
        assert_eq!(app.strategy_skip_recent, "14");

        cleanup_app(&app);
    }

    #[test]
    fn test_connect_clicked_shows_dialog() {
        let mut app = test_app();

        let _ = app.update(Message::ConnectClicked);
        assert!(app.show_connect_dialog);
        assert_eq!(app.connect_dialog_text, "Connecting...");

        cleanup_app(&app);
    }

    #[test]
    fn test_close_connect_dialog() {
        let mut app = test_app();
        app.show_connect_dialog = true;

        let _ = app.update(Message::CloseConnectDialog);
        assert!(!app.show_connect_dialog);

        cleanup_app(&app);
    }

    #[test]
    fn test_auto_discover_clicked_shows_dialog() {
        let mut app = test_app();

        let _ = app.update(Message::AutoDiscoverClicked);
        assert!(app.show_auto_discover_dialog);
        assert_eq!(app.auto_discover_dialog_text, "Analyzing...This could take quite a while");

        cleanup_app(&app);
    }

    #[test]
    fn test_close_auto_discover_dialog() {
        let mut app = test_app();
        app.show_auto_discover_dialog = true;

        let _ = app.update(Message::CloseAutoDiscoverDialog);
        assert!(!app.show_auto_discover_dialog);

        cleanup_app(&app);
    }

    // ─── parse_size_to_mb tests ─────────────────────────────────────

    #[test]
    fn test_parse_size_to_mb() {
        assert_eq!(parse_size_to_mb("500MB"), 500);
        assert_eq!(parse_size_to_mb("1GB"), 1024);
        assert_eq!(parse_size_to_mb("2GB"), 2048);
        assert_eq!(parse_size_to_mb(""), 1024); // default
        assert_eq!(parse_size_to_mb("abc"), 1024); // default
        assert_eq!(parse_size_to_mb("100"), 100); // plain number = MB
    }

    #[test]
    fn test_format_mb_to_display() {
        assert_eq!(format_mb_to_display(1024), "1GB");
        assert_eq!(format_mb_to_display(2048), "2GB");
        assert_eq!(format_mb_to_display(500), "500MB");
        assert_eq!(format_mb_to_display(1500), "1500MB");
    }

    // ─── CloseConfig preserves selected state ───────────────────────

    #[test]
    fn test_close_config_preserves_selected_state() {
        let mut app = test_app();
        app.data.update_folders(vec![
            FolderEntry { path: "C:\\Selected".into(), description: "a".into(), selected: true },
            FolderEntry { path: "C:\\Unselected".into(), description: "b".into(), selected: false },
        ]);
        app.show_config = true;
        app.config_folders = vec![
            ConfigFolderRow { path: "C:\\Selected".into(), description: "a".into() },
            ConfigFolderRow { path: "C:\\Unselected".into(), description: "b".into() },
        ];

        let _ = app.update(Message::CloseConfig);
        assert!(app.data.folders[0].selected);
        assert!(!app.data.folders[1].selected);

        cleanup_app(&app);
    }

    #[test]
    fn test_close_config_new_folder_defaults_selected_true() {
        let mut app = test_app();
        app.show_config = true;
        app.config_folders = vec![
            ConfigFolderRow { path: "C:\\BrandNew".into(), description: "new".into() },
        ];

        let _ = app.update(Message::CloseConfig);
        assert_eq!(app.data.folders.len(), 1);
        assert!(app.data.folders[0].selected); // new folder defaults to true

        cleanup_app(&app);
    }

    // ─── ConfigTabChanged saves strategy when leaving Strategy ──────

    #[test]
    fn test_config_tab_changed_saves_strategy_on_leave() {
        let mut app = test_app();
        app.config_tab = ConfigTab::Strategy;
        app.strategy_base_url = "http://test.com/v1".into();
        app.strategy_model = "TestModel".into();
        app.strategy_api_key = "sk-test-123".into();
        app.strategy_skip_list = "C:\\skip".into();
        app.strategy_min_size = "500MB".into();
        app.strategy_skip_recent = "14".into();

        // Switch away from Strategy tab
        let _ = app.update(Message::ConfigTabChanged(ConfigTab::Folder));

        // Verify LLM config was saved
        let llm = app.data.read_llm_config();
        assert_eq!(llm.base_url, "http://test.com/v1");
        assert_eq!(llm.model, "TestModel");
        assert_eq!(llm.api_key, "sk-test-123");

        // Verify strategy config was saved
        let s = app.data.read_strategy_config();
        assert_eq!(s.skip_list, "C:\\skip");
        assert_eq!(s.min_size, 500);
        assert_eq!(s.skip_recent, 14);

        cleanup_app(&app);
    }

    #[test]
    fn test_config_tab_changed_no_save_when_not_leaving_strategy() {
        let mut app = test_app();
        app.config_tab = ConfigTab::Folder;
        app.strategy_base_url = "http://unsaved.com/v1".into();

        // Switch from Folder to Theme — should NOT save strategy
        let _ = app.update(Message::ConfigTabChanged(ConfigTab::Theme));
        let llm = app.data.read_llm_config();
        assert_eq!(llm.base_url, ""); // still empty, not saved

        cleanup_app(&app);
    }

    // ─── RemoveFolderRow ────────────────────────────────────────────

    #[test]
    fn test_remove_folder_row() {
        let mut app = test_app();
        app.config_folders = vec![
            ConfigFolderRow { path: "A".into(), description: "".into() },
            ConfigFolderRow { path: "B".into(), description: "".into() },
            ConfigFolderRow { path: "C".into(), description: "".into() },
        ];

        let _ = app.update(Message::RemoveFolderRow(1));
        assert_eq!(app.config_folders.len(), 2);
        assert_eq!(app.config_folders[0].path, "A");
        assert_eq!(app.config_folders[1].path, "C");

        cleanup_app(&app);
    }

    #[test]
    fn test_remove_folder_row_out_of_bounds() {
        let mut app = test_app();
        app.config_folders = vec![
            ConfigFolderRow { path: "A".into(), description: "".into() },
        ];

        // Should not panic
        let _ = app.update(Message::RemoveFolderRow(5));
        assert_eq!(app.config_folders.len(), 1);

        cleanup_app(&app);
    }

    // ─── ConnectResult / AutoDiscoverResult ─────────────────────────

    #[test]
    fn test_connect_result_updates_text() {
        let mut app = test_app();
        app.show_connect_dialog = true;
        app.connect_dialog_text = "Connecting...".into();

        let _ = app.update(Message::ConnectResult("Connection successful!".into()));
        assert_eq!(app.connect_dialog_text, "Connection successful!");

        cleanup_app(&app);
    }

    #[test]
    fn test_auto_discover_result_updates_text() {
        let mut app = test_app();
        app.show_auto_discover_dialog = true;
        app.auto_discover_dialog_text = "Analyzing...".into();

        let _ = app.update(Message::AutoDiscoverResult("Found 5 directories".into()));
        assert_eq!(app.auto_discover_dialog_text, "Found 5 directories");

        cleanup_app(&app);
    }

    // ─── ThemeApplied is a no-op ────────────────────────────────────

    #[test]
    fn test_theme_applied_is_noop() {
        let mut app = test_app();
        app.data.settings.theme = "dark".into();

        let _ = app.update(Message::ThemeApplied);
        assert_eq!(app.data.settings.theme, "dark"); // unchanged

        cleanup_app(&app);
    }

    // ─── PermanentDeleteDone ────────────────────────────────────────

    #[test]
    fn test_permanent_delete_done_stores_results() {
        let mut app = test_app();
        app.data.update_folders(vec![
            FolderEntry { path: "C:\\A".into(), description: "".into(), selected: true },
            FolderEntry { path: "C:\\B".into(), description: "".into(), selected: false },
        ]);

        let results = vec![Some(0), None];
        let _ = app.update(Message::PermanentDeleteDone(results));
        assert_eq!(app.folder_statuses, vec![Some(0), None]);

        cleanup_app(&app);
    }

    // ─── Language tests ─────────────────────────────────────────────

    #[test]
    fn test_language_changed_updates_locale() {
        let mut app = test_app();
        assert_eq!(app.locale, Locale::EnUs);

        let _ = app.update(Message::LanguageChanged("zh-CN".into()));
        assert_eq!(app.locale, Locale::ZhCn);
        assert_eq!(app.data.settings.language, "zh-CN");

        let _ = app.update(Message::LanguageChanged("en-US".into()));
        assert_eq!(app.locale, Locale::EnUs);
        assert_eq!(app.data.settings.language, "en-US");

        cleanup_app(&app);
    }

    // ─── Clear folders tests ────────────────────────────────────────

    #[test]
    fn test_clear_folders_clicked_shows_confirm() {
        let mut app = test_app();

        let _ = app.update(Message::ClearFoldersClicked);
        assert!(app.show_clear_confirm);

        cleanup_app(&app);
    }

    #[test]
    fn test_confirm_clear_folders_clears_list() {
        let mut app = test_app();
        app.config_folders.push(ConfigFolderRow {
            path: "C:\\Test".into(),
            description: "test".into(),
        });
        app.show_clear_confirm = true;

        let _ = app.update(Message::ConfirmClearFolders);
        assert!(!app.show_clear_confirm);
        assert!(app.config_folders.is_empty());

        cleanup_app(&app);
    }

    #[test]
    fn test_cancel_clear_folders_hides_confirm() {
        let mut app = test_app();
        app.show_clear_confirm = true;

        let _ = app.update(Message::CancelClearFolders);
        assert!(!app.show_clear_confirm);

        cleanup_app(&app);
    }
}
