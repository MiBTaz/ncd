use std::env;
use std::path::{PathBuf};
use std::process;
use lexopt::{Parser, Arg};
use std::fmt;

#[cfg(test)]
mod unit_tests; 
#[cfg(test)]
const DEFAULT_TEST_ROOT: &str = "V:\\tmp\\ncd_tests";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CdMode {
    Origin,
    Target,
    Hybrid,
}

#[derive(Debug)]
pub enum NcdError {
    InvalidUnicode(std::ffi::OsString),
    ResolutionFailed(String),
    ArgError(String),
    Io(std::io::Error),
}

fn main() {
    if let Err(e) = run() {
        eprintln!("NCD Error: {}", e);
        process::exit(1);
    }
}

impl fmt::Display for NcdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUnicode(os) => write!(f, "Invalid Unicode in path: {:?}", os),
            Self::ResolutionFailed(q) => write!(f, "Could not resolve \"{}\"", q),
            Self::ArgError(msg) => write!(f, "Argument error: {}", msg),
            Self::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for NcdError {}

fn run() -> Result<(), NcdError> {
    let mut query: Option<String> = None;
    let mut quiet = false;
    let mut exact_mode = false;
    let mut list_mode = false;

    let mut mode = match env::var("NCD_MODE").ok().as_deref() {
        Some("target") => CdMode::Target,
        Some("hybrid") => CdMode::Hybrid,
        _ => CdMode::Origin,
    };

    let mut parser = Parser::from_env();

    while let Some(arg) = parser.next().map_err(|e| NcdError::ArgError(e.to_string()))? {
        match arg {
            Arg::Short('h') | Arg::Long("help") => { help(); process::exit(0); }
            Arg::Short('l') | Arg::Long("list") => list_mode = true,
            Arg::Short('q') | Arg::Long("quiet") => quiet = true,
            Arg::Short('e') | Arg::Long("exact") => exact_mode = true,
            Arg::Long("cd") => {
                let val = parser.value().map_err(|e| NcdError::ArgError(e.to_string()))?;
                mode = match val.to_string_lossy().as_ref() {
                    "origin" => CdMode::Origin,
                    "target" => CdMode::Target,
                    "hybrid" => CdMode::Hybrid,
                    _ => return Err(NcdError::ArgError("Invalid cd mode.".into())),
                };
            }
            Arg::Value(val) => { query = Some(val.into_string().map_err(NcdError::InvalidUnicode)?); }
            _ => return Err(NcdError::ArgError("Unexpected argument".into())),
        }
    }

    let q = query.unwrap_or_else(|| "~".to_string());

    if q == "~" {
        let home = env::var_os("USERPROFILE").or_else(|| env::var_os("HOME")).map(PathBuf::from);
        if let Some(path) = home { println!("{}", path.display()); return Ok(()); }
        return Err(NcdError::ResolutionFailed("HOME not found".into()));
    }

    let results = evaluate_jump(&q, mode, exact_mode, list_mode);

    if results.is_empty() {
        if !quiet { eprintln!("NCD: Could not resolve \"{}\"", q); }
        process::exit(1);
    }

    for path in results {
        let path_str = path.to_string_lossy();
        let clean_path = path_str.trim_start_matches(r"\\?\");
        if !clean_path.is_empty() {
            println!("{}", clean_path);
        }
    }
    Ok(())
}

pub fn evaluate_jump(query: &str, mode: CdMode, exact_mode: bool, list_mode: bool) -> Vec<PathBuf> {
    if query == "-" {
        return env::var_os("OLDPWD").map(|os| vec![PathBuf::from(os)]).unwrap_or_default();
    }

    // --- 1. Handle Leading Slashes & Split ---
    let is_root_anchored = query.starts_with('/') || query.starts_with('\\');
    let trimmed_query = if is_root_anchored { &query[1..] } else { query };

    let parts: Vec<&str> = trimmed_query.splitn(2, |c| c == '/' || c == '\\').collect();
    let head = parts[0];
    let tail = parts.get(1).copied();

    // --- 2. Ellipsis & Absolute Paths ---
    if head.len() > 1 && head.chars().all(|c| c == '.') {
        if let Ok(mut current) = env::current_dir() {
            for _ in 0..(head.len() - 1) { current.pop(); }
            if let Some(remainder) = tail {
                return search_cdpath(remainder, CdMode::Origin, exact_mode, list_mode, Some(current.into_os_string()));
            }
            return vec![current];
        }
    }

    // Try literal absolute path first
    let p = PathBuf::from(query);
    if is_root_anchored || p.is_absolute() {
        if let Ok(abs) = std::path::absolute(&p) {
            if abs.exists() { return vec![abs]; }
        }
    }

    // --- 3. The Search Fallback ---
    let mock_root = if is_root_anchored {
        env::current_dir().ok().and_then(|p| {
            p.ancestors().last().map(|root| root.to_path_buf().into_os_string())
        })
    } else {
        None
    };

    let matches = search_cdpath(head, mode, exact_mode, list_mode, mock_root);

    matches.into_iter().map(|mut path| {
        if let Some(remainder) = tail { path.push(remainder); }
        path
    }).collect()
}

pub fn search_cdpath(
    name: &str,
    mode: CdMode,
    exact_mode: bool,
    list_mode: bool,
    mock_cdpath: Option<std::ffi::OsString>
) -> Vec<PathBuf> {
    let mut all_matches = Vec::new();
    let query_lower = name.to_lowercase();
    let is_wildcard = name.contains('*') || name.contains('?');

    let wildcard_re = if is_wildcard {
        let pattern = name.replace(".", "\\.").replace("?", ".").replace("*", ".*");
        regex::RegexBuilder::new(&format!("^{}$", pattern))
            .case_insensitive(!exact_mode)
            .build().ok()
    } else { None };

    let mut search_roots: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = env::current_dir() { search_roots.push(cwd); }

    let cdpath_env = mock_cdpath.unwrap_or_else(|| {
        env::var_os("CDPATH").unwrap_or_else(|| "V:\\Projects".into())
    });
    search_roots.extend(env::split_paths(&cdpath_env));

    for (i, root) in search_roots.into_iter().enumerate() {
        if !root.is_dir() { continue; }
        let mut matches = Vec::new();

        // PHASE A: DIRECT JOIN (with Windows Reality Check)
        if !is_wildcard && !name.is_empty() {
            let direct_child = root.join(name);
            if direct_child.is_dir() {
                if !exact_mode {
                    return vec![direct_child];
                } else {
                    let disk_name = direct_child.canonicalize().ok()
                        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                        .unwrap_or_default();
                    if disk_name == name { return vec![direct_child]; }
                }
            }
        }

        // PHASE B: TARGET (CDPATH Entries)
        if i > 0 && mode != CdMode::Origin {
            if let Some(root_name_os) = root.file_name() {
                let root_name = root_name_os.to_string_lossy();
                let is_match = if let Some(ref re) = wildcard_re { re.is_match(&root_name) }
                else if exact_mode { root_name == name }
                else { root_name.to_lowercase() == query_lower };
                if is_match { matches.push(root.clone()); }
            }
        }

        // PHASE C: ORIGIN (Scan inside)
        if mode != CdMode::Target && (i == 0 || matches.is_empty()) {
            if let Ok(entries) = std::fs::read_dir(&root) {
                for entry in entries.flatten() {
                    if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
                    let name_str = entry.file_name().to_string_lossy().into_owned();
                    let is_match = if let Some(ref re) = wildcard_re { re.is_match(&name_str) }
                    else if exact_mode { name_str == name }
                    else {
                        let nl = name_str.to_lowercase();
                        nl == query_lower || nl.starts_with(&query_lower)
                    };
                    if is_match { matches.push(entry.path()); }
                }
            }
        }

        if !matches.is_empty() {
            if list_mode { all_matches.extend(matches); }
            else if matches.len() == 1 { return matches; }
            else {
                eprintln!("\nNCD Error: Ambiguous match in {}:", root.display());
                for m in matches { eprintln!("  -> {}", m.display()); }
                process::exit(1);
            }
        }
    }
    all_matches
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
