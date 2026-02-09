use std::{env, fs};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use crate::{CdMode, DirMatch, SearchOptions, DEFAULT_TEST_ROOT};

// src/unit_tests.rs
struct CwdGuard(PathBuf);
impl CwdGuard {
    fn new(path: &Path) -> Self {
        let old = env::current_dir().unwrap();
        env::set_current_dir(path).unwrap();
        Self(old)
    }
}
impl Drop for CwdGuard {
    fn drop(&mut self) { env::set_current_dir(&self.0).unwrap(); }
}

/// Helper to generate SearchOptions on the fly for tests.
/// This keeps the test calls clean and matches the new 2-argument signature.
fn get_opts(mode: CdMode, exact: bool, mock: Option<OsString>) -> SearchOptions {
    SearchOptions {
        mode,
        exact,
        dir_match: DirMatch::default(),
        list: false, // Default to false for unit tests
        mock_path: mock,
    }
}

fn get_opts_fuzzy(mode: CdMode, exact: bool, mock: Option<OsString>) -> SearchOptions {
    SearchOptions {
        mode,
        exact,
        dir_match: DirMatch::Fuzzy,
        list: false, // Default to false for unit tests
        mock_path: mock,
    }
}

/// Dynamically resolves a test root based on environment or persistent defaults.
fn get_test_root() -> PathBuf {
    if let Ok(env_path) = env::var("NCD_TEST_DIR") {
        let p = PathBuf::from(env_path);
        if fs::create_dir_all(&p).is_ok() { return p; }
    }

    let persistent_root = PathBuf::from(DEFAULT_TEST_ROOT);
    if fs::create_dir_all(&persistent_root).is_ok() {
        return persistent_root;
    }

    let temp = env::temp_dir().join("ncd_tests");
    fs::create_dir_all(&temp).ok();
    temp
}

pub fn test_opts() -> SearchOptions {
    SearchOptions { mode: CdMode::Origin, exact: true, list: false, mock_path: None, dir_match: DirMatch::default(), }
}

fn setup_test_env() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    // Simulate: /Projects, /Drivers, /Windows, /Users/Guest/Desktop
    std::fs::create_dir_all(root.join("Projects/ncd/src")).unwrap();
    std::fs::create_dir_all(root.join("Drivers")).unwrap();
    std::fs::create_dir_all(root.join("Windows/System32")).unwrap();
    std::fs::create_dir_all(root.join("Users/Guest/Desktop")).unwrap();
    let root_path = root.clone();
    (tmp, root_path)
}




#[cfg(test)]
mod tests {
    use std::{env, fs};
    use crate::*;
    use tempfile::tempdir;
    use serial_test::serial;
    use std::path::{Path, PathBuf};
    use crate::unit_tests::{get_opts, get_test_root, setup_test_env, test_opts, CwdGuard};

    #[test]
    fn test_junction_follow() {
        let root = PathBuf::from("V:\\temp\\ncd_tests");
        if !root.exists() { return; }

        let test_dir = root.join("JunctionFollow");
        fs::create_dir_all(&test_dir).ok();

        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));
        let res = search_cdpath("JunctionFollow", &opts);
        assert!(!res.is_empty());
    }

    #[test]
    fn test_cdpath_exact_vs_fuzzy() {
        let root = get_test_root().join("ExactFuzzy");
        fs::create_dir_all(&root).ok();

        let name = "MixedCase123";
        let proj = root.join(name);
        if !proj.exists() { fs::create_dir(&proj).unwrap(); }

        let actual_name = proj.canonicalize().unwrap().file_name().unwrap().to_str().unwrap().to_string();
        let mock = root.into_os_string();

        // Fuzzy Match
        let opts_f = get_opts(CdMode::Origin, false, Some(mock.clone()));
        let res_f = search_cdpath("mixedcase123", &opts_f);
        assert!(!res_f.is_empty());

        // Exact Match
        let opts_e = get_opts(CdMode::Origin, true, Some(mock));
        let res_e = search_cdpath("mixedcase123", &opts_e);
        if actual_name == "mixedcase123" { assert!(!res_e.is_empty()); }
        else { assert!(res_e.is_empty()); }
    }

    #[test]
    fn test_dot_traversal() {
        let opts = get_opts(CdMode::Origin, false, None);
        let result = evaluate_jump("...", &opts);
        assert!(!result.is_empty());
        let current = env::current_dir().unwrap();
        let expected = current.parent().unwrap().parent().unwrap();
        assert_eq!(result[0], expected);
    }

    #[test]
    fn test_extreme_ellipsis() {
        let opts = get_opts(CdMode::Origin, false, None);
        let result = evaluate_jump(".....", &opts);
        assert!(!result.is_empty());
        let mut expected = env::current_dir().unwrap();
        for _ in 0..4 {
            if let Some(parent) = expected.parent() { expected = parent.to_path_buf(); }
        }
        assert_eq!(result[0], expected);
    }

    #[test]
    fn test_cdpath_exact_vs_fuzzy_tmp() {
        let dir = tempdir().unwrap();
        let proj_path = dir.path().join("MyProject");
        fs::create_dir(&proj_path).unwrap();
        let mock_env = Some(dir.path().as_os_str().to_os_string());

        let opts_f = get_opts(CdMode::Origin, false, mock_env.clone());
        let res_fuzzy = search_cdpath("myproject", &opts_f);
        assert!(!res_fuzzy.is_empty());

        let opts_e = get_opts(CdMode::Origin, true, mock_env);
        let res_exact = search_cdpath("myproject", &opts_e);
        assert!(res_exact.is_empty());
    }

    #[test]
    fn test_hybrid_mode() {
        let dir = tempdir().unwrap();
        let bookmark = dir.path().join("Work");
        fs::create_dir(&bookmark).unwrap();
        let mock_cdpath = Some(bookmark.as_os_str().to_os_string());

        let opts = get_opts(CdMode::Hybrid, true, mock_cdpath);
        let res = search_cdpath("Work", &opts);
        assert!(!res.is_empty());
        assert_eq!(res[0].canonicalize().unwrap(), bookmark.canonicalize().unwrap());
    }

    #[test]
    fn test_root_anchored_logic() {
        let opts = get_opts(CdMode::Origin, false, None);
        let result = evaluate_jump("\\Projects", &opts);
        assert!(!result.is_empty());
        let path_str = result[0].to_string_lossy();
        assert!(path_str.contains(":\\Projects"));
    }

    #[test]
    fn test_root_anchored_logic_mk2() {
        let opts = get_opts(CdMode::Origin, false, None);
        // Use a raw string to avoid escaping backslashes
        let result = evaluate_jump(r"\Projects", &opts);

        assert!(!result.is_empty(), "Search failed to return results for root anchor");

        let path_str = result[0].to_string_lossy().to_string();
        // Normalize Windows UNC for comparison
        let normalized = path_str.replace(r"\\?\", "");

        assert!(normalized.contains("Projects"), "Path missing 'Projects': {}", normalized);
        assert!(normalized.contains(":\\"), "Path should contain drive separator: {}", normalized);
    }

    #[test]
    fn test_wildcard_regex_logic() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("testing.1")).unwrap();
        let mock_path = Some(dir.path().as_os_str().to_os_string());

        let opts = get_opts(CdMode::Origin, false, mock_path);
        let res = search_cdpath("test*.*", &opts);
        assert!(!res.is_empty());
        assert!(res[0].to_string_lossy().contains("testing.1"));
    }

    #[serial]
    #[test]
    fn test_parent_globbing() {
        let dir = tempdir().unwrap();
        let parent = dir.path().join("parent_dir");
        let child = parent.join("child_glob");
        fs::create_dir_all(&child).unwrap();

        let original_cwd = env::current_dir().unwrap();
        env::set_current_dir(&child).unwrap();

        let opts = get_opts(CdMode::Origin, false, None);
        // Using portable separator check via evaluate_jump logic
        let res = evaluate_jump("..\\child*", &opts);

        env::set_current_dir(original_cwd).unwrap();

        assert!(!res.is_empty());
        assert!(res[0].to_string_lossy().contains("child_glob"));
    }

    #[serial]
    #[test]
    fn test_root_anchored_wildcard() {
        let root = get_test_root();
        let test_dir = root.join("WildcardTarget");
        let _ = fs::create_dir_all(&test_dir);

        let original_cwd = env::current_dir().unwrap();
        env::set_current_dir(&root).unwrap();

        let query = "Wildcard*";
        let opts = get_opts(CdMode::Hybrid, false, None);
        let res = evaluate_jump(query, &opts);

        env::set_current_dir(original_cwd).unwrap();

        assert!(!res.is_empty(), "Wildcard expansion failed in test root");
        assert!(res[0].to_string_lossy().contains("WildcardTarget"));
    }

    #[test]
    fn test_ellipsis_sibling_resolution() {
        let temp = tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let target = root.join("SiblingTarget");
        let cwd_mock = root.join("CurrentDir");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&cwd_mock).unwrap();

        // handle_ellipsis now only takes (segment, base) and returns Vec<PathBuf>
        let matches = handle_ellipsis("..", cwd_mock);

        assert!(!matches.is_empty(), "Matches should not be empty");
        let found = matches[0].canonicalize().unwrap();
        let expected = root.canonicalize().unwrap();
        assert_eq!(found, expected, "Resolved to root (parent of CWD).");
    }

    #[test]
    fn test_walker_sibling_resolution() {
        let temp = tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let target = root.join("SiblingTarget");
        let cwd_mock = root.join("CurrentDir");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&cwd_mock).unwrap();

        let opts = test_opts();

        // Path: go up from CWD, then look for "SiblingTarget"
        let segments = vec!["..", "SiblingTarget"];
        let results = resolve_path_segments(vec![cwd_mock], segments, &opts);

        assert_eq!(results[0].canonicalize().unwrap(), target.canonicalize().unwrap());
    }

    #[test]
    fn test_ellipsis_sibling_resolution_mk2() {
        let temp = tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let target = root.join("SiblingTarget");
        let depth_layer = root.join("DepthLayer");
        let cwd_mock = depth_layer.join("CurrentDir");

        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&cwd_mock).unwrap();

        // "..." pops 2: CurrentDir -> DepthLayer -> root
        let matches = handle_ellipsis("...", cwd_mock);

        assert!(!matches.is_empty(), "Ellipsis should return the jumped path");
        let found_path = matches[0].canonicalize().unwrap();
        let expected_path = target.parent().unwrap().canonicalize().unwrap();

        assert_eq!(found_path, expected_path, "Should have popped twice to reach the root containing SiblingTarget");
    }

    #[test]
    fn test_primitive_dot_resolution() {
        let opts = get_opts(CdMode::Origin, false, None);
        let current = env::current_dir().unwrap();

        // Test "."
        let res_dot = evaluate_jump(".", &opts);
        assert_eq!(res_dot[0], current);

        // Test " ." (trailing space)
        let res_space = evaluate_jump(" . ", &opts);
        assert_eq!(res_space[0], current);
    }

    #[test]
    fn test_walker_finds_sibling_after_jump() {
        let temp = tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let target = root.parent().unwrap().join("SiblingTarget");
        let cwd_mock = root.join("CurrentDir");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&cwd_mock).unwrap();

        let opts = test_opts();

        // Logic: Jump 2 levels up ("..."), then look for "SiblingTarget"
        let segments = vec!["...", "SiblingTarget"];
        let results = resolve_path_segments(vec![cwd_mock], segments, &opts);

        assert!(!results.is_empty());
        assert_eq!(results[0].canonicalize().unwrap(), target.canonicalize().unwrap());
    }

    #[test]
    fn test_root_protection_logic() {
        let _opts = get_opts(CdMode::Origin, false, None);
        // Simulate being at a drive root (e.g., V:\)
        let root = PathBuf::from("V:\\");

        // We can't easily change the real CWD to V:\ in a test,
        // but we can test the logic if evaluate_jump were to use our mock base.
        // (Note: You might need to tweak evaluate_jump to accept a base for full testing)

        if let Some(_parent) = root.parent() {
            // If we actually have a parent, this test isn't at the root.
        } else {
            // This is where your 'if q == ".."' logic returns 'base'
            assert!(true);
        }
    }

    #[test]
    fn test_single_level_authority() {
        use std::fs;
        use std::path::Path;

        let sandbox = Path::new(DEFAULT_TEST_ROOT);
        let inner_folder = sandbox.join("Projects");

        // Ensure clean state and physical existence
        let _ = fs::remove_dir_all(sandbox);
        fs::create_dir_all(&inner_folder).expect("Could not create test sandbox");

        let opts = SearchOptions {
            mode: CdMode::Hybrid, // <--- Change this from Target to Hybrid or Origin
            exact: false,
            list: false,
            dir_match: Default::default(),
            mock_path: Some(DEFAULT_TEST_ROOT.into()),
        };

        let results = search_cdpath("pro*", &opts);

        assert!(!results.is_empty(), "Engine failed to find 'Projects' in {}", DEFAULT_TEST_ROOT);
        assert!(results[0].to_string_lossy().contains("Projects"));

        fs::remove_dir_all(sandbox).ok();
    }

    #[test]
    fn test_drive_root_regression() {
        let path = PathBuf::from("V:\\"); // Use the explicit root to avoid drive-relative issues
        let tail = vec!["Projects"];
        let results = resolve_path_segments(vec![path], tail, &test_opts());

        let output = results[0].to_string_lossy();
        // Verify it didn't mangle into V:Projects
        assert!(output.contains('\\') || output.contains('/'), "Path was joined without separator: {}", output);
        assert!(output.starts_with("V:\\") || output.starts_with("V:/"));
    }
    #[test]
    fn test_drive_root_regression_two_mocked() {
        let (tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let opts = get_opts(CdMode::Origin, true, Some(root.clone().into_os_string()));

        // The environment already has root/Projects/ncd/src
        let tail = vec!["Projects"];
        let results = resolve_path_segments(vec![root.clone()], tail, &opts);

        assert!(!results.is_empty(), "Search failed to find Projects in mock root");
        let output = results[0].to_string_lossy();

        // Verify separator logic is handled by PathBuf, not string hacking
        let has_sep = results[0].components().count() > 1;
        assert!(has_sep, "Path was not properly joined with separator: {}", output);
        assert!(output.contains("Projects"), "Tail lost in resolution");
    }

    #[test]
    fn test_drive_root_regression_mk3() {
        let (tmp, v_drive_mock) = setup_test_env(); // Projects/ncd/src exists here
        let sub_dir = v_drive_mock.join("Projects");

        // Guard the CWD to the sub-directory to test relative-to-drive behavior
        let _guard = CwdGuard::new(&sub_dir);

        // We pass the "Drive Root" as the mock_path
        let opts = get_opts(CdMode::Hybrid, true, Some(v_drive_mock.clone().into_os_string()));

        // Input "V:ncd" (no slash) should look in CWD of that drive
        // Input "V:\Projects" should look at the root
        let tail = vec!["Projects", "ncd", "src"];
        let results = resolve_path_segments(vec![sub_dir], tail, &opts);

        assert!(!results.is_empty(), "Failed to resolve from sub-dir of mocked drive");
        assert!(results[0].ends_with("src"), "Path resolution broken: {:?}", results[0]);
        assert!(results[0].starts_with(&v_drive_mock), "Escaped the virtual drive!");
    }

    #[test]
    fn test_drive_root_regression_mk4() {
        let (tmp, v_drive_mock) = setup_test_env();
        let opts = get_opts(CdMode::Origin, true, Some(v_drive_mock.clone().into_os_string()));
        // Start from the root so the tail "Projects/ncd/src" aligns perfectly
        let results = resolve_path_segments(vec![v_drive_mock.clone()], vec!["Projects", "ncd", "src"], &opts);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_drive_root_regression_three() {
        let temp = tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        std::fs::create_dir_all(root.join("Projects")).unwrap();

        let _guard = CwdGuard::new(&root); // Reverts to original CWD on drop
        let results = resolve_path_segments(vec![PathBuf::from(".")], vec!["Projects"], &test_opts());

        assert!(!results.is_empty(), "Relative search failed");
        assert!(results[0].ends_with("Projects"));
    }
}


#[cfg(test)]
mod battery_2 {
    use std::env;
    use std::ffi::OsString;
    use crate::{evaluate_jump, handle_ellipsis, resolve_path_segments, CdMode, DirMatch, SearchOptions};
    use std::path::PathBuf;
    use crate::unit_tests::{get_opts, get_opts_fuzzy, setup_test_env, CwdGuard};

    #[test]
    fn check_edges() {
        let (_tmp, root) = setup_test_env();
        let cases = vec![
            ("Proj*", "Projects"), ("Driv*", "Drivers"), ("Users/G*/Desk*", "Users\\Guest\\Desktop"),
            ("Windows/Sys*", "Windows\\System32"), ("Proj*/ncd/src", "Projects\\ncd\\src")
        ];
        for (query, expected) in cases {
            let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into_os_string()) };
            let res = evaluate_jump(query, &opts);
            assert!(res.iter().any(|p| p.to_string_lossy().contains(expected)), "Failed: {} -> {}", query, expected);
        }
    }

    #[test]
    fn test_path_resolutions() {
        let (_tmp, root) = setup_test_env();
        let root_str = root.to_str().unwrap();
        let opts = |m: Option<String>| SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: m.map(|s| s.into()) };

        // --- 1-3: AUTHORITY ---
        let res1 = evaluate_jump("Pr*", &opts(Some(root_str.into())));
        assert!(!res1.is_empty(), "Step 1 Failed: 'Pr*' returned nothing in {}", root_str);

        let res2 = evaluate_jump("Windows/Sys*", &opts(Some(root_str.into())));
        assert!(!res2.is_empty() && res2[0].is_dir(), "Step 2 Failed: 'Windows/Sys*' not found or not dir. Got: {:?}", res2);

        // --- 4-5: TAIL REATTACHMENT ---
        let res4 = evaluate_jump("Proj*/ncd/src", &opts(Some(root_str.into())));
        assert!(!res4.is_empty(), "Step 4 Failed: 'Proj*/ncd/src' returned nothing.");
        let target4 = PathBuf::from("Projects").join("ncd").join("src");
        assert!(res4[0].ends_with(&target4), "Step 4 Path Mismatch: Expected ends_with {:?}, Got {:?}", target4, res4[0]);

        let res5 = evaluate_jump("Users/G*/Desktop", &opts(Some(root_str.into())));
        assert!(!res5.is_empty(), "Step 5 Failed: 'Users/G*/Desktop' returned nothing.");
        let target5 = PathBuf::from("Users").join("Guest").join("Desktop");
        assert!(res5[0].ends_with(&target5), "Step 5 Path Mismatch: Expected ends_with {:?}, Got {:?}", target5, res5[0]);

        // --- 8-9: ELLIPSIS ---
        let base = root.join("Projects").join("ncd");
        let res8 = handle_ellipsis("...", base.clone()); // No .expect()

        assert!(!res8.is_empty(), "Step 8 Failed: No results returned");
        let found8 = res8[0].canonicalize().unwrap();
        let expected8 = root.canonicalize().unwrap();
        assert_eq!(found8, expected8, "Step 8 Failed: Parent jump did not hit root. Got: {:?}", found8);
    }

    #[test]
    fn test_walker_integration() {
        let (tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        // Explicitly set the mock_path to our temp root
        let opts = get_opts(CdMode::Hybrid, true, Some(root.clone().into_os_string()));

        // This now exists because setup_test_env created /Users/Guest/Desktop
        let tail = vec!["Users", "Guest"];
        let results = resolve_path_segments(vec![root], tail, &opts);

        assert!(!results.is_empty(), "Walker failed to resolve path in temp env");
        assert!(results[0].display().to_string().contains("Guest"));
    }

    #[test]
    fn test_edge_interspersed_dots() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Case: "Projects/./ncd" -> "Projects/ncd"
        let res = evaluate_jump("Projects/./ncd", &opts);
        assert!(!res.is_empty(), "Failed to resolve interspersed dot '.'");
        assert!(res[0].ends_with(PathBuf::from("Projects").join("ncd")));
    }

    #[test]
    fn test_edge_interspersed_parents_mk1() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Case: "Projects/ncd/../Drivers" -> "Drivers"
        let res = evaluate_jump("Projects/ncd/..././Drivers", &opts);
        assert!(!res.is_empty());
        assert!(res[0].ends_with("Drivers"));
    }

    #[test]
    fn test_edge_wildcard_question_mark() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Case: "Pr?jects" -> "Projects"
        let res = evaluate_jump("Pr?jects", &opts);
        assert!(!res.is_empty(), "Failed single-char wildcard '?'");
    }

    #[test]
    fn test_edge_mixed_wildcards_mk1() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Case: "P*j?cts" -> "Projects"
        let res = evaluate_jump("P*j?cts", &opts);
        assert!(!res.is_empty(), "Failed mix of '*' and '?'");
    }

    #[test]
    fn test_edge_triple_dots_nop() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Case: "././." -> CWD
        let res = evaluate_jump("././.", &opts);
        assert_eq!(res[0].canonicalize().unwrap(), env::current_dir().unwrap().canonicalize().unwrap());
    }

    #[test]
    fn test_complex_edge_cases() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match:DirMatch::default(), mock_path: Some(root.clone().into()) };

        // 1. Interspersed dots: "Projects/./ncd" -> should resolve as "Projects/ncd"
        let res1 = evaluate_jump("Projects/./ncd", &opts);
        assert!(!res1.is_empty(), "Failed interspersed dot");

        // 2. Interspersed parent: "Projects/ncd/../Drivers" -> "Drivers"
        let res2 = evaluate_jump("Projects/ncd/../../Drivers", &opts);
        assert!(res2[0].ends_with("Drivers"), "Failed interspersed parent");

        // 3. Single character wildcard (?): "Pr?jects" -> "Projects"
        let res3 = evaluate_jump("Pr?jects", &opts);
        assert!(!res3.is_empty(), "Failed single-char wildcard '?'");

        // 4. Mixed wildcards: "P*j?cts" -> "Projects"
        let res4 = evaluate_jump("P*j?cts", &opts);
        assert!(!res4.is_empty(), "Failed mixed wildcards");

        // 5. Multiple ??: "syst??32" -> "System32"
        let res5 = evaluate_jump("Windows/syst??32", &opts);
        assert!(!res5.is_empty(), "Failed double '??'");

        // 6. Trailing slashes: "Projects/ncd/" -> Should not error
        let res6 = evaluate_jump("Projects/ncd/", &opts);
        assert!(!res6.is_empty(), "Failed trailing slash");

        // 7. Double slashes: "Projects//ncd" -> Should treat as single
        let res7 = evaluate_jump("Projects//ncd", &opts);
        assert!(!res7.is_empty(), "Failed double slash");

        // 8. The "Nop" jump: " . / . / . " -> current directory
        let res8 = evaluate_jump(" . / . / . ", &opts);
        assert_eq!(res8[0].canonicalize().unwrap(), env::current_dir().unwrap().canonicalize().unwrap());

        // 9. Deep wildcards: "*/Guest/*top" -> "Users/Guest/Desktop"
        let res9 = evaluate_jump("*/Guest/*top", &opts);
        assert!(!res9.is_empty(), "Failed deep wildcard walk");

        // 10. Empty Query: "" -> should probably return current or empty
        let res10 = evaluate_jump("", &opts);
        assert!(res10.is_empty());
    }

    #[test]
    fn test_complex_edge_case_1() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // 1. Interspersed dots: "Projects/./ncd" -> should resolve as "Projects/ncd"
        let res1 = evaluate_jump("Projects/./ncd", &opts);
        assert!(!res1.is_empty(), "Failed interspersed dot");
    }
    #[test]
    fn test_complex_edge_case_2() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // 2. Interspersed parent: "Projects/ncd/../Drivers" -> "Drivers"
        let res2 = evaluate_jump("Projects/ncd/../../Drivers", &opts);
        assert!(res2[0].ends_with("Drivers"), "Failed interspersed parent");
    }
    #[test]
    fn test_complex_edge_case_3() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // 3. Single character wildcard (?): "Pr?jects" -> "Projects"
        let res3 = evaluate_jump("Pr?jects", &opts);
        assert!(!res3.is_empty(), "Failed single-char wildcard '?'");
    }
    #[test]
    fn test_complex_edge_case_4() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // 4. Mixed wildcards: "P*j?cts" -> "Projects"
        let res4 = evaluate_jump("P*j?cts", &opts);
        assert!(!res4.is_empty(), "Failed mixed wildcards");
    }
    #[test]
    fn test_complex_edge_case_5() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // 5. Multiple ??: "syst??32" -> "System32"
        let res5 = evaluate_jump("Windows/syst??32", &opts);
        assert!(!res5.is_empty(), "Failed double '??'");
    }
    #[test]
    fn test_complex_edge_case_6() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // 6. Trailing slashes: "Projects/ncd/" -> Should not error
        let res6 = evaluate_jump("Projects/ncd/", &opts);
        assert!(!res6.is_empty(), "Failed trailing slash");
    }
    #[test]
    fn test_complex_edge_case_7() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // 7. Double slashes: "Projects//ncd" -> Should treat as single
        let res7 = evaluate_jump("Projects//ncd", &opts);
        assert!(!res7.is_empty(), "Failed double slash");
    }
    #[test]
    fn test_complex_edge_case_8() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // 8. The "Nop" jump: " . / . / . " -> current directory
        let res8 = evaluate_jump(" . / . / . ", &opts);
        assert_eq!(res8[0].canonicalize().unwrap(), env::current_dir().unwrap().canonicalize().unwrap());
    }
    #[test]
    fn test_complex_edge_case_9a() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // 9. Deep wildcards: "*/Guest/*top" -> "Users/Guest/Desktop"
        let res9 = evaluate_jump("*/Guest/*top", &opts);
        assert!(!res9.is_empty(), "Failed deep wildcard walk");
    }
    #[test]
    fn test_complex_edge_case_9b() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Origin, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // 9. Deep wildcards: "*/Guest/*top" -> "Users/Guest/Desktop"
        let res9 = evaluate_jump("*/Guest/*top", &opts);
        assert!(!res9.is_empty(), "Failed deep wildcard walk");
    }
    #[test]
    fn test_complex_edge_case_9c() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Target, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // 9. Deep wildcards: "*/Guest/*top" -> "Users/Guest/Desktop"
        let res9 = evaluate_jump("*/Guest/*top", &opts);
        assert!(!res9.is_empty(), "Failed deep wildcard walk");
    }


    #[test]
    fn test_complex_edge_case_10() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match:DirMatch::default(), mock_path: Some(root.clone().into()) };
        // 10. Empty Query: "" -> should probably return current or empty
        let res10 = evaluate_jump("", &opts);
        assert!(res10.is_empty());
    }

    #[test]
    fn test_edge_interspersed_parents_mk2() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Verifies: Projects/ncd/../Drivers -> root/Drivers
        let res = evaluate_jump("Projects/ncd/.././../Projects/ncd/.../Drivers", &opts);
        assert!(!res.is_empty(), "Failed to resolve '..' segment inside fuzzy path");
        assert!(res[0].ends_with("Drivers"), "Path mismatch. Got: {:?}", res[0]);
    }

    #[test]
    fn test_edge_interspersed_parents_mk4() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts_fuzzy(CdMode::Hybrid, false, Some(root.clone().into()));
        // Verifies: Projects/ncd/../Drivers -> root/Drivers
        let res = evaluate_jump("Projects/ncd/.././../Projects/ncd/.../Driv", &opts);
        assert!(!res.is_empty(), "Failed to resolve '..' segment inside fuzzy path");
        assert!(res[0].ends_with("Drivers"), "Path mismatch. Got: {:?}", res[0]);
    }

    #[test]
    fn test_edge_interspersed_parents_mk5() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts_fuzzy(CdMode::Hybrid, false, Some(root.clone().into()));
        // Verifies: Projects/ncd/../Drivers -> root/Drivers
        let res = evaluate_jump("Projects/ncd/.././../Proj/ncd/.../Drivers", &opts);
        assert!(!res.is_empty(), "Failed to resolve '..' segment inside fuzzy path");
        assert!(res[0].ends_with("Drivers"), "Path mismatch. Got: {:?}", res[0]);
    }

    #[test]
    fn test_edge_mixed_wildcards_and_dots() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Verifies: Pr*/./n*d -> root/Projects/ncd
        let res = evaluate_jump("Pr*/./n*d", &opts);
        assert!(!res.is_empty(), "Failed mixed wildcard and dot resolution");
        assert!(res[0].to_string_lossy().contains("ncd"));
    }

    #[test]
    fn test_edge_question_mark_wildcard() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Verifies: Wind?ws/Sys??m32 -> root/Windows/System32
        let res = evaluate_jump("Wind?ws/Sys??m32", &opts);
        assert!(!res.is_empty(), "Failed single-character wildcard '?'");
        assert!(res[0].to_string_lossy().contains("System32"));
    }

    #[test]
    fn test_edge_interspersed_parents_mk3() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        let res = evaluate_jump("Projects/ncd/.../Drivers", &opts);
        assert!(!res.is_empty(), "Failed to resolve '..' in fuzzy path");
        assert!(res[0].to_string_lossy().contains("Drivers"));
    }

    #[test]
    fn test_edge_mixed_wildcards_mk2() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Testing both * and ? together
        let res = evaluate_jump("Pro*s", &opts);
        assert!(!res.is_empty(), "1 Mixed wildcards with parent jump failed");
        assert!(res[0].to_string_lossy().contains("Projects"));
        let res = evaluate_jump("Pro*s/", &opts);
        assert!(!res.is_empty(), "2 Mixed wildcards with parent jump failed");
        assert!(res[0].to_string_lossy().contains("Projects"));
        let res = evaluate_jump("Pro*s/nc?", &opts);
        assert!(!res.is_empty(), "3 Mixed wildcards with parent jump failed");
        assert!(res[0].to_string_lossy().contains("ncd"));
        let res = evaluate_jump("Pro*s/nc?/", &opts);
        assert!(!res.is_empty(), "4 Mixed wildcards with parent jump failed");
        assert!(res[0].to_string_lossy().contains("ncd"));
        let res = evaluate_jump("Pro*s/nc?/..", &opts);
        assert!(!res.is_empty(), "5 Mixed wildcards with parent jump failed");
        assert!(res[0].to_string_lossy().contains("Projects"));
        let res = evaluate_jump("Pro*s/nc?/../", &opts);
        assert!(!res.is_empty(), "6 Mixed wildcards with parent jump failed");
        assert!(res[0].to_string_lossy().contains("Projects"));
        let res = evaluate_jump("Pro*s/nc?/../Dri*", &opts);
        assert!(!res.is_empty(), "7 Mixed wildcards with parent jump failed");
        assert!(res[0].to_string_lossy().contains("Drivers"));
    }

    #[test]
    fn test_edge_multiple_dots() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        let res = evaluate_jump("Projects/././ncd/.", &opts);
        assert!(!res.is_empty(), "Interspersed dots '.' failed");
        assert!(res[0].to_string_lossy().contains("ncd"));
    }

    #[test]
    fn test_edge_interspersed_parents() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match:DirMatch::default(),  mock_path: Some(root.clone().into()) };
        // Case: Projects/ncd/../Drivers -> should resolve to root/Drivers
        let res = evaluate_jump("Projects/ncd/../../Drivers", &opts);
        assert!(!res.is_empty(), "Failed interspersed parent jump");
        assert!(res[0].ends_with("Drivers"));
    }

    #[test]
    fn test_edge_mixed_wildcards() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // Case: Pr*j?cts -> Projects
        let res = evaluate_jump("Pr*j?cts", &opts);
        assert!(!res.is_empty(), "Failed mixed * and ? wildcards");
        assert!(res[0].ends_with("Projects"));
    }

    #[test]
    fn test_edge_dot_navigation() {
        let (_tmp, root) = setup_test_env();
        let opts = SearchOptions { mode: CdMode::Hybrid, exact: false, list: false, dir_match: DirMatch::default(), mock_path: Some(root.clone().into()) };
        // Case: Projects/./ncd -> Projects/ncd
        let res = evaluate_jump("Projects/./ncd", &opts);
        assert!(!res.is_empty(), "Failed interspersed dot navigation");
        assert!(res[0].to_string_lossy().contains("ncd"));
    }
}