// ─── Internationalization (i18n) ─────────────────────────────────────
//
// All user-facing fixed text is defined here so that adding a new
// language only requires adding a new `Locale` variant and its
// corresponding `Texts` instance.
//
// The texts are compiled into the binary (no external file needed).

/// Supported locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    EnUs,
    ZhCn,
}

impl Locale {
    /// Convert a settings string (e.g. "en-US") to a `Locale`.
    pub fn from_str(s: &str) -> Self {
        match s {
            "zh-CN" => Locale::ZhCn,
            _ => Locale::EnUs,
        }
    }

    /// Convert a `Locale` back to the settings string.
    pub fn as_str(self) -> &'static str {
        match self {
            Locale::EnUs => "en-US",
            Locale::ZhCn => "zh-CN",
        }
    }

    /// Display name shown in the UI language selector.
    pub fn display_name(self) -> &'static str {
        match self {
            Locale::EnUs => "English",
            Locale::ZhCn => "\u{4E2D}\u{6587}",
        }
    }
}

/// All user-facing strings used in the application.
#[derive(Debug, Clone)]
pub struct Texts {
    // ── Main page ────────────────────────────────────────────────────
    pub app_title: &'static str,
    pub col_path: &'static str,
    pub col_description: &'static str,
    pub col_size: &'static str,
    pub col_status: &'static str,
    pub btn_refresh: &'static str,
    pub btn_delete: &'static str,
    pub btn_permanently_delete: &'static str,
    pub status_succeed: &'static str,
    pub status_failed: &'static str,
    pub status_skipped: &'static str,
    pub scanning_text: &'static str,

    // ── Delete confirm dialog ────────────────────────────────────────
    pub delete_confirm_icon: &'static str,
    pub delete_confirm_title: &'static str,
    pub delete_confirm_msg: &'static str,
    pub btn_yes_delete: &'static str,
    pub btn_cancel: &'static str,

    // ── Permanent delete confirm dialog ──────────────────────────────
    pub perm_confirm_icon: &'static str,
    pub perm_confirm_title: &'static str,
    pub perm_confirm_msg: &'static str,
    pub btn_yes_perm_delete: &'static str,

    // ── Config page – common ─────────────────────────────────────────
    pub btn_save_and_close: &'static str,

    // ── Config page – sidebar tabs ───────────────────────────────────
    pub tab_folders: &'static str,
    pub tab_strategy: &'static str,
    pub tab_theme: &'static str,
    pub tab_about: &'static str,

    // ── Config – Folder panel ────────────────────────────────────────
    pub folder_title: &'static str,
    pub folder_subtitle: &'static str,
    pub folder_path_placeholder: &'static str,
    pub folder_desc_placeholder: &'static str,
    pub btn_add_folder: &'static str,
    pub btn_clear: &'static str,
    pub btn_auto_discovery: &'static str,

    // ── Config – Strategy panel ──────────────────────────────────────
    pub strategy_llm_title: &'static str,
    pub strategy_llm_description: &'static str,
    pub strategy_base_url: &'static str,
    pub strategy_model: &'static str,
    pub strategy_api_key: &'static str,
    pub btn_connect: &'static str,
    pub strategy_rules_title: &'static str,
    pub strategy_skip_list: &'static str,
    pub strategy_min_size: &'static str,
    pub strategy_skip_recent: &'static str,
    pub strategy_skip_recent_suffix: &'static str,

    // ── Config – Theme panel ─────────────────────────────────────────
    pub theme_title: &'static str,
    pub theme_description: &'static str,
    pub theme_light: &'static str,
    pub theme_dark: &'static str,
    pub theme_system: &'static str,
    pub language_label: &'static str,

    // ── Config – About panel ─────────────────────────────────────────
    pub about_title: &'static str,
    pub about_description: &'static str,
    pub about_homepage_label: &'static str,
    pub about_version_label: &'static str,

    // ── Connect dialog ───────────────────────────────────────────────
    pub connect_dialog_connecting: &'static str,
    pub btn_close: &'static str,

    // ── Auto Discovery dialog ────────────────────────────────────────
    pub auto_discover_analyzing: &'static str,

    // ── Clear confirm dialog ─────────────────────────────────────────
    pub clear_confirm_title: &'static str,
    pub clear_confirm_msg: &'static str,
    pub btn_yes_clear: &'static str,
}

/// English (US) texts.
const EN_US: Texts = Texts {
    // Main page
    app_title: "ClearMyC",
    col_path: "Path",
    col_description: "Description",
    col_size: "Size",
    col_status: "Status",
    btn_refresh: "Refresh",
    btn_delete: "Delete",
    btn_permanently_delete: "Permanently Delete",
    status_succeed: "Succeed \u{2705}",
    status_failed: "Failed \u{274C}",
    status_skipped: "Skipped \u{23ED}",
    scanning_text: "Scanning...",

    // Delete confirm
    delete_confirm_icon: "\u{1F5D1}",
    delete_confirm_title: "Move to Recycle Bin?",
    delete_confirm_msg: "Selected folders will be moved to the Recycle Bin. You can recover them later if needed.",
    btn_yes_delete: "Yes, Delete",
    btn_cancel: "Cancel",

    // Permanent delete confirm
    perm_confirm_icon: "\u{26A0}",
    perm_confirm_title: "Permanently Delete?",
    perm_confirm_msg: "Selected folders will be permanently deleted. This action CANNOT be undone!",
    btn_yes_perm_delete: "Yes, Permanently Delete",

    // Config common
    btn_save_and_close: "Save & Close",

    // Config sidebar
    tab_folders: "Folders",
    tab_strategy: "Strategy",
    tab_theme: "Theme",
    tab_about: "About",

    // Config – Folder
    folder_title: "Manage Folders",
    folder_subtitle: "Add directories you want to clean up. Empty path won't be saved.",
    folder_path_placeholder: "C:\\path\\to\\folder",
    folder_desc_placeholder: "Description (optional)",
    btn_add_folder: "+ Add Folder",
    btn_clear: "Clear",
    btn_auto_discovery: "Auto Discovery",

    // Config – Strategy
    strategy_llm_title: "LLM Configuration",
    strategy_llm_description: "Configure LLM connection for auto-discovery and set filtering rules.",
    strategy_base_url: "BaseUrl:",
    strategy_model: "Model:",
    strategy_api_key: "ApiKey:",
    btn_connect: "Connect",
    strategy_rules_title: "Search Rules",
    strategy_skip_list: "Skip List:",
    strategy_min_size: "Min Size:",
    strategy_skip_recent: "Skip Recent:",
    strategy_skip_recent_suffix: "days",

    // Config – Theme
    theme_title: "Appearance",
    theme_description: "Choose your preferred color scheme.",
    theme_light: "Light",
    theme_dark: "Dark",
    theme_system: "System",
    language_label: "Language",

    // Config – About
    about_title: "About ClearMyC",
    about_description: "A lightweight Windows 10 desktop app for discovering and cleaning up disk-hogging directories.",
    about_homepage_label: "Homepage:",
    about_version_label: "Version",

    // Connect dialog
    connect_dialog_connecting: "Connecting...",
    btn_close: "Close",

    // Auto Discovery dialog
    auto_discover_analyzing: "Analyzing...This could take quite a while",

    // Clear confirm
    clear_confirm_title: "Clear All Folders?",
    clear_confirm_msg: "This will remove all folder entries from the list. No files on disk will be deleted.",
    btn_yes_clear: "Yes, Clear",
};

/// Simplified Chinese texts.
const ZH_CN: Texts = Texts {
    // Main page
    app_title: "ClearMyC",
    col_path: "\u{8DEF}\u{5F84}",
    col_description: "\u{63CF}\u{8FF0}",
    col_size: "\u{5927}\u{5C0F}",
    col_status: "\u{72B6}\u{6001}",
    btn_refresh: "\u{5237}\u{65B0}",
    btn_delete: "\u{5220}\u{9664}",
    btn_permanently_delete: "\u{5F7B}\u{5E95}\u{5220}\u{9664}",
    status_succeed: "\u{6210}\u{529F} \u{2705}",
    status_failed: "\u{5931}\u{8D25} \u{274C}",
    status_skipped: "\u{8DF3}\u{8FC7} \u{23ED}",
    scanning_text: "\u{626B}\u{63CF}\u{4E2D}...",

    // Delete confirm
    delete_confirm_icon: "\u{1F5D1}",
    delete_confirm_title: "\u{79FB}\u{5230}\u{56DE}\u{6536}\u{7AD9}\u{FF1F}",
    delete_confirm_msg: "\u{9009}\u{4E2D}\u{7684}\u{6587}\u{4EF6}\u{5939}\u{5C06}\u{88AB}\u{79FB}\u{5230}\u{56DE}\u{6536}\u{7AD9}\u{3002}\u{5982}\u{6709}\u{9700}\u{8981}\u{FF0C}\u{60A8}\u{53EF}\u{4EE5}\u{7A0D}\u{540E}\u{6062}\u{590D}\u{3002}",
    btn_yes_delete: "\u{786E}\u{8BA4}\u{5220}\u{9664}",
    btn_cancel: "\u{53D6}\u{6D88}",

    // Permanent delete confirm
    perm_confirm_icon: "\u{26A0}",
    perm_confirm_title: "\u{5F7B}\u{5E95}\u{5220}\u{9664}\u{FF1F}",
    perm_confirm_msg: "\u{9009}\u{4E2D}\u{7684}\u{6587}\u{4EF6}\u{5939}\u{5C06}\u{88AB}\u{5F7B}\u{5E95}\u{5220}\u{9664}\u{3002}\u{6B64}\u{64CD}\u{4F5C}\u{4E0D}\u{53EF}\u{64A4}\u{9500}\u{FF01}",
    btn_yes_perm_delete: "\u{786E}\u{8BA4}\u{5F7B}\u{5E95}\u{5220}\u{9664}",

    // Config common
    btn_save_and_close: "\u{4FDD}\u{5B58}\u{5E76}\u{5173}\u{95ED}",

    // Config sidebar
    tab_folders: "\u{6587}\u{4EF6}\u{5939}",
    tab_strategy: "\u{7B56}\u{7565}",
    tab_theme: "\u{4E3B}\u{9898}",
    tab_about: "\u{5173}\u{4E8E}",

    // Config – Folder
    folder_title: "\u{7BA1}\u{7406}\u{6587}\u{4EF6}\u{5939}",
    folder_subtitle: "\u{6DFB}\u{52A0}\u{60A8}\u{8981}\u{6E05}\u{7406}\u{7684}\u{76EE}\u{5F55}\u{3002}\u{7A7A}\u{8DEF}\u{5F84}\u{4E0D}\u{4F1A}\u{88AB}\u{4FDD}\u{5B58}\u{3002}",
    folder_path_placeholder: "C:\\\u{8DEF}\u{5F84}\\\u{6587}\u{4EF6}\u{5939}",
    folder_desc_placeholder: "\u{63CF}\u{8FF0}\u{FF08}\u{53EF}\u{9009}\u{FF09}",
    btn_add_folder: "+ \u{6DFB}\u{52A0}\u{6587}\u{4EF6}\u{5939}",
    btn_clear: "\u{6E05}\u{7A7A}",
    btn_auto_discovery: "\u{81EA}\u{52A8}\u{53D1}\u{73B0}",

    // Config – Strategy
    strategy_llm_title: "\u{5927}\u{6A21}\u{578B}\u{914D}\u{7F6E}",
    strategy_llm_description: "\u{4E3A}\u{5927}\u{8BED}\u{8A00}\u{6A21}\u{578B}\u{914D}\u{7F6E}\u{81EA}\u{52A8}\u{53D1}\u{73B0}\u{8FDE}\u{63A5}\u{5E76}\u{8BBE}\u{7F6E}\u{8FC7}\u{6EE4}\u{89C4}\u{5219}。",
    strategy_base_url: "\u{63A5}\u{53E3}\u{5730}\u{5740}:",
    strategy_model: "\u{6A21}\u{578B}:",
    strategy_api_key: "\u{5BC6}\u{94A5}:",
    btn_connect: "\u{8FDE}\u{63A5}\u{6D4B}\u{8BD5}",
    strategy_rules_title: "\u{68C0}\u{7D22}\u{89C4}\u{5219}",
    strategy_skip_list: "\u{8DF3}\u{8FC7}\u{5217}\u{8868}:",
    strategy_min_size: "\u{6700}\u{5C0F}\u{5927}\u{5C0F}:",
    strategy_skip_recent: "\u{8DF3}\u{8FC7}\u{6700}\u{8FD1}:",
    strategy_skip_recent_suffix: "\u{5929}",

    // Config – Theme
    theme_title: "\u{5916}\u{89C2}\u{8BBE}\u{7F6E}",
    theme_description: "\u{8BF7}\u{9009}\u{62E9}\u{60A8}\u{504F}\u{597D}\u{7684}\u{914D}\u{8272}\u{65B9}\u{6848}。",
    theme_light: "\u{6D45}\u{8272}",
    theme_dark: "\u{6DF1}\u{8272}",
    theme_system: "\u{8DDF}\u{968F}\u{7CFB}\u{7EDF}",
    language_label: "\u{8BED}\u{8A00}",

    // Config – About
    about_title: "\u{5173}\u{4E8E}ClearMyC",
    about_description: "\u{4E00}\u{6B3E}\u{8F7B}\u{91CF}\u{7EA7} Windows 10 \u{684C}\u{9762}\u{5E94}\u{7528}\u{FF0C}\u{7528}\u{4E8E}\u{53D1}\u{73B0}\u{548C}\u{6E05}\u{7406}\u{5360}\u{7528}\u{78C1}\u{76D8}\u{7A7A}\u{95F4}\u{7684}\u{76EE}\u{5F55}\u{3002}",
    about_homepage_label: "\u{4E3B}\u{9875}",
    about_version_label: "\u{7248}\u{672C}",

    // Connect dialog
    connect_dialog_connecting: "\u{8FDE}\u{63A5}\u{4E2D}...",
    btn_close: "\u{5173}\u{95ED}",

    // Auto Discovery dialog
    auto_discover_analyzing: "\u{5206}\u{6790}\u{4E2D}...\u{53EF}\u{80FD}\u{9700}\u{8981}\u{8F83}\u{957F}\u{65F6}\u{95F4}",

    // Clear confirm
    clear_confirm_title: "\u{6E05}\u{7A7A}\u{6240}\u{6709}\u{6587}\u{4EF6}\u{5939}\u{FF1F}",
    clear_confirm_msg: "\u{8FD9}\u{5C06}\u{4ECE}\u{5217}\u{8868}\u{4E2D}\u{79FB}\u{9664}\u{6240}\u{6709}\u{6587}\u{4EF6}\u{5939}\u{6761}\u{76EE}\u{3002}\u{78C1}\u{76D8}\u{4E0A}\u{7684}\u{6587}\u{4EF6}\u{4E0D}\u{4F1A}\u{88AB}\u{5220}\u{9664}\u{3002}",
    btn_yes_clear: "\u{786E}\u{8BA4}\u{6E05}\u{7A7A}",
};

/// Return the `Texts` for a given locale.
pub fn texts(locale: Locale) -> &'static Texts {
    match locale {
        Locale::EnUs => &EN_US,
        Locale::ZhCn => &ZH_CN,
    }
}
