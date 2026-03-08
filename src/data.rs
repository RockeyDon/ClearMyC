use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use aes::cipher::{BlockEncryptMut, BlockDecryptMut, KeyIvInit};
use aes::cipher::block_padding::Pkcs7;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

const DATA_FILE: &str = "clear_my_c.json";

// AES-128-CBC key and IV (fixed, for simple obfuscation of api_key)
const AES_KEY: &[u8; 16] = b"ClearMyC_AesK3y!";
const AES_IV: &[u8; 16] = b"ClearMyC_InitVec";

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

// ─── Encryption helpers ──────────────────────────────────────────────

pub fn encrypt_string(plaintext: &str) -> String {
    if plaintext.is_empty() {
        return String::new();
    }
    let pt_bytes = plaintext.as_bytes();
    // Allocate buffer with room for PKCS7 padding (up to 16 extra bytes)
    let pad_len = 16 - (pt_bytes.len() % 16);
    let total_len = pt_bytes.len() + pad_len;
    let mut buf = vec![0u8; total_len];
    buf[..pt_bytes.len()].copy_from_slice(pt_bytes);

    let enc = Aes128CbcEnc::new(AES_KEY.into(), AES_IV.into());
    let ct = enc.encrypt_padded_mut::<Pkcs7>(&mut buf, pt_bytes.len())
        .expect("buffer is correctly sized");
    BASE64.encode(ct)
}

pub fn decrypt_string(ciphertext: &str) -> String {
    if ciphertext.is_empty() {
        return String::new();
    }
    let mut ct_bytes = match BASE64.decode(ciphertext) {
        Ok(b) => b,
        Err(_) => return String::new(),
    };
    let dec = Aes128CbcDec::new(AES_KEY.into(), AES_IV.into());
    match dec.decrypt_padded_mut::<Pkcs7>(&mut ct_bytes) {
        Ok(pt) => String::from_utf8(pt.to_vec()).unwrap_or_default(),
        Err(_) => String::new(),
    }
}

// ─── Data structures ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub settings: Settings,
    pub folders: Vec<FolderEntry>,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub strategy: StrategyConfig,

    /// Runtime-only: the directory where data is stored.
    #[serde(skip)]
    pub data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "en-US".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderEntry {
    pub path: String,
    pub description: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub base_url: String,
    pub model: String,
    pub api_key: String, // stored encrypted in JSON
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            model: String::new(),
            api_key: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub skip_list: String,
    pub min_size: u64,
    pub skip_recent: u32,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            skip_list: String::from("C:\\$Recycle.Bin;C:\\Windows"),
            min_size: 1024, // 1024 MB = 1 GB in the UI, stored as MB
            skip_recent: 7,
        }
    }
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            settings: Settings {
                theme: "light".to_string(),
                language: "en-US".to_string(),
            },
            folders: Vec::new(),
            llm: LlmConfig::default(),
            strategy: StrategyConfig::default(),
            data_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

impl AppData {
    fn data_path(&self) -> PathBuf {
        self.data_dir.join(DATA_FILE)
    }

    /// Load from the current working directory.
    pub fn load() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        Self::load_from(exe_dir)
    }

    /// Load from a specific directory (useful for testing).
    pub fn load_from(dir: PathBuf) -> Self {
        let path = dir.join(DATA_FILE);
        let mut data: AppData = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        };
        data.data_dir = dir;
        if !path.exists() {
            data.save();
        }
        data
    }

    pub fn save(&self) {
        let path = self.data_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn update_folders(&mut self, folders: Vec<FolderEntry>) {
        self.folders = folders;
        self.save();
    }

    pub fn update_theme(&mut self, theme: String) {
        self.settings.theme = theme;
        self.save();
    }

    pub fn update_language(&mut self, language: String) {
        self.settings.language = language;
        self.save();
    }

    pub fn toggle_selected(&mut self, index: usize) {
        if let Some(folder) = self.folders.get_mut(index) {
            folder.selected = !folder.selected;
            self.save();
        }
    }

    // ─── New: LLM config ────────────────────────────────────────────

    /// Read LLM config, decrypting the api_key.
    pub fn read_llm_config(&self) -> LlmConfig {
        LlmConfig {
            base_url: self.llm.base_url.clone(),
            model: self.llm.model.clone(),
            api_key: decrypt_string(&self.llm.api_key),
        }
    }

    /// Write LLM config, encrypting the api_key before saving.
    pub fn update_llm_config(&mut self, base_url: String, model: String, api_key: String) {
        self.llm.base_url = base_url;
        self.llm.model = model;
        self.llm.api_key = encrypt_string(&api_key);
        self.save();
    }

    // ─── New: Strategy config ───────────────────────────────────────

    pub fn read_strategy_config(&self) -> StrategyConfig {
        self.strategy.clone()
    }

    pub fn update_strategy_config(&mut self, skip_list: String, min_size: u64, skip_recent: u32) {
        self.strategy.skip_list = skip_list;
        self.strategy.min_size = min_size;
        self.strategy.skip_recent = skip_recent;
        self.save();
    }

    /// Append discovered folders to the existing folders list.
    pub fn append_folders(&mut self, new_folders: Vec<FolderEntry>) {
        self.folders.extend(new_folders);
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Create a temporary directory for test isolation.
    fn temp_dir() -> PathBuf {
        let id = format!(
            "clearmyc_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(id);
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn cleanup(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_default_creates_file() {
        let dir = temp_dir();
        let data = AppData::load_from(dir.clone());
        assert!(dir.join(DATA_FILE).exists());
        assert_eq!(data.settings.theme, "light");
        assert!(data.folders.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_save_and_reload() {
        let dir = temp_dir();
        let mut data = AppData::load_from(dir.clone());

        data.update_folders(vec![
            FolderEntry {
                path: "C:\\Test".to_string(),
                description: "test folder".to_string(),
                selected: true,
            },
            FolderEntry {
                path: "D:\\Cache".to_string(),
                description: "".to_string(),
                selected: false,
            },
        ]);
        data.update_theme("dark".to_string());

        // Reload and verify
        let reloaded = AppData::load_from(dir.clone());
        assert_eq!(reloaded.settings.theme, "dark");
        assert_eq!(reloaded.folders.len(), 2);
        assert_eq!(reloaded.folders[0].path, "C:\\Test");
        assert_eq!(reloaded.folders[0].description, "test folder");
        assert!(reloaded.folders[0].selected);
        assert_eq!(reloaded.folders[1].path, "D:\\Cache");
        assert!(!reloaded.folders[1].selected);

        cleanup(&dir);
    }

    #[test]
    fn test_toggle_selected() {
        let dir = temp_dir();
        let mut data = AppData::load_from(dir.clone());
        data.update_folders(vec![FolderEntry {
            path: "C:\\Foo".to_string(),
            description: "".to_string(),
            selected: true,
        }]);

        assert!(data.folders[0].selected);
        data.toggle_selected(0);
        assert!(!data.folders[0].selected);
        data.toggle_selected(0);
        assert!(data.folders[0].selected);

        // Out-of-bounds should not panic
        data.toggle_selected(999);

        cleanup(&dir);
    }

    #[test]
    fn test_corrupted_json_falls_back_to_default() {
        let dir = temp_dir();
        let path = dir.join(DATA_FILE);
        fs::write(&path, "not valid json!!!").unwrap();

        let data = AppData::load_from(dir.clone());
        assert_eq!(data.settings.theme, "light");
        assert!(data.folders.is_empty());

        cleanup(&dir);
    }

    #[test]
    fn test_json_structure() {
        let dir = temp_dir();
        let mut data = AppData::load_from(dir.clone());
        data.update_folders(vec![FolderEntry {
            path: "C:\\MyDir".to_string(),
            description: "desc".to_string(),
            selected: true,
        }]);
        data.update_theme("system".to_string());

        let json_str = fs::read_to_string(dir.join(DATA_FILE)).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["settings"]["theme"], "system");
        assert_eq!(parsed["folders"][0]["path"], "C:\\MyDir");
        assert_eq!(parsed["folders"][0]["description"], "desc");
        assert_eq!(parsed["folders"][0]["selected"], true);

        cleanup(&dir);
    }

    // ─── Encryption tests ───────────────────────────────────────────

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "sk-test-key-12345";
        let encrypted = encrypt_string(original);
        assert_ne!(encrypted, original);
        let decrypted = decrypt_string(&encrypted);
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_encrypt_empty_string() {
        assert_eq!(encrypt_string(""), "");
        assert_eq!(decrypt_string(""), "");
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        assert_eq!(decrypt_string("not-valid-base64!!!"), "");
    }

    // ─── LLM config tests ──────────────────────────────────────────

    #[test]
    fn test_llm_config_save_and_read() {
        let dir = temp_dir();
        let mut data = AppData::load_from(dir.clone());

        data.update_llm_config(
            "http://localhost:8080/v1".to_string(),
            "Qwen".to_string(),
            "sk-secret".to_string(),
        );

        // api_key should be encrypted in storage
        assert_ne!(data.llm.api_key, "sk-secret");

        // read_llm_config should decrypt
        let llm = data.read_llm_config();
        assert_eq!(llm.base_url, "http://localhost:8080/v1");
        assert_eq!(llm.model, "Qwen");
        assert_eq!(llm.api_key, "sk-secret");

        // Reload from disk
        let reloaded = AppData::load_from(dir.clone());
        let llm2 = reloaded.read_llm_config();
        assert_eq!(llm2.api_key, "sk-secret");

        cleanup(&dir);
    }

    // ─── Strategy config tests ──────────────────────────────────────

    #[test]
    fn test_strategy_config_defaults() {
        let dir = temp_dir();
        let data = AppData::load_from(dir.clone());
        let s = data.read_strategy_config();
        assert_eq!(s.skip_list, "C:\\$Recycle.Bin;C:\\Windows");
        assert_eq!(s.min_size, 1024);
        assert_eq!(s.skip_recent, 7);
        cleanup(&dir);
    }

    #[test]
    fn test_strategy_config_save_and_read() {
        let dir = temp_dir();
        let mut data = AppData::load_from(dir.clone());
        data.update_strategy_config("C:\\skip1;C:\\skip2".to_string(), 500, 14);

        let reloaded = AppData::load_from(dir.clone());
        let s = reloaded.read_strategy_config();
        assert_eq!(s.skip_list, "C:\\skip1;C:\\skip2");
        assert_eq!(s.min_size, 500);
        assert_eq!(s.skip_recent, 14);

        cleanup(&dir);
    }

    // ─── Append folders test ────────────────────────────────────────

    #[test]
    fn test_append_folders() {
        let dir = temp_dir();
        let mut data = AppData::load_from(dir.clone());
        data.update_folders(vec![FolderEntry {
            path: "C:\\Existing".to_string(),
            description: "".to_string(),
            selected: true,
        }]);

        data.append_folders(vec![FolderEntry {
            path: "C:\\New".to_string(),
            description: "discovered".to_string(),
            selected: true,
        }]);

        assert_eq!(data.folders.len(), 2);
        assert_eq!(data.folders[1].path, "C:\\New");

        cleanup(&dir);
    }

    // ─── Additional encryption tests ────────────────────────────────

    #[test]
    fn test_encrypt_decrypt_long_string() {
        let original = "sk-proj-abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let encrypted = encrypt_string(original);
        assert_ne!(encrypted, original);
        let decrypted = decrypt_string(&encrypted);
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_encrypt_decrypt_special_characters() {
        let original = "p@$w0rd!#%^&*()_+-={}[]|\\:\";<>?,./~`";
        let encrypted = encrypt_string(original);
        let decrypted = decrypt_string(&encrypted);
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_encrypt_decrypt_unicode() {
        let original = "密码测试🔑";
        let encrypted = encrypt_string(original);
        let decrypted = decrypt_string(&encrypted);
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_decrypt_corrupted_ciphertext() {
        // Valid base64 but not valid AES ciphertext
        let result = decrypt_string("AAAAAAAAAAAAAAAAAAAAAA==");
        assert_eq!(result, "");
    }

    // ─── LLM config persistence tests ───────────────────────────────

    #[test]
    fn test_llm_config_api_key_encrypted_on_disk() {
        let dir = temp_dir();
        let mut data = AppData::load_from(dir.clone());
        data.update_llm_config(
            "http://localhost/v1".to_string(),
            "model".to_string(),
            "sk-plaintext-key".to_string(),
        );

        // Read raw JSON from disk and verify api_key is NOT plaintext
        let json_str = fs::read_to_string(dir.join(DATA_FILE)).unwrap();
        assert!(!json_str.contains("sk-plaintext-key"));

        // But read_llm_config decrypts it correctly
        let reloaded = AppData::load_from(dir.clone());
        assert_eq!(reloaded.read_llm_config().api_key, "sk-plaintext-key");

        cleanup(&dir);
    }

    #[test]
    fn test_llm_config_empty_fields() {
        let dir = temp_dir();
        let mut data = AppData::load_from(dir.clone());
        data.update_llm_config(String::new(), String::new(), String::new());

        let llm = data.read_llm_config();
        assert_eq!(llm.base_url, "");
        assert_eq!(llm.model, "");
        assert_eq!(llm.api_key, "");

        cleanup(&dir);
    }

    // ─── Strategy config edge cases ─────────────────────────────────

    #[test]
    fn test_strategy_config_persists_across_reload() {
        let dir = temp_dir();
        let mut data = AppData::load_from(dir.clone());
        data.update_strategy_config("C:\\Windows;C:\\Program Files".to_string(), 2048, 30);

        let reloaded = AppData::load_from(dir.clone());
        let s = reloaded.read_strategy_config();
        assert_eq!(s.skip_list, "C:\\Windows;C:\\Program Files");
        assert_eq!(s.min_size, 2048);
        assert_eq!(s.skip_recent, 30);

        cleanup(&dir);
    }

    // ─── Append folders preserves existing ──────────────────────────

    #[test]
    fn test_append_folders_preserves_selected_state() {
        let dir = temp_dir();
        let mut data = AppData::load_from(dir.clone());
        data.update_folders(vec![FolderEntry {
            path: "C:\\Existing".to_string(),
            description: "keep".to_string(),
            selected: false,
        }]);

        data.append_folders(vec![
            FolderEntry {
                path: "C:\\Selected".to_string(),
                description: "yes".to_string(),
                selected: true,
            },
            FolderEntry {
                path: "C:\\Unselected".to_string(),
                description: "no".to_string(),
                selected: false,
            },
        ]);

        assert_eq!(data.folders.len(), 3);
        assert!(!data.folders[0].selected); // existing preserved
        assert!(data.folders[1].selected);  // new selected
        assert!(!data.folders[2].selected); // new unselected

        // Verify persistence
        let reloaded = AppData::load_from(dir.clone());
        assert_eq!(reloaded.folders.len(), 3);
        assert!(!reloaded.folders[0].selected);
        assert!(reloaded.folders[1].selected);
        assert!(!reloaded.folders[2].selected);

        cleanup(&dir);
    }

    // ─── Backward compatibility: old JSON without new fields ────────

    #[test]
    fn test_load_old_json_without_llm_and_strategy() {
        let dir = temp_dir();
        let old_json = r#"{"settings":{"theme":"dark"},"folders":[]}"#;
        fs::write(dir.join(DATA_FILE), old_json).unwrap();

        let data = AppData::load_from(dir.clone());
        assert_eq!(data.settings.theme, "dark");
        // New fields should have defaults
        let llm = data.read_llm_config();
        assert_eq!(llm.base_url, "");
        let s = data.read_strategy_config();
        assert_eq!(s.min_size, 1024);
        assert_eq!(s.skip_recent, 7);

        cleanup(&dir);
    }
}
