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
use std::collections::{HashSet};
use std::path::{Path, PathBuf};
use lexopt::{Parser, Arg};

#[cfg(test)]
mod unit_tests_local;
#[cfg(test)]
mod unit_tests;

#[cfg(test)]
pub(crate) const DEFAULT_TEST_ROOT: &str = "V:\\tmp\\ncd_tests";

#[cfg(windows)]
pub const DOS_SEPARATOR: char = '\\';
#[cfg(not(windows))]
pub const DOS_SEPARATOR: char = '/';

pub const UNIX_SEPARATOR: char = '/';

#[cfg(windows)]
pub const DRIVE_SEPARATOR: char = ':';
#[cfg(not(windows))]
pub const DRIVE_SEPARATOR: char = char::MAX;

pub const PATH_SEPARATORS: &[char] = &[DOS_SEPARATOR, UNIX_SEPARATOR];

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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DirMatch {
    AsIs,
    Fuzzy,
}
impl Default for DirMatch {
    fn default() -> Self {
        Self::AsIs
    }
}

/// Consolidated state to prevent "Parameter Bloat" in the search pipeline.
/// Using a struct ensures that adding future features (like Frecency)
/// doesn't require changing every function signature in the project.
pub struct SearchOptions {
    pub mode: CdMode,
    pub exact: bool,
    pub list: bool,
    pub dir_match: DirMatch,
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
        dir_match: DirMatch::AsIs,
        mock_path: None,
    };

    let mut parser = Parser::from_env();
    while let Some(arg) = parser.next().map_err(|e| NcdError::ArgError(e.to_string()))? {
        match arg {
            Arg::Short('h') | Arg::Long("help") => { help(); process::exit(0); }
            Arg::Short('l') | Arg::Long("list") => opts.list = true,
            Arg::Short('e') | Arg::Long("exact") => opts.exact = true,
            Arg::Short('#') | Arg::Long("glob") => opts.dir_match = DirMatch::Fuzzy,
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
    let t = s.trim().trim_end_matches(|c| c == DOS_SEPARATOR || c == UNIX_SEPARATOR);
    let q  = trim_to_elipses(t).to_string();

    match q.as_str() {
        "" if !s.trim().is_empty() => {
            println!("{}", DOS_SEPARATOR);
            return Ok(())
        }
        "." => {
            let p = env::current_dir().map_err(|e| NcdError::ResolutionFailed(e.to_string()))?;
            println!("{}", p.display()); return Ok(());
        }
        ".." => {
            let p = env::current_dir().map_err(|e| NcdError::ResolutionFailed(e.to_string()))?;
            println!("{}", p.parent().unwrap_or(&p).display()); return Ok(());
        }
        "~" => return resolve_home(),
        "-" => {
            let old = env::var_os("OLDPWD").ok_or(NcdError::ResolutionFailed("OLDPWD not set".into()))?;
            println!("{}", PathBuf::from(old).display()); return Ok(());
        }
        _ => {} // Continue to evaluate_jump
    }

    // Execute the Search Pipeline
    let results = evaluate_jump(&q, &opts);
    if results.len() > 1 && !opts.list {
        report_ambiguity(Path::new(&q), results);
    }

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
pub fn evaluate_jump(raw_query: &str, opts: &SearchOptions) -> Vec<PathBuf> {
    let query_check = raw_query.trim();
    if query_check.is_empty() { return vec![]; }
    let query = PathBuf::from(query_check).to_string_lossy().to_string();
    let base = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let starts_with_sep = query.starts_with(std::path::is_separator);
    let (bare, is_anchored, components) = get_drive_components(&query);
    if bare {
        return vec![PathBuf::from(query)];
    }

    let (head, tails) = split_query(components, starts_with_sep, is_anchored);

    // If not anchored, the 'head' is our first search term
    let (start_roots, segments) = if is_anchored || starts_with_sep {
        let sep = PATH_SEPARATORS[0];
        let root = if starts_with_sep {
            let r = get_drive_root(&base).unwrap_or_else(|| PathBuf::from(sep.to_string()));
            PathBuf::from(format!("{}{}", r.display().to_string().trim_end_matches(PATH_SEPARATORS), sep))
        } else {
            PathBuf::from(format!("{}{}", head, sep))
        };
        (vec![root], tails)
    } else {
        // Naked query: Start at CWD, and the whole query parts are segments
        let mut s = Vec::new();
        if !head.is_empty() { s.push(head); }
        s.extend(tails);
        (vec![base], s)
    };

    resolve_path_segments(start_roots, segments, opts)
}

fn resolve_path_segments(matches: Vec<PathBuf>, mut segments: Vec<&str>, opts: &SearchOptions) -> Vec<PathBuf> {
    segments.retain(|&s| !s.trim().is_empty() && s.trim() != ".");
    if segments.is_empty() || matches.is_empty() { return matches; }

    let segment = segments.remove(0);
    let mut next_matches = Vec::new();

    for path in matches {
        let nav = trim_to_elipses(segment);
        if is_ellipsis(&nav) {
            next_matches.extend(handle_ellipsis(&nav, path));
        } else if nav == ".." {
            if let Some(parent) = path.parent() {
                next_matches.push(parent.to_path_buf());
            }
        } else {
            let is_base_cwd = env::current_dir().map(|c| c == path).unwrap_or(false);

            let found = if is_base_cwd {
                // First segment: check CDPATH/CWD
                search_cdpath(segment, opts)
            } else {
                // Sub-segments: LOCK to the specific folder found
                let locked_opts = SearchOptions {
                    mode: opts.mode,
                    exact: opts.exact,
                    list: opts.list,
                    dir_match: opts.dir_match,
                    mock_path: Some(path.clone().into_os_string()),
                };
                search_cdpath(segment, &locked_opts)
            };

            next_matches.extend(found);
        }
    }

    if next_matches.is_empty() { return Vec::new(); }

    // THE RECURSION FIX:
    // Create the new options for the next level.
    // We use the first match as the anchor for the next segment's search.
    let next_opts = SearchOptions {
        mode: opts.mode,
        exact: opts.exact,
        list: opts.list,
        dir_match: opts.dir_match,
        mock_path: Some(next_matches[0].clone().into_os_string()),
    };

    resolve_path_segments(next_matches, segments, &next_opts)
}

/// The main search loop. It iterates through possible search roots (CWD, CDPATH)
/// and applies a 3-phase matching strategy to each.
pub fn search_cdpath(name: &str, opts: &SearchOptions) -> Vec<PathBuf> {
    let engine = SearchEngine::new(name, opts.exact);
    let mut all_matches = Vec::new();
    let mut dirs = HashSet::new();
    let roots = get_search_roots(&opts.mock_path);

    for (i, root) in roots.into_iter().enumerate() {
        if !root.is_dir() { continue; }
        let mut matches = Vec::new();
        let canon_root = root.canonicalize().unwrap_or_else(|_| root.clone());
        let is_mock_search = opts.mock_path.is_some();

        // PHASE A: DIRECT CHILD HIT (Absolute/Relative paths)
        if !engine.is_wildcard && !name.is_empty() {
            if let Some(path) = engine.check_direct(&root) {
                let d = path.canonicalize().unwrap_or_else(|_| path.clone());
                if !opts.exact { return vec![path]; }
                if dirs.insert(d) { matches.push(path); }
            }
        }

        // Phase B: for CWD, wildcard matching, any mode.
        if engine.is_wildcard && i == 0 {
            for p in engine.scan_dir(&root, opts) {
                let d = p.canonicalize().unwrap_or_else(|_| p.clone());
                if dirs.insert(d) { matches.push(p); }
            }
        }

        // PHASE C: TARGET (The folder itself is the bookmark)
        if (i > 0 || is_mock_search) && opts.mode != CdMode::Origin {
            if engine.matches_path(&root) {
                if dirs.insert(canon_root.clone()) { matches.push(root.clone()); }
            }
        }


        // PHASE D: ORIGIN (Search inside the folder)
        //        if opts.mode != CdMode::Target && (i == 0 || matches.is_empty()) {
        if opts.mode != CdMode::Target && (i == 0 || matches.is_empty()) {
            for p in engine.scan_dir(&root, opts) {
                let d = p.canonicalize().unwrap_or_else(|_| p.clone());
                if dirs.insert(d) { matches.push(p); }
            }
        }

        if !matches.is_empty() {
            if opts.list || engine.is_wildcard { all_matches.extend(matches); }
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
    fn scan_dir(&self, root: &Path, opts: &SearchOptions) -> Vec<PathBuf> {
        let mut found = Vec::new();
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                // Ignore files; NCD is strictly for directory navigation.
                // if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
                // don't ignore junctions
                if !entry.path().is_dir() { continue; }
                // skip CWD for wildcard
                let name = entry.file_name().to_string_lossy().into_owned();
                let is_match = if let Some(ref re) = self.re {
                    re.is_match(&name)
                } else if self.exact {
                    name == self.query
                } else {
                    // Supports both 'exact match' and 'starts with' for fast typing.
                    let nl = name.to_lowercase();
                    if opts.dir_match == DirMatch::Fuzzy {
                        nl == self.query_lower || nl.starts_with(&self.query_lower)
                    } else { // (opts.dir_match == DirMatch::AsIs
                        nl == self.query_lower
                    }
                };
                if is_match { found.push(entry.path()); }
            }
        }
        found
    }
}

// --- UTILITIES & SYSTEM HELPERS ---
fn get_drive_components(path: &str) -> (bool, bool, Vec<&str>)  {
    let mut anchored = false;
    let mut bare = false;
    let parts: Vec<&str> = path.split(PATH_SEPARATORS).collect();

    if parts.is_empty() {
        return (bare, anchored, Vec::new());
    }

    if path.starts_with(|c| PATH_SEPARATORS.contains(&c)) { anchored = true; }

    if parts[0].len() == 2 &&
       parts[0].ends_with(DRIVE_SEPARATOR) &&
       parts[0].chars().next().map_or(false, |c| c.is_alphabetic()) {
            anchored = true;
    }

    if anchored && (parts.len() == 1 || parts[1].is_empty()) {
        bare = true;
    }

    (bare, anchored, parts)
}

/// Returns the actual case-preserved name stored by NTFS.
fn get_disk_casing(path: &Path) -> String {
    path.canonicalize().ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_default()
}

/// Splits queries using platform-native separators.
/// Handles both standard paths and root-anchored paths (starting with \ or C:\).
pub fn split_query(query: Vec<&str>, starts_with_sep: bool, is_anchored: bool) -> (&str, Vec<&str>) {
    let parts: Vec<&str> = query.into_iter()
        .filter(|s| !s.is_empty())
        .collect();

    if parts.is_empty() { return ("", Vec::new()); }

    if starts_with_sep {
        // Root relative: e.g., "\Projects" -> parts are ["Projects"]
        ("", parts)
    } else if is_anchored {
        // Drive anchored: e.g., "C:\Projects" -> head is "C:", tails are ["Projects"]
        (parts[0], parts[1..].to_vec())
    } else {
        // Naked query: e.g., "Project_Alpha*" -> treat all as segments for search_cdpath
        ("", parts)
    }
}

fn is_ellipsis(head: &str) -> bool {
    let dir = trim_to_elipses(head);
    dir.len() > 1 && dir.chars().all(|c| c == '.')
}

fn trim_to_elipses(path: &str) -> String {
    let is_nav = path.chars().all(|c| c == '.' || c == ' ');
    let p = if is_nav {
        path.chars().filter(|&c| c == '.').collect::<String>()
    } else {
        path.to_string()
    };
    p
}

/// Handles the "..." syntax.
/// If a tail exists (e.g., ".../src"), it pivots the search root to the calculated parent.
fn handle_ellipsis(segment: &str, base: PathBuf) -> Vec<PathBuf> {
    let mut current = if base.is_absolute() { base } else {
        base.canonicalize().unwrap_or_else(|_| env::current_dir().unwrap_or_default().join(base))
    };

    for _ in 0..(segment.len() - 1) {
        if !current.pop() { break; }
    }
    vec![current]
}

/// Finds the root of the current drive (e.g., V:\Projects -> V:\) to support root-anchored jumps.
fn get_drive_root(path: &Path) -> Option<PathBuf> {
    path.components()
        .next()
        .map(|c| PathBuf::from(c.as_os_str()))
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
    let mut seen = HashSet::new();
    let mut roots = Vec::new();

    // cwd is expected to be in the list even is it's dup'd in CDPATH.
    // so leave it in place
    if let Ok(cwd) = env::current_dir() {
        let _cwd2 = cwd.canonicalize().unwrap_or_else(|_| cwd.clone());
        // if seen.insert(cwd2) { roots.push(cwd.clone()); }
        roots.push(cwd.clone());
    }

    if let Some(cdpath) = env::var_os("CDPATH") {
        for p in env::split_paths(&cdpath) {
            let p2 = p.clone().canonicalize().unwrap_or_else(|_| p.clone());
            if seen.insert(p2.clone()) { roots.push(p); }
        }
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
    -#, --glob        Does wildcard matching without using the globs (* and ?).
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

