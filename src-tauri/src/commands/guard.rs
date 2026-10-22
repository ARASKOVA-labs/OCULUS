//! Checks on what the interface sends over IPC. The strict CSP keeps the webview to our own code,
//! but if that ever failed, these keep a hostile page from reading or overwriting files the user
//! never chose: secrets in hidden folders in the home directory, shell start-up files, anything
//! through a crafted document id.

use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

pub const MAX_FILE_SIZE_BYTES: u64 = 250 * 1024 * 1024; // 250 MB

/// A document id names a file in the cache: letters, digits, `-` and `_` only.
pub fn document_id(id: &str) -> Result<&str, String> {
    let ok = !id.is_empty()
        && id.len() <= 128
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok {
        Ok(id)
    } else {
        Err("Invalid document id".to_string())
    }
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .and_then(|h| h.canonicalize().ok())
}

/// Whether a path is a hidden file or folder at the top of the home directory (`~/.ssh`,
/// `~/.config`, `~/.zshrc`, ...): where keys and settings live, never documents.
fn hidden_in_home(p: &Path) -> bool {
    let Some(home) = home() else { return false };
    let Ok(rest) = p.strip_prefix(&home) else {
        return false;
    };
    matches!(rest.components().next(), Some(Component::Normal(s)) if s.to_string_lossy().starts_with('.'))
}

/// A document to read: an existing regular file (links resolved), not in a hidden folder in the
/// home directory, within the size limit.
pub fn readable_file(path: &str) -> Result<PathBuf, String> {
    let p = Path::new(path)
        .canonicalize()
        .map_err(|e| format!("Cannot open '{}': {}", path, e))?;
    let meta = std::fs::metadata(&p).map_err(|e| format!("Cannot open '{}': {}", path, e))?;
    if !meta.is_file() {
        return Err(format!("'{}' is not a file", path));
    }
    if hidden_in_home(&p) {
        return Err(format!(
            "OCULUS does not open files in hidden folders: '{}'",
            path
        ));
    }
    if meta.len() > MAX_FILE_SIZE_BYTES {
        return Err(format!(
            "File size exceeds safety limit of 250MB (size: {} MB)",
            meta.len() / (1024 * 1024)
        ));
    }
    Ok(p)
}

/// Where an export may be written: an absolute path ending in the format's extension (so it can
/// never replace a script, a key or a start-up file), not a link, not in a hidden folder in the
/// home directory.
pub fn export_target(path: &Path, format: &str) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(format!(
            "Export path must be absolute: '{}'",
            path.display()
        ));
    }
    let ext_ok = path
        .extension()
        .is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case(format));
    if !ext_ok {
        return Err(format!(
            "Export path must end in .{}: '{}'",
            format,
            path.display()
        ));
    }
    if path
        .symlink_metadata()
        .is_ok_and(|m| m.file_type().is_symlink())
    {
        return Err(format!("Export path is a link: '{}'", path.display()));
    }
    // the nearest existing folder, links resolved, decides where the file really lands
    let real = path
        .ancestors()
        .skip(1)
        .find_map(|a| a.canonicalize().ok())
        .unwrap_or_default();
    if hidden_in_home(&real) || hidden_in_home(path) {
        return Err(format!(
            "OCULUS does not write into hidden folders: '{}'",
            path.display()
        ));
    }
    Ok(path.to_path_buf())
}

/// What a converted file may be called when it is saved beside its original.
const OUTPUT_EXTS: &[&str] = &[
    "pdf", "docx", "xlsx", "pptx", "jpg", "png", "zip", "epub", "mobi", "avif", "txt",
];
const MAX_OUTPUT_BYTES: usize = 1024 * 1024 * 1024; // 1 GB

/// Saves a converted file in the folder of the document it came from. `source` must be a
/// document OCULUS could open; `name` a plain file name (no folders) ending in an output format.
/// An existing file is never replaced: "quote.docx" becomes "quote (2).docx", and so on.
pub fn save_beside(source: &str, name: &str, bytes: &[u8]) -> Result<PathBuf, String> {
    let src = readable_file(source)?;
    let dir = src
        .parent()
        .ok_or_else(|| format!("No folder for '{}'", source))?;
    let plain = !name.is_empty()
        && name.len() <= 200
        && !name.starts_with('.')
        && !name.contains(['/', '\\', ':', '\0'])
        && name != ".."
        && Path::new(name).file_name().is_some_and(|f| f == name);
    if !plain {
        return Err(format!("Not a plain file name: '{}'", name));
    }
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() && OUTPUT_EXTS.contains(&e.to_ascii_lowercase().as_str()) => {
            (s, e)
        }
        _ => return Err(format!("Not an output OCULUS makes: '{}'", name)),
    };
    if bytes.len() > MAX_OUTPUT_BYTES {
        return Err("The converted file is larger than 1 GB".to_string());
    }
    for k in 1..1000 {
        let candidate = if k == 1 {
            name.to_string()
        } else {
            format!("{} ({}).{}", stem, k, ext)
        };
        let target = export_target(&dir.join(&candidate), &ext.to_ascii_lowercase())?;
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
        {
            Ok(mut f) => {
                use std::io::Write;
                f.write_all(bytes)
                    .map_err(|e| format!("Could not save '{}': {}", target.display(), e))?;
                return Ok(target);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("Could not save '{}': {}", target.display(), e)),
        }
    }
    Err(format!("Too many files called '{}' in that folder", name))
}

/// Undoes `encodeURIComponent` (the interface sends names in headers, which must be ASCII).
pub fn percent_decode(s: &str) -> Result<String, String> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let hex = s
                .get(i + 1..i + 3)
                .and_then(|h| u8::from_str_radix(h, 16).ok())
                .ok_or("Bad percent-encoding")?;
            out.push(hex);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| "Bad percent-encoding".to_string())
}

/// This run's private working folder: a fresh, randomly named folder in the temp directory that
/// only this user can open. It holds copies of documents being read, and is removed on exit.
static WORK_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn work_dir() -> Result<&'static Path, String> {
    if let Some(d) = WORK_DIR.get() {
        return Ok(d);
    }
    let dir = std::env::temp_dir().join(format!("oculus-{}", uuid::Uuid::new_v4()));
    let mut builder = std::fs::DirBuilder::new();
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder
        .create(&dir)
        .map_err(|e| format!("Cannot create a working folder: {}", e))?;
    Ok(WORK_DIR.get_or_init(|| dir))
}

/// Removes this run's working folder, and the shared cache older versions left behind.
pub fn clean_up() {
    if let Some(d) = WORK_DIR.get() {
        let _ = std::fs::remove_dir_all(d);
    }
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join("oculus_cache"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_ids_cannot_name_other_files() {
        assert!(document_id("1729-0-p3").is_ok());
        assert!(document_id("scan-1729").is_ok());
        for bad in ["", "../x", "..", "a/b", "a\\b", "/etc/passwd", "C:x", "a.b"] {
            assert!(document_id(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn exports_keep_their_extension_and_stay_out_of_hidden_folders() {
        let tmp = std::env::temp_dir().canonicalize().unwrap();
        assert!(export_target(&tmp.join("a.txt"), "txt").is_ok());
        assert!(export_target(&tmp.join("a.TXT"), "txt").is_ok());
        assert!(export_target(&tmp.join("a.sh"), "txt").is_err());
        assert!(export_target(&tmp.join("a"), "txt").is_err());
        assert!(export_target(Path::new("a.txt"), "txt").is_err());
        if let Some(h) = home() {
            assert!(export_target(&h.join(".ssh").join("x.txt"), "txt").is_err());
            assert!(export_target(&h.join(".zshrc.txt"), "txt").is_err());
            assert!(readable_file(&h.join(".ssh/id_ed25519").to_string_lossy()).is_err());
        }
    }

    #[test]
    fn reads_only_regular_files() {
        let tmp = std::env::temp_dir();
        assert!(readable_file(&tmp.to_string_lossy()).is_err());
        assert!(readable_file("/definitely/not/here.pdf").is_err());
    }

    #[test]
    fn saves_beside_the_original_and_never_replaces() {
        let dir = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("oculus-beside-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let src = dir.join("quote.pdf");
        std::fs::write(&src, b"%PDF-1.7").unwrap();
        let src = src.to_string_lossy().into_owned();

        let first = save_beside(&src, "quote.docx", b"one").unwrap();
        let second = save_beside(&src, "quote.docx", b"two").unwrap();
        assert_eq!(first, dir.join("quote.docx"));
        assert_eq!(second, dir.join("quote (2).docx"));
        assert_eq!(std::fs::read(&first).unwrap(), b"one");

        for bad in [
            "../x.docx",
            "a/b.docx",
            ".hidden.docx",
            "run.sh",
            "noext",
            "",
            ".docx",
        ] {
            assert!(save_beside(&src, bad, b"x").is_err(), "{bad}");
        }
        assert!(save_beside(&dir.join("missing.pdf").to_string_lossy(), "a.docx", b"x").is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn decodes_percent_encoded_names() {
        assert_eq!(
            percent_decode("quote%20(1).docx").unwrap(),
            "quote (1).docx"
        );
        assert_eq!(
            percent_decode("%E0%B4%AE%E0%B4%B2.pdf").unwrap(),
            "\u{0D2E}\u{0D32}.pdf"
        );
        assert!(percent_decode("bad%zz").is_err());
        assert!(percent_decode("bad%").is_err());
    }

    #[test]
    fn the_work_dir_is_private() {
        let d = work_dir().unwrap();
        assert!(d.is_dir());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(d).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
    }
}
