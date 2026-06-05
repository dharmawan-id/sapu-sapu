// Sapu cleaner engine.
// Everything here is deliberately dependency-light: jwalk for parallel walks,
// a direct kernel32 FFI for free space, and std for the rest. The protected-path
// guard is enforced here (server side), so the frontend can never ask Sapu to
// delete something dangerous.

use serde::Serialize;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};

const REPARSE: u32 = 0x400; // FILE_ATTRIBUTE_REPARSE_POINT (junctions / symlinks)
const NO_WINDOW: u32 = 0x0800_0000; // CREATE_NO_WINDOW

// ---- kernel32 FFI: free space without pulling in the `windows` crate ----
#[link(name = "kernel32")]
extern "system" {
    fn GetDiskFreeSpaceExW(
        lp_directory_name: *const u16,
        lp_free_bytes_available_to_caller: *mut u64,
        lp_total_number_of_bytes: *mut u64,
        lp_total_number_of_free_bytes: *mut u64,
    ) -> i32;
}

fn wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

// ---- env helpers ----
fn up() -> String {
    std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string())
}
fn local() -> String {
    std::env::var("LOCALAPPDATA").unwrap_or_else(|_| format!("{}\\AppData\\Local", up()))
}
fn roaming() -> String {
    std::env::var("APPDATA").unwrap_or_else(|_| format!("{}\\AppData\\Roaming", up()))
}

// ---- shared types ----
#[derive(Serialize)]
pub struct DiskInfo {
    drive: String,
    free: u64,
    total: u64,
    used: u64,
    exists: bool,
}

#[derive(Serialize)]
pub struct Entry {
    path: String,
    size: u64,
}

#[derive(Serialize)]
pub struct TypeAgg {
    category: String,
    size: u64,
    count: u64,
}

#[derive(Serialize)]
pub struct Overview {
    drive: String,
    scanned_files: u64,
    top_folders: Vec<Entry>,
    top_files: Vec<Entry>,
    by_type: Vec<TypeAgg>,
}

#[derive(Serialize)]
pub struct CleanTarget {
    id: String,
    label: String,
    tier: String,
    note: String,
    paths: Vec<String>,
    size: u64,
    files: u64,
}

#[derive(Serialize)]
pub struct ProjectArtifact {
    path: String,
    kind: String,
    size: u64,
    modified_days: i64,
    git_dirty: bool,
    safe: bool,
}

#[derive(Serialize)]
pub struct CleanResult {
    freed: u64,
    deleted: u64,
    skipped: u64,
    protected_blocked: u64,
    free_before: u64,
    free_after: u64,
}

// ---- disk free ----
#[tauri::command]
pub fn disk_info(drive: String) -> DiskInfo {
    let letter = drive.trim().trim_end_matches('\\').trim_end_matches(':');
    let root = format!("{}:\\", letter);
    let w = wide(&root);
    let mut avail: u64 = 0;
    let mut total: u64 = 0;
    let mut free: u64 = 0;
    let ok = unsafe { GetDiskFreeSpaceExW(w.as_ptr(), &mut avail, &mut total, &mut free) };
    if ok == 0 {
        DiskInfo { drive: letter.to_string(), free: 0, total: 0, used: 0, exists: false }
    } else {
        DiskInfo {
            drive: letter.to_string(),
            free: avail,
            total,
            used: total.saturating_sub(avail),
            exists: true,
        }
    }
}

fn free_drive(letter: &str) -> u64 {
    disk_info(letter.to_string()).free
}

// ---- size + delete primitives (resilient, reparse-aware) ----
fn is_reparse(md: &std::fs::Metadata) -> bool {
    md.file_attributes() & REPARSE != 0
}

fn dir_size(path: &Path) -> (u64, u64) {
    let mut bytes = 0u64;
    let mut files = 0u64;
    if !path.exists() {
        return (0, 0);
    }
    for entry in jwalk::WalkDir::new(path).skip_hidden(false).follow_links(false) {
        if let Ok(e) = entry {
            if let Ok(md) = e.metadata() {
                if md.is_file() && !is_reparse(&md) {
                    bytes += md.len();
                    files += 1;
                }
            }
        }
    }
    (bytes, files)
}

// Delete everything under `path`. With remove_root, also remove `path` itself.
// Locked / in-use files are skipped, never fatal. Returns (deleted, skipped).
fn purge(path: &Path, remove_root: bool) -> (u64, u64) {
    let mut deleted = 0u64;
    let mut skipped = 0u64;
    if !path.exists() {
        return (0, 0);
    }
    let mut dirs: Vec<PathBuf> = Vec::new();
    for entry in jwalk::WalkDir::new(path).skip_hidden(false).follow_links(false) {
        let e = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let p = e.path();
        if p == path {
            continue;
        }
        let reparse = e
            .metadata()
            .ok()
            .map(|m| is_reparse(&m))
            .unwrap_or(false);
        if e.file_type().is_dir() && !reparse {
            dirs.push(p);
        } else {
            match std::fs::remove_file(&p) {
                Ok(_) => deleted += 1,
                Err(_) => {
                    if std::fs::remove_dir(&p).is_ok() {
                        deleted += 1;
                    } else {
                        skipped += 1;
                    }
                }
            }
        }
    }
    dirs.sort_by_key(|d| Reverse(d.components().count()));
    for d in dirs {
        let _ = std::fs::remove_dir(&d);
    }
    if remove_root {
        let _ = std::fs::remove_dir(path);
    }
    (deleted, skipped)
}

// ---- protected-path guard (server side, non-negotiable) ----
fn is_protected(path: &Path) -> bool {
    let s = path.to_string_lossy().to_lowercase().replace('/', "\\");

    // never delete a drive root or near-root path
    if path.components().count() <= 2 {
        return true;
    }

    let needles = [
        "\\windows\\installer",
        "\\windows\\system32",
        "\\windows\\winsxs",
        "\\appdata\\roaming\\claude\\vm_bundles",
        "\\appdata\\roaming\\npm", // global CLI prefix (claude, gemini, vercel)
        "\\.ssh",
        "\\.aws",
        "\\.kube",
        "\\.gnupg",
        "\\devtools\\rust", // our own toolchain, do not nuke
    ];
    if needles.iter().any(|n| s.contains(n)) {
        return true;
    }

    // browser User Data is protected UNLESS the path is a known cache subfolder
    let is_browser_userdata =
        s.contains("\\google\\chrome\\user data") || s.contains("\\brave-browser\\user data");
    if is_browser_userdata {
        let cache_markers = [
            "\\cache",
            "code cache",
            "gpucache",
            "shadercache",
            "grshadercache",
            "cachestorage",
            "scriptcache",
            "dawncache",
            "dawngraphitecache",
            "dawnwebgpucache",
            "graphitedawncache",
        ];
        let looks_like_cache = cache_markers.iter().any(|m| s.contains(m));
        if !looks_like_cache {
            return true;
        }
    }
    false
}

// ---- overview scan (disk analyzer) ----
fn category_for(ext: &str) -> &'static str {
    match ext {
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" => "Video",
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" | "heic" | "raw" | "psd" => "Image",
        "zip" | "rar" | "7z" | "tar" | "gz" | "xz" | "bz2" | "zst" => "Archive",
        "iso" | "img" | "vhd" | "vhdx" | "vmdk" | "wim" => "Disk image",
        "exe" | "msi" | "msix" | "appx" | "msp" => "Installer",
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => "Audio",
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "csv" => "Document",
        "rs" | "js" | "ts" | "py" | "json" | "lock" | "node" | "rlib" | "pdb" | "obj" | "o"
        | "rmeta" | "d" => "Code / build",
        _ => "Other",
    }
}

fn accumulate_folders(root: &Path, file: &Path, sz: u64, cap: usize, map: &mut HashMap<String, u64>) {
    if let Ok(rel) = file.strip_prefix(root) {
        let comps: Vec<_> = rel.components().collect();
        let dir_count = comps.len().saturating_sub(1); // last component is the file
        let upto = dir_count.min(cap);
        let mut acc = root.to_path_buf();
        for c in comps.iter().take(upto) {
            acc.push(c.as_os_str());
            *map.entry(acc.to_string_lossy().to_string()).or_insert(0) += sz;
        }
    }
}

#[tauri::command]
pub fn scan_overview(drive: String, topn: usize) -> Overview {
    let letter = drive.trim().trim_end_matches('\\').trim_end_matches(':');
    let root_str = format!("{}:\\", letter);
    let root = PathBuf::from(&root_str);
    let cap = 4usize;
    let n = topn.max(1);

    let mut folder: HashMap<String, u64> = HashMap::new();
    let mut by_type: HashMap<&'static str, (u64, u64)> = HashMap::new();
    let mut files = 0u64;
    let mut heap: BinaryHeap<Reverse<(u64, String)>> = BinaryHeap::new();

    for entry in jwalk::WalkDir::new(&root).skip_hidden(false).follow_links(false) {
        if let Ok(e) = entry {
            if let Ok(md) = e.metadata() {
                if md.is_file() && !is_reparse(&md) {
                    let sz = md.len();
                    files += 1;
                    let p = e.path();

                    let ext = p
                        .extension()
                        .and_then(|x| x.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let cat = category_for(&ext);
                    let ent = by_type.entry(cat).or_insert((0, 0));
                    ent.0 += sz;
                    ent.1 += 1;

                    let ps = p.to_string_lossy().to_string();
                    if heap.len() < n {
                        heap.push(Reverse((sz, ps)));
                    } else if let Some(Reverse((min, _))) = heap.peek() {
                        if sz > *min {
                            heap.pop();
                            heap.push(Reverse((sz, ps)));
                        }
                    }

                    accumulate_folders(&root, &p, sz, cap, &mut folder);
                }
            }
        }
    }

    let mut top_folders: Vec<Entry> = folder
        .into_iter()
        .map(|(path, size)| Entry { path, size })
        .collect();
    top_folders.sort_by(|a, b| b.size.cmp(&a.size));
    top_folders.truncate(n);

    let mut top_files: Vec<Entry> = heap
        .into_sorted_vec()
        .into_iter()
        .rev()
        .map(|Reverse((size, path))| Entry { path, size })
        .collect();
    top_files.truncate(n);

    let mut bt: Vec<TypeAgg> = by_type
        .into_iter()
        .map(|(category, (size, count))| TypeAgg {
            category: category.to_string(),
            size,
            count,
        })
        .collect();
    bt.sort_by(|a, b| b.size.cmp(&a.size));

    Overview {
        drive: letter.to_string(),
        scanned_files: files,
        top_folders,
        top_files,
        by_type: bt,
    }
}

// ---- green-tier cache targets ----
fn browser_cache_paths() -> Vec<String> {
    let local = local();
    let roots = [
        format!("{}\\Google\\Chrome\\User Data", local),
        format!("{}\\BraveSoftware\\Brave-Browser\\User Data", local),
    ];
    let cache_names = [
        "Cache",
        "Code Cache",
        "GPUCache",
        "ShaderCache",
        "GrShaderCache",
        "GraphiteDawnCache",
        "DawnCache",
        "DawnGraphiteCache",
        "DawnWebGPUCache",
        "Service Worker\\CacheStorage",
        "Service Worker\\ScriptCache",
    ];
    let mut out = Vec::new();
    for r in roots.iter() {
        let rp = Path::new(r);
        if !rp.exists() {
            continue;
        }
        let mut profiles: Vec<PathBuf> = vec![rp.to_path_buf()];
        if let Ok(rd) = std::fs::read_dir(rp) {
            for ent in rd.flatten() {
                let name = ent.file_name().to_string_lossy().to_string();
                if name == "Default"
                    || name.starts_with("Profile ")
                    || name == "System Profile"
                    || name == "Guest Profile"
                {
                    profiles.push(ent.path());
                }
            }
        }
        for pf in profiles {
            for cn in cache_names.iter() {
                let cp = pf.join(cn);
                if cp.exists() {
                    out.push(cp.to_string_lossy().to_string());
                }
            }
        }
    }
    out
}

fn cargo_cache_paths() -> Vec<String> {
    let ch = std::env::var("CARGO_HOME").unwrap_or_else(|_| format!("{}\\.cargo", up()));
    vec![
        format!("{}\\registry\\cache", ch),
        format!("{}\\registry\\src", ch),
        format!("{}\\git\\checkouts", ch),
        format!("{}\\git\\db", ch),
    ]
}

#[tauri::command]
pub fn list_clean_targets() -> Vec<CleanTarget> {
    let up = up();
    let local = local();
    let roaming = roaming();

    let defs: Vec<(&str, &str, &str, Vec<String>)> = vec![
        (
            "uv",
            "uv cache (Python)",
            "Build and download cache. Regenerates on next install. Logical size can be hardlink-inflated, so real reclaim may be lower.",
            vec![
                format!("{}\\uv\\cache\\builds-v0", local),
                format!("{}\\uv\\cache\\archive-v0", local),
                format!("{}\\uv\\cache\\git-v0", local),
            ],
        ),
        (
            "npm",
            "npm cache",
            "Package cache. npm re-downloads as needed.",
            vec![format!("{}\\npm-cache", local), format!("{}\\.npm-cache", up)],
        ),
        (
            "pip",
            "pip cache (Python)",
            "Wheel cache. Re-downloaded on next install.",
            vec![format!("{}\\pip\\cache", local)],
        ),
        (
            "cargo",
            "cargo registry + git cache (Rust)",
            "Downloaded crates. Re-fetched on next build.",
            cargo_cache_paths(),
        ),
        (
            "temp",
            "Temp (User + Windows)",
            "Temporary files. Locked files in use are skipped.",
            vec![format!("{}\\Temp", local), "C:\\Windows\\Temp".to_string()],
        ),
        (
            "browser",
            "Browser cache (Chrome + Brave)",
            "Web cache only. Profiles, passwords and bookmarks are untouched.",
            browser_cache_paths(),
        ),
        (
            "hf",
            "HuggingFace cache",
            "Downloaded ML models. Re-downloaded when used again.",
            vec![format!("{}\\.cache\\huggingface", up)],
        ),
        (
            "vscode",
            "VS Code cache",
            "Editor cache and cached data. Settings and extensions untouched.",
            vec![
                format!("{}\\Code\\Cache", roaming),
                format!("{}\\Code\\CachedData", roaming),
                format!("{}\\Code\\Code Cache", roaming),
                format!("{}\\Code\\GPUCache", roaming),
            ],
        ),
    ];

    let mut out = Vec::new();
    for (id, label, note, paths) in defs.into_iter() {
        let existing: Vec<String> = paths
            .into_iter()
            .filter(|p| Path::new(p).exists())
            .collect();
        let mut size = 0u64;
        let mut files = 0u64;
        for p in &existing {
            let (b, f) = dir_size(Path::new(p));
            size += b;
            files += f;
        }
        out.push(CleanTarget {
            id: id.to_string(),
            label: label.to_string(),
            tier: "green".to_string(),
            note: note.to_string(),
            paths: existing,
            size,
            files,
        });
    }
    out
}

// ---- yellow-tier project artifacts ----
fn find_artifacts(dir: &Path, depth: usize, max_depth: usize, names: &[&str], out: &mut Vec<PathBuf>) {
    if depth > max_depth {
        return;
    }
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for ent in rd.flatten() {
        let md = match ent.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if md.is_dir() && !is_reparse(&md) {
            let name = ent.file_name().to_string_lossy().to_lowercase();
            if names.contains(&name.as_str()) {
                out.push(ent.path()); // matched: record and do not descend
            } else {
                find_artifacts(&ent.path(), depth + 1, max_depth, names, out);
            }
        }
    }
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start);
    while let Some(d) = cur {
        if d.join(".git").exists() {
            return Some(d.to_path_buf());
        }
        cur = d.parent();
    }
    None
}

fn git_dirty_root(root: &Path) -> bool {
    use std::os::windows::process::CommandExt;
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root.as_os_str())
        .arg("status")
        .arg("--porcelain")
        .creation_flags(NO_WINDOW)
        .output();
    match out {
        Ok(o) => !o.stdout.is_empty(),
        Err(_) => false,
    }
}

fn modified_days(md: &std::fs::Metadata) -> i64 {
    match md.modified() {
        Ok(t) => match std::time::SystemTime::now().duration_since(t) {
            Ok(d) => (d.as_secs() / 86_400) as i64,
            Err(_) => 0,
        },
        Err(_) => -1,
    }
}

#[tauri::command]
pub fn scan_projects(root: String, depth: usize) -> Vec<ProjectArtifact> {
    let names = [
        "node_modules",
        "target",
        "dist",
        "build",
        ".next",
        "__pycache__",
        ".turbo",
        ".parcel-cache",
    ];
    let start = PathBuf::from(&root);
    let mut found: Vec<PathBuf> = Vec::new();
    if start.exists() {
        find_artifacts(&start, 0, depth.max(1), &names, &mut found);
    }

    let mut git_cache: HashMap<PathBuf, bool> = HashMap::new();
    let mut out = Vec::new();
    for p in found {
        let (size, _files) = dir_size(&p);
        let mdays = std::fs::symlink_metadata(&p)
            .ok()
            .as_ref()
            .map(modified_days)
            .unwrap_or(-1);
        let parent = p.parent().map(|x| x.to_path_buf()).unwrap_or_else(|| p.clone());
        let dirty = match find_git_root(&parent) {
            Some(gr) => *git_cache.entry(gr.clone()).or_insert_with(|| git_dirty_root(&gr)),
            None => false,
        };
        let kind = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let recency_ok = mdays >= 30;
        let safe = recency_ok && !dirty;
        out.push(ProjectArtifact {
            path: p.to_string_lossy().to_string(),
            kind,
            size,
            modified_days: mdays,
            git_dirty: dirty,
            safe,
        });
    }
    out.sort_by(|a, b| b.size.cmp(&a.size));
    out
}

// ---- delete commands (free-space delta = real reclaim) ----
#[tauri::command]
pub fn clean_paths(paths: Vec<String>) -> CleanResult {
    run_delete(paths, false)
}

#[tauri::command]
pub fn delete_projects(paths: Vec<String>) -> CleanResult {
    run_delete(paths, true)
}

fn run_delete(paths: Vec<String>, remove_root: bool) -> CleanResult {
    let cb = free_drive("C");
    let db = free_drive("D");
    let mut deleted = 0u64;
    let mut skipped = 0u64;
    let mut blocked = 0u64;
    for ps in &paths {
        let p = PathBuf::from(ps);
        if is_protected(&p) {
            blocked += 1;
            continue;
        }
        let (d, s) = purge(&p, remove_root);
        deleted += d;
        skipped += s;
    }
    let ca = free_drive("C");
    let da = free_drive("D");
    let freed = ca.saturating_sub(cb).saturating_add(da.saturating_sub(db));
    CleanResult {
        freed,
        deleted,
        skipped,
        protected_blocked: blocked,
        free_before: cb.saturating_add(db),
        free_after: ca.saturating_add(da),
    }
}

#[tauri::command]
pub fn empty_recycle_bin() -> CleanResult {
    use std::os::windows::process::CommandExt;
    let cb = free_drive("C");
    let db = free_drive("D");
    let _ = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Clear-RecycleBin -Force -ErrorAction SilentlyContinue",
        ])
        .creation_flags(NO_WINDOW)
        .status();
    let ca = free_drive("C");
    let da = free_drive("D");
    let freed = ca.saturating_sub(cb).saturating_add(da.saturating_sub(db));
    CleanResult {
        freed,
        deleted: 0,
        skipped: 0,
        protected_blocked: 0,
        free_before: cb.saturating_add(db),
        free_after: ca.saturating_add(da),
    }
}
