# Sapu Sapu design spec

Status: approved 2026-06-06. v1.

## Goal

A Windows desktop cleaner that does two jobs honestly: map where disk space goes on `C:` and `D:`, and clean safe caches with a preview-first, real-reclaim workflow. Showcase quality, neo-brutalist, timber accent.

## Approach (decided)

- **Stack**: Tauri 2, Rust backend, vanilla HTML/CSS/JS frontend rendered in WebView2. No framework, no bundler.
- **Toolchain**: Rust GNU toolchain so it builds without Visual Studio. Installed on `D:` (`RUSTUP_HOME`, `CARGO_HOME`) to keep `C:` clean.
- **No admin** for v1: all targets live in the user profile or are reachable without elevation.

## Two modes

1. **Overview** (space analyzer): scan a whole drive with a parallel walk (`jwalk`), report the biggest folders (rolled up to depth 4), the biggest files (bounded heap), and totals by file type. MFT-direct scan is roadmap, not v1, because it needs admin and raw NTFS parsing.
2. **Clean** (risk-tiered):
   - Green: uv, npm, pip, cargo, Temp, browser, HuggingFace, VS Code caches. Contents deleted, folder kept.
   - Yellow: `node_modules`, `target`, `dist`, `build`, `.next`, `__pycache__` under a chosen root. Recency guard (30 days) and git-dirty check decide the safe flag. Whole folder removed.
   - Protected: Installer store, active `vm_bundles`, `Roaming\npm`, browser profiles, SSH/cloud keys. Never deleted; guard enforced in Rust.

## Safety

Preview always, process-aware skip of locked files, git-aware and recency-aware Yellow flags, and reclaim measured from the real `C:`+`D:` free-space delta rather than logical sizes (hardlinks make logical sizes lie).

## Commands (Rust to JS bridge)

`disk_info(drive)`, `scan_overview(drive, topn)`, `list_clean_targets()`, `scan_projects(root, depth)`, `clean_paths(paths)`, `delete_projects(paths)`, `empty_recycle_bin()`.

## Layout

```
Clean/
  ui/        index.html, styles.css, app.js
  src-tauri/ Cargo.toml, tauri.conf.json, build.rs, capabilities/, icons/, src/{main,cleaner}.rs
  scripts/   make_icon.py
  docs/      spec.md
  README.md  LICENSE  .gitignore
```
