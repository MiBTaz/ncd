// src/main.rs
// License: PolyForm Noncommercial 1.0.0 (Personal & Research Use Only)
// Commercial use is strictly prohibited without a separate agreement.
// Redistribution is permitted provided this notice and license remain intact.
//
//! # NCD (Navigation Control Directory)
//!
//! NCD is a high-speed directory jumper optimized for Windows development environments.
//! It resolves ambiguous or partial paths using a prioritized search pipeline:
//!
//! 1. **Literal/Anchored**: Immediate resolution for absolute or root-relative paths.
//! 2. **Ellipsis**: Intelligent parent-directory hopping (`...` -> `up 2`).
//! 3. **CWD Context**: Searching children of the current directory.
//! 4. **CDPATH Context**: Searching locations defined in the environment.

use std::{env, fmt, process};
use std::path::{Path, PathBuf};
use lexopt::{Parser, Arg};
use std::ffi::OsString;

#[cfg(test)]
mod unit_tests;

#[cfg(test)]
pub(crate) const DEFAULT_TEST_ROOT: &str = "V:\\tmp\\ncd_tests";

/// Governs how the engine treats directories found in the `CDPATH`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CdMode {
    /// Matches contents *inside* the CDPATH entries (Classic Shell style).
    Origin,
    /// Matches the CDPATH entry itself (Bookmark style).
    Target,
    /// Matches the entry name first, then its contents (Hybrid style).
    Hybrid
}

/// Consolidated state to prevent "Parameter Bloat" in the search pipeline.
/// Using a struct ensures that adding future features (like Frecency)
/// doesn't require changing every function signature in the project.
pub struct SearchOptions {
    pub mode: CdMode,
    pub exact: bool,
    pub list: bool,
    pub mock_path: Option<std::ffi::OsString>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("NCD Error: {}", e);
        process::exit(1);
    }
}

/// CLI Entry Point: Orchestrates environment state and user input.
///
/// This function serves as the high-level conductor. It parses CLI arguments,
/// initializes the search context, and delegates to the search pipeline.
/// It is specifically designed to handle the "Silent Failure" problem by
/// ensuring all non-zero exits provide meaningful feedback to the shell.
fn run() -> Result<(), NcdError> {
    let mut query: Option<String> = None;
    let mut opts = SearchOptions {
        mode: match env::var("NCD_MODE").ok().as_deref() {
            Some("target") => CdMode::Target,
            Some("hybrid") => CdMode::Hybrid,
            _ => CdMode::Origin,
        },
        exact: false,
        list: false,
        mock_path: None,
    };

    let mut parser = Parser::from_env();
    while let Some(arg) = parser.next().map_err(|e| NcdError::ArgError(e.to_string()))? {
        match arg {
            Arg::Short('h') | Arg::Long("help") => { help(); process::exit(0); }
            Arg::Short('l') | Arg::Long("list") => opts.list = true,
            Arg::Short('e') | Arg::Long("exact") => opts.exact = true,
            Arg::Long("cd") => {
                let val = parser.value().map_err(|e| NcdError::ArgError(e.to_string()))?;
                opts.mode = match val.to_string_lossy().as_ref() {
                    "origin" => CdMode::Origin, "target" => CdMode::Target, "hybrid" => CdMode::Hybrid,
                    _ => return Err(NcdError::ArgError("Invalid cd mode.".into())),
                };
            }
            Arg::Value(val) => { query = Some(val.into_string().map_err(NcdError::InvalidUnicode)?); }
            _ => {}
        }
    }

    // Default to Home (~) if no query is provided.
    let s = query.unwrap_or_else(|| "~".to_string());
    let q = s.trim();

    // ~ Resolution: Bypasses the search engine for immediate OS profile mapping.
    if q == "~" { return resolve_home(); }

    // Execute the Search Pipeline
    let results = evaluate_jump(&q, &opts);

    // ERROR RESOLUTION & INTEGRATION TEST COMPLIANCE:
    // If results are empty, we must emit a specific error string to stderr
    // before exiting with 1. This prevents the shell wrapper from attempting
    // a null jump and satisfies the integration test predicates.
    if results.is_empty() {
        eprintln!("NCD Error: Could not resolve \"{}\"", q);
        process::exit(1);
    }

    // Output valid paths to stdout for shell capture.
    for path in results {
        // UNC paths (\\?\) are stripped to ensure compatibility with standard shell built-ins.
        println!("{}", path.to_string_lossy().trim_start_matches(r"\\?\"));
    }
    Ok(())
}

// --- CORE NAVIGATION ENGINE ---

/// The central brain of NCD. It deconstructs the user query and routes it
/// through specialized logic handlers (Ellipsis, Anchors, or CDPATH Search).
pub fn evaluate_jump(query: &str, opts: &SearchOptions) -> Vec<PathBuf> {
    let base = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if query == "." {
        return vec![base];
    }

    if query == ".." {
        return match base.parent() {
            Some(p) => vec![p.to_path_buf()],
            None => vec![base], // Root protection
        };
    }

    // Standard shell 'last directory' toggle.
    if query == "-" {
        return env::var_os("OLDPWD").map(|os| vec![PathBuf::from(os)]).unwrap_or_default();
    }
    let base = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // 1. ANCHOR ANALYSIS
    // Anchors determine the "Search Root".
    // If anchored with a slash, we ignore the CWD and jump straight to the Volume Root.
    let is_root_anchored = query.chars().next().map(std::path::is_separator).unwrap_or(false)
        || (query.len() >= 3 && query.get(1..3) == Some(":\\"));
    let p = PathBuf::from(query);
    let (head, tail) = split_query(query, is_root_anchored);

    //  2. THE FAST PATH (Literal/Absolute)
    // If it's a real, existing path on disk, we take it immediately.
    // No regex, no scanning, no ambiguity checks.
    if (is_root_anchored || p.is_absolute()) && p.exists() {
        if let Ok(abs) = std::path::absolute(&p) { return vec![abs]; }
    }

    // 3. ELLIPSIS LOGIC
    // Translates '...' into 'cd ../..'. This is handled before searching
    // because it relies on path arithmetic rather than directory scanning.
    if is_ellipsis(head) {
        // handle_ellipsis now calls search_cdpath which is scoped by get_search_roots
        return handle_ellipsis(head, tail, opts, base).unwrap_or_default();
    }

    // 4. THE SEARCH ENGINE FALLBACK
    // If the path isn't literal, we determine our "Roots" and start searching.
    // If it was root-anchored but didn't exist as a literal (likely a wildcard),
    // we search ONLY the root.
    if is_root_anchored {
        let root = get_drive_root(&base).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("\\"));
        let search_opts = SearchOptions {
            mode: CdMode::Hybrid,
            exact: opts.exact,
            list: opts.list,
            mock_path: Some(root.into_os_string()),
        };
        let matches = search_cdpath(head, &search_opts);
        return reattach_tail(matches, tail);
    }
    // 5. THE NAKED FALLTHROUGH
    // This executes ONLY if none of the above specific protocols matched.
    let matches = search_cdpath(head, opts);
    reattach_tail(matches, tail)
}

/// Helper to ensure the "tail" of a query is preserved after a fuzzy match.
/// Example: query 'proj/src' -> engine finds 'V:\Projects\ProjectA' -> returns 'V:\Projects\ProjectA\src'
/*
fn reattach_tail(matches: Vec<PathBuf>, tail: Option<&str>) -> Vec<PathBuf> {
    matches.into_iter().map(|mut path| {
        if let Some(t) = tail {
            path.push(t);
        }
        path
    }).collect()
}
*/
fn reattach_tail(matches: Vec<PathBuf>, tail: Option<&str>) -> Vec<PathBuf> {
    matches.into_iter().map(|mut path| {
        if let Some(t) = tail {
            let s = path.to_string_lossy();
            if s.len() == 2 && s.get(1..2) == Some(":") {
                path.push(std::path::MAIN_SEPARATOR.to_string());
            }
            path.push(t);
        }
        path
    }).collect()
}



/// The main search loop. It iterates through possible search roots (CWD, CDPATH)
/// and applies a 3-phase matching strategy to each.
pub fn search_cdpath(name: &str, opts: &SearchOptions) -> Vec<PathBuf> {
    let engine = SearchEngine::new(name, opts.exact);
    let mut all_matches = Vec::new();
    let roots = get_search_roots(&opts.mock_path);

    for (i, root) in roots.into_iter().enumerate() {
        if !root.is_dir() { continue; }
        let mut matches = Vec::new();

        // PHASE A: DIRECT CHILD HIT
        // Checks if 'root/name' exists as a literal directory.
        // On Windows, we must manually verify case-preserving 'Exact' matches
        // because the OS filesystem is naturally case-insensitive.
        if !engine.is_wildcard && !name.is_empty() {
            if let Some(path) = engine.check_direct(&root) {
                if !opts.exact { return vec![path]; }
                matches.push(path);
            }
        }

        // PHASE B: TARGET (CDPATH Folder Match)
        // If query is 'Work' and 'V:\Work' is in CDPATH, this phase identifies it.
        // Skips index 0 (the CWD) to prevent NCD from "finding itself" constantly.
        let is_mock_search = opts.mock_path.is_some();
        if (i > 0 || is_mock_search) && opts.mode != CdMode::Origin {
            if engine.matches_path(&root) {
                matches.push(root.clone());
            }
        }

        // PHASE C: ORIGIN (Scoping Scan)
        // The standard "Search Inside" phase. Iterates through all children of the root.
        if opts.mode != CdMode::Target && (i == 0 || matches.is_empty()) {
            matches.extend(engine.scan_dir(&root));
        }

        // AMBIGUITY RESOLUTION
        // In 'List' mode, we collect everything. In 'Jump' mode, we require a unique match.
        if !matches.is_empty() {
            if opts.list { all_matches.extend(matches); }
            else if matches.len() == 1 { return matches; }
            else { report_ambiguity(&root, matches); }
        }
    }
    all_matches
}

// --- ENGINE MODULES ---

/// Encapsulates all pattern-matching logic.
/// Centralizing this prevents duplication between Phase B and Phase C scans.
struct SearchEngine {
    query: String,
    query_lower: String,
    is_wildcard: bool,
    exact: bool,
    re: Option<regex::Regex>,
}

impl SearchEngine {
    /// Constructs the engine and pre-compiles Wildcards into RegEx.
    /// Uses a "Progressive Transformation" (Perl-style) to convert
    /// Shell Globs into valid, anchored Regular Expressions.
    fn new(name: &str, exact: bool) -> Self {
        let is_wildcard = name.contains('*') || name.contains('?');
        let re = if is_wildcard {
            // Start with a mutable string to perform sequential sanitization.
            let mut pattern = name.to_string();

            // 1. Literal Escape: Ensure dots are treated as dots, not "any character".
            pattern = pattern.replace('.', "\\.");

            // 2. Glob translation: '?' in shell means "one character" (. in regex).
            pattern = pattern.replace('?', ".");

            // 3. Glob translation: '*' in shell means "any characters" (.* in regex).
            pattern = pattern.replace('*', ".*");

            // Build the final anchored regex. We anchor with ^ and $ to ensure
            // the pattern matches the WHOLE directory name, not just a substring.
            // Note: We do NOT escape backslashes here because this engine
            // matches against file_name() which is a pure component (no slashes).
            regex::RegexBuilder::new(&format!("^{}$", pattern))
                .case_insensitive(!exact)
                .build()
                .ok()
        } else { None };

        Self {
            query: name.to_string(),
            query_lower: name.to_lowercase(),
            is_wildcard,
            exact,
            re,
        }
    }
    /// Verifies existence and performs the "Truth Check" for Windows casing.
    fn check_direct(&self, root: &Path) -> Option<PathBuf> {
        let path = root.join(&self.query);
        if !path.is_dir() { return None; }

        // Windows Truth Check: canonicalize() returns the path exactly as stored on disk.
        if self.exact && get_disk_casing(&path) != self.query { return None; }
        Some(path)
    }

    /// Primary matching logic used for both folder names and directory entries.
    fn matches_path(&self, path: &Path) -> bool {
        let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        if let Some(ref re) = self.re { re.is_match(&name) }
        else if self.exact { name == self.query }
        else { name.to_lowercase() == self.query_lower }
    }

    /// High-performance directory crawler.
    fn scan_dir(&self, root: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                // Ignore files; NCD is strictly for directory navigation.
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }

                let name = entry.file_name().to_string_lossy().into_owned();
                let is_match = if let Some(ref re) = self.re { re.is_match(&name) }
                else if self.exact { name == self.query }
                else {
                    let nl = name.to_lowercase();
                    // Supports both 'exact match' and 'starts with' for fast typing.
                    nl == self.query_lower || nl.starts_with(&self.query_lower)
                };
                if is_match { found.push(entry.path()); }
            }
        }
        found
    }
}

// --- UTILITIES & SYSTEM HELPERS ---

/// Returns the actual case-preserved name stored by NTFS.
fn get_disk_casing(path: &Path) -> String {
    path.canonicalize().ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_default()
}

/// Splits queries using platform-native separators (supports / and \ on Windows).
fn split_query(query: &str, anchored: bool) -> (&str, Option<&str>) {
    let trimmed = if anchored { &query[1..] } else { query };
    let parts: Vec<&str> = trimmed.splitn(2, std::path::is_separator).collect();
    (parts[0], parts.get(1).copied())
}

/*
fn split_query(query: &str, anchored: bool) -> (&str, Option<&str>) {
    if anchored {
        match query.rfind(std::path::is_separator) {
            Some(pos) => (&query[..pos], Some(&query[pos + 1..])),
            None => (query, None),
        }
    } else {
        let parts: Vec<&str> = query.splitn(2, std::path::is_separator).collect();
        (parts[0], parts.get(1).copied())
    }
}
 */

fn is_ellipsis(head: &str) -> bool {
    head.len() > 1 && head.chars().all(|c| c == '.')
}

/// Computes parent directory hops. testing primarily optional parm: pathbuf
fn handle_ellipsis(head: &str,
                   tail: Option<&str>,
                   opts: &SearchOptions,
                   base: PathBuf ) -> Option<Vec<PathBuf>> {
    let mut current = base;
    // Dot count to jump mapping: '..' is parent, '...' is parent of parent.
    for _ in 0..(head.len() - 1) { current.pop(); }

    if let Some(remainder) = tail {
        // Use split_query here too!
        // We treat the remainder as a naked query relative to our new 'current' root.
        let (sub_head, sub_tail) = split_query(remainder, false);
        // Recursive-style jump: Pop parents, THEN search for the remainder.
        let sub_opts = SearchOptions {
            mode: CdMode::Hybrid, exact: opts.exact, list: opts.list,
            mock_path: Some(current.into_os_string())
        };
        let matches = search_cdpath(sub_head, &sub_opts);
        return Some(reattach_tail(matches, sub_tail));
    }
    Some(vec![current])
}

/// Finds the root of the current drive (e.g., V:\Projects -> V:\) to support root-anchored jumps.
fn get_drive_root<P: AsRef<Path>>(cwd: P) -> Option<OsString> {
    cwd.as_ref()
        .ancestors()
        .last()
        .map(|r| r.to_path_buf().into_os_string())
}

/// Gathers all possible search origins. Priority: 1. CWD, 2. CDPATH.
/// Gathers search origins based on the "Exclusive Authority" principle.
/// If a mock root is provided (via Ellipsis or Root Anchor), CWD and CDPATH are ignored.
fn get_search_roots(mock: &Option<std::ffi::OsString>) -> Vec<PathBuf> {
    if let Some(m) = mock {
        // PATH-LOCK: The user specified exactly where to start.
        // We refuse to "pollute" the search with the CWD or CDPATH.
        return vec![PathBuf::from(m)];
    }

    // NAKED QUERY: Fallback to standard heuristics.
    let mut roots = Vec::new();
    if let Ok(cwd) = env::current_dir() { roots.push(cwd); }

    if let Some(cdpath) = env::var_os("CDPATH") {
        roots.extend(env::split_paths(&cdpath));
    } else {
        // Fallback for environments where CDPATH isn't initialized
        roots.push(PathBuf::from("V:\\Projects"));
    }
    roots
}

fn resolve_home() -> Result<(), NcdError> {
    let home = env::var_os("USERPROFILE").or_else(|| env::var_os("HOME")).map(PathBuf::from);
    if let Some(path) = home { println!("{}", path.display()); return Ok(()); }
    Err(NcdError::ResolutionFailed("HOME not found".into()))
}

/// Prevents non-deterministic navigation by forcing the user to be more specific.
fn report_ambiguity(root: &Path, matches: Vec<PathBuf>) -> ! {
    eprintln!("\nNCD Error: Ambiguous match in {}:", root.display());
    for m in matches { eprintln!("  -> {}", m.display()); }
    process::exit(1);
}

// --- BOILERPLATE ---

#[derive(Debug)]
pub enum NcdError {
    InvalidUnicode(std::ffi::OsString),
    ResolutionFailed(String),
    ArgError(String),
    Io(std::io::Error)
}

impl std::error::Error for NcdError {}
impl fmt::Display for NcdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUnicode(os) => write!(f, "Invalid Unicode: {:?}", os),
            Self::ResolutionFailed(q) => write!(f, "Could not resolve \"{}\"", q),
            Self::ArgError(msg) => write!(f, "Arg error: {}", msg),
            Self::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

fn help() {
    let help_text = r#"
NCD: High-Speed Directory Navigator (Fortress Edition)

USAGE:
    ncd [OPTIONS] <PATH>

ARGUMENTS:
    <PATH>
        ...           Jump up parent directories (3 dots = up 2 levels, no limit).
        -             Jump to the previous directory (OLDPWD).
        ~             Jump to home directory.
        project       Search for a project directory in CWD then CDPATH.
        project/src   Search for 'project' then append 'src'.
        proj*         Wildcard search (Matches 'Project_Alpha', etc).
        *go*          Glob match (Matches '.cargo', 'Cargo', gopher, etc).

OPTIONS:
    -h, --help        Print this help message.
    -q, --quiet       Suppress error messages on resolution failure.
    -e, --exact       Disable case-insensitive fallback (Strict matching).
    -l, --list        List all matches instead of jumping (Search Engine mode).
    --cd=<MODE>       Set search strategy (default mode: origin).

MODES:
    origin            Scans INSIDE directories listed in CDPATH. (default, sh style)
    target            Matches the FOLDER NAME of entries in CDPATH (bookmarks).
    hybrid            Checks if entry is the target; if not, scans inside.

WILDCARDS:
    * Matches any sequence of characters.
    ?                 Matches any single character.
    Note: Standard jumps require a unique match. If multiple directories
    match a wildcard, NCD will list them and abort to prevent "FUBAR" jumps.
    Use --list to see all matches without aborting.

ENVIRONMENT VARIABLES:
    CDPATH            Semicolon-separated list of search roots.
                      Default: V:\Projects

    NCD_MODE          Set default strategy (origin, target, hybrid).
                      Default: origin

    USERPROFILE/HOME  Used for '~' resolution.

    OLDPWD            Maintained by shell; used for '-' resolution.

EXAMPLES:
    ncd .....           (Up four levels)
    ncd ...\build       (up two levels, down to build)
    ncd --cd=origin     (CDPATH logic, set to origin)
    ncd -               (Toggle back)
    ncd *test*          Jump to the unique directory containing "test".
    ncd --list pro*     List all projects starting with "pro".

CAVEATS
    search priority is: 1. Ellipse Logic (... and .../dir)
                        2. CWD (Current Working Directory) children/explicit paths
                        3. CDPATH roots (via Origin, Target, or Hybrid strategy
                           Default CDPATH behaviour is POSIX/Unix (--cd=origin)

Portability: Uses OS-native path separators and environment variables.

"#;
    eprintln!("{}", help_text.trim());
}
