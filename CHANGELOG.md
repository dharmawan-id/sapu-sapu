# Changelog

All notable changes to Sapu Sapu.

## v0.2.0

**Added**
- **Streaming scan progress.** The drive overview streams a live file count, a running byte total, and the current path over a Tauri channel while it scans, so a full-drive scan never looks frozen.
- **Cancellable scans.** A Cancel button stops a running scan; the engine checks a shared flag on each step and bails out, reporting the count it reached.

## v0.1.0

The first public release.

**Added**
- A **disk overview** that scans a whole drive with a parallel walk and reports the biggest folders (rolled up to depth four), the biggest files, and a breakdown by file type, for `C:` and `D:`.
- A **risk-tiered cleaner**:
  - Green caches: uv, npm, pip, cargo, Temp, browser (Chrome and Brave), HuggingFace, VS Code.
  - Yellow project artifacts: `node_modules`, `target`, `dist`, `build`, `.next`, `__pycache__`, with a 30-day recency guard and a git-dirty check.
  - Protected paths: the Windows Installer store, the active Claude Code sandbox, the global npm prefix, browser profiles, and SSH and cloud keys, refused in the Rust engine.
- **Real reclaim**: the freed total is measured from the actual free-space change across `C:` and `D:`, not from logical folder sizes, so hardlinked caches report honestly.
- **Preview-first** selection, process-aware skipping of locked files, and an empty-recycle-bin action.
- A neo-brutalist interface with a timber accent, from the shared public-repo design system.
- A landing page and a bilingual (English and Indonesian) README.

**Notes**
- Built on the Rust GNU toolchain, so it compiles without Visual Studio.
- Runs without administrator rights. Needs the WebView2 runtime that ships with current Windows 10 and 11.
- Scans run off the UI thread (async commands on a blocking pool), so the window stays responsive during a full-drive scan. The overview also collapses ancestor and descendant folders so a parent and its dominant child are not both listed.
