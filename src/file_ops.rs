use std::path::Path;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND};

#[derive(Debug, PartialEq)]
pub enum DeleteResult {
    PathNotFound,
    Succeed,
    Failed,
}

/// Calculate the total size of a directory recursively using the Win32
/// `FindFirstFileExW` / `FindNextFileW` API.
///
/// Compared to the pure-Rust `walkdir` approach this avoids:
///   - short-name lookups  (`FindExInfoBasic`)
///   - small I/O buffers   (`FIND_FIRST_EX_LARGE_FETCH`)
///
/// Returns the size in bytes, or 0 if the path doesn't exist / is
/// inaccessible.
#[cfg(windows)]
pub fn calculate_dir_size(path: &str) -> u64 {
    use std::collections::VecDeque;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::{
        FindClose, FindFirstFileExW, FindNextFileW,
        FindExInfoBasic, FindExSearchNameMatch,
        FIND_FIRST_EX_LARGE_FETCH,
        FILE_ATTRIBUTE_DIRECTORY,
        WIN32_FIND_DATAW,
    };
    use windows::core::PCWSTR;

    let root = Path::new(path);
    if !root.exists() || !root.is_dir() {
        return 0;
    }

    let mut total: u64 = 0;
    let mut dirs: VecDeque<String> = VecDeque::new();
    dirs.push_back(path.to_string());

    while let Some(dir) = dirs.pop_front() {
        // Build search pattern: dir\*
        let pattern = format!("{}\\*", dir.trim_end_matches('\\'));
        let wide: Vec<u16> = OsStr::new(&pattern)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut find_data = WIN32_FIND_DATAW::default();

        let handle = unsafe {
            FindFirstFileExW(
                PCWSTR(wide.as_ptr()),
                FindExInfoBasic,
                &mut find_data as *mut _ as *mut _,
                FindExSearchNameMatch,
                None,
                FIND_FIRST_EX_LARGE_FETCH,
            )
        };

        let handle = match handle {
            Ok(h) => h,
            Err(_) => continue,
        };

        loop {
            // Decode file name
            let name_len = find_data.cFileName.iter().position(|&c| c == 0).unwrap_or(find_data.cFileName.len());
            let name = String::from_utf16_lossy(&find_data.cFileName[..name_len]);

            // Skip "." and ".."
            if name != "." && name != ".." {
                let is_dir = (find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY.0) != 0;
                if is_dir {
                    dirs.push_back(format!("{}\\{}", dir.trim_end_matches('\\'), name));
                } else {
                    let file_size = ((find_data.nFileSizeHigh as u64) << 32)
                        | (find_data.nFileSizeLow as u64);
                    total += file_size;
                }
            }

            let ok = unsafe { FindNextFileW(handle, &mut find_data) };
            if ok.is_err() {
                break;
            }
        }

        unsafe { let _ = FindClose(handle); }
    }

    total
}

/// Fallback for non-Windows (used in cross-compilation / CI).
#[cfg(not(windows))]
pub fn calculate_dir_size(path: &str) -> u64 {
    use walkdir::WalkDir;
    let path = Path::new(path);
    if !path.exists() || !path.is_dir() {
        return 0;
    }
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Format bytes into human-readable string (e.g., "1.5 G", "300 M", "12 k")
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if bytes >= TB {
        format!("{:.1} T", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} k", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Move a directory to the Windows Recycle Bin using SHFileOperationW.
/// Returns Ok(()) on success, Err(message) on failure.
#[cfg(windows)]
pub fn move_to_recycle_bin(path: &str) -> DeleteResult {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::UI::Shell::{SHFileOperationW, SHFILEOPSTRUCTW, FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_SILENT};
    use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND};

    // SHFileOperationW requires double-null-terminated string
    let wide: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .chain(std::iter::once(0))
        .collect();

    let mut op = SHFILEOPSTRUCTW {
        pFrom: windows::core::PCWSTR(wide.as_ptr()),
        wFunc: FO_DELETE,
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT).0 as u16,
        ..Default::default()
    };

    let result = unsafe { SHFileOperationW(&mut op) };
    if result == 0 {
        DeleteResult::Succeed
    } else if result == ERROR_FILE_NOT_FOUND.0 as i32 || result == ERROR_PATH_NOT_FOUND.0 as i32 {
        DeleteResult::PathNotFound
    } else {
        DeleteResult::Failed
    }
    // if result == 0 {
    //     Ok(())
    // } else {
    //     Err(format!("SHFileOperationW failed with code: {}", result))
    // }
}

/// Permanently delete a directory (no recycle bin).
/// Returns Ok(()) on success, Err(message) on failure.
#[cfg(windows)]
pub fn permanently_delete(path: &str) -> DeleteResult {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::UI::Shell::{SHFileOperationW, SHFILEOPSTRUCTW, FO_DELETE, FOF_NOCONFIRMATION, FOF_SILENT};

    let wide: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .chain(std::iter::once(0))
        .collect();

    let mut op = SHFILEOPSTRUCTW {
        pFrom: windows::core::PCWSTR(wide.as_ptr()),
        wFunc: FO_DELETE,
        fFlags: (FOF_NOCONFIRMATION | FOF_SILENT).0 as u16, // No FOF_ALLOWUNDO = permanent delete
        ..Default::default()
    };

    let result = unsafe { SHFileOperationW(&mut op) };
    if result == 0 {
        DeleteResult::Succeed
    } else if result == ERROR_FILE_NOT_FOUND.0 as i32 || result == ERROR_PATH_NOT_FOUND.0 as i32 {
        DeleteResult::PathNotFound
    } else {
        DeleteResult::Failed
    }
    // if result == 0 {
    //     Ok(())
    // } else {
    //     Err(format!("SHFileOperationW failed with code: {}", result))
    // }
}

#[cfg(not(windows))]
pub fn move_to_recycle_bin(_path: &str) -> Result<(), String> {
    Err("Recycle bin is only supported on Windows".to_string())
}

#[cfg(not(windows))]
pub fn permanently_delete(_path: &str) -> Result<(), String> {
    Err("Permanent delete is only supported on Windows".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("clearmyc_fileops_{}_{}", name, std::process::id()));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn cleanup(dir: &std::path::PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    // ─── format_size tests ───────────────────────────────────────────

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(format_size(1024), "1.0 k");
        assert_eq!(format_size(1536), "1.5 k");
        assert_eq!(format_size(10240), "10.0 k");
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size(1024 * 1024), "1.0 M");
        assert_eq!(format_size(300 * 1024 * 1024), "300.0 M");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 G");
        assert_eq!(format_size(1024u64 * 1024 * 1024 * 2 + 1024 * 1024 * 512), "2.5 G");
    }

    #[test]
    fn test_format_size_tb() {
        assert_eq!(format_size(1024u64 * 1024 * 1024 * 1024), "1.0 T");
    }

    // ─── calculate_dir_size tests ────────────────────────────────────

    #[test]
    fn test_calculate_dir_size_nonexistent() {
        assert_eq!(calculate_dir_size("Z:\\nonexistent_path_12345"), 0);
    }

    #[test]
    fn test_calculate_dir_size_empty_dir() {
        let dir = temp_dir("empty");
        assert_eq!(calculate_dir_size(dir.to_str().unwrap()), 0);
        cleanup(&dir);
    }

    #[test]
    fn test_calculate_dir_size_with_files() {
        let dir = temp_dir("withfiles");

        // Create files with known sizes
        fs::write(dir.join("a.txt"), "hello").unwrap(); // 5 bytes
        fs::write(dir.join("b.txt"), "world!").unwrap(); // 6 bytes

        let sub = dir.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("c.txt"), "12345678").unwrap(); // 8 bytes

        let size = calculate_dir_size(dir.to_str().unwrap());
        assert_eq!(size, 5 + 6 + 8);

        cleanup(&dir);
    }

    #[test]
    fn test_calculate_dir_size_file_path_returns_zero() {
        let dir = temp_dir("filepath");
        let file = dir.join("test.txt");
        fs::write(&file, "data").unwrap();

        // Passing a file path (not a dir) should return 0
        assert_eq!(calculate_dir_size(file.to_str().unwrap()), 0);

        cleanup(&dir);
    }

    // ─── recycle bin / permanent delete tests ────────────────────────

    #[cfg(windows)]
    #[test]
    fn test_move_to_recycle_bin() {
        let dir = temp_dir("recycle");
        let target = dir.join("to_delete");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("file.txt"), "data").unwrap();

        let result = move_to_recycle_bin(target.to_str().unwrap());
        assert_eq!(result, DeleteResult::Succeed);
        assert!(!target.exists());

        cleanup(&dir);
    }

    #[cfg(windows)]
    #[test]
    fn test_permanently_delete() {
        let dir = temp_dir("permdelete");
        let target = dir.join("to_delete");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("file.txt"), "data").unwrap();

        let result = permanently_delete(target.to_str().unwrap());
        assert_eq!(result, DeleteResult::Succeed);
        assert!(!target.exists());

        cleanup(&dir);
    }

    #[cfg(windows)]
    #[test]
    fn test_delete_nonexistent_path() {
        // Deleting a path that doesn't exist — SHFileOperation returns error
        let result = move_to_recycle_bin("Z:\\this_path_does_not_exist_12345");
        // This may succeed or fail depending on Windows behavior; just ensure no panic
        let _ = result;
    }
}
