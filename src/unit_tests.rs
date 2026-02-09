// src/unit_tests.rs
use std::{env, fs};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use crate::{CdMode, DirMatch, SearchOptions, DEFAULT_TEST_ROOT};

#[cfg(test)]
mod battery_1_mk1 {
    use std::fs;
    use std::path::PathBuf;
    use crate::{evaluate_jump, handle_ellipsis, resolve_path_segments, CdMode};
    use crate::unit_tests_local::{get_opts, setup_test_env, test_opts, CwdGuard};

    #[test]
    fn test_ellipsis_sibling_resolution() {
        let (_tmp, root) = setup_test_env();
        let root = root.canonicalize().unwrap();
        let target = root.join("SiblingTarget");
        let cwd_mock = root.join("CurrentDir");

        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&cwd_mock).unwrap();

        // handle_ellipsis logic check: climbing up from 'CurrentDir' to its parent
        let matches = handle_ellipsis("..", cwd_mock);

        assert!(!matches.is_empty(), "Matches should not be empty");

        let found = matches[0].canonicalize().unwrap();
        let expected = root.canonicalize().unwrap();

        assert_eq!(found, expected, "Failed to resolve to parent of CWD");
    }
    #[test]
    fn test_walker_sibling_resolution() {
        let (_tmp, root) = setup_test_env();
        let root = root.canonicalize().unwrap();
        let target = root.join("SiblingTarget");
        let cwd_mock = root.join("CurrentDir");

        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&cwd_mock).unwrap();

        let opts = test_opts();

        // Path logic: go up from CWD (to root), then look for "SiblingTarget"
        let segments = vec!["..", "SiblingTarget"];
        let results = resolve_path_segments(vec![cwd_mock], segments, &opts);

        assert!(!results.is_empty(), "Failed to resolve sibling path via '..'");

        let found = results[0].canonicalize().expect("Resolved path does not exist");
        let expected = target.canonicalize().unwrap();

        assert_eq!(found, expected, "Resolved to wrong directory: {:?}", found);
    }
    #[test]
    fn test_walker_multi_step_resolution() {
        let (_tmp, root) = setup_test_env();
        // tree: root/Projects/ncd/src
        let start_dir = root.join("Projects");
        let target = root.join("Projects/ncd/src");

        let opts = test_opts();
        let segments = vec!["ncd", "src"];
        let results = resolve_path_segments(vec![start_dir], segments, &opts);

        assert!(!results.is_empty());
        assert_eq!(results[0].canonicalize().unwrap(), target.canonicalize().unwrap());
    }
    #[test]
    fn test_ellipsis_sibling_resolution_mk2() {
        let (_tmp, root) = setup_test_env();
        let root = root.canonicalize().unwrap();
        let target = root.join("SiblingTarget");
        let depth_layer = root.join("DepthLayer");
        let cwd_mock = depth_layer.join("CurrentDir");

        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&cwd_mock).unwrap();

        // "..." pops 2 levels: CurrentDir -> DepthLayer -> root
        let matches = handle_ellipsis("...", cwd_mock);

        assert!(!matches.is_empty(), "Ellipsis should return the jumped path");

        let found_path = matches[0].canonicalize().expect("Resolved path not found");
        let expected_path = root.clone();

        assert_eq!(found_path, expected_path, "Should have popped twice to reach the root");
    }
    #[test]
    fn test_primitive_dot_resolution() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));

        // Test "." - current directory resolution
        let res_dot = evaluate_jump(".", &opts);
        assert!(!res_dot.is_empty(), "Failed to resolve single dot");
        assert_eq!(res_dot[0], root, "Single dot did not resolve to CWD");

        // Test " ." - check if the engine handles leading/trailing whitespace
        let res_space = evaluate_jump(" . ", &opts);
        assert!(!res_space.is_empty(), "Failed to resolve dot with whitespace");
        assert_eq!(res_space[0], root, "Whitespace dot did not resolve to CWD");
    }
    #[test]
    fn test_walker_finds_sibling_after_jump() {
        let (_tmp, root) = setup_test_env();
        let root = root.canonicalize().unwrap();

        // Structure: root/SiblingTarget and root/Depth/CurrentDir
        let target = root.join("SiblingTarget");
        let cwd_mock = root.join("Depth/CurrentDir");

        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&cwd_mock).unwrap();

        let opts = test_opts();

        // Logic: Jump 2 levels up ("...": CurrentDir -> Depth -> root), then find "SiblingTarget"
        let segments = vec!["...", "SiblingTarget"];
        let results = resolve_path_segments(vec![cwd_mock], segments, &opts);

        assert!(!results.is_empty(), "Walker failed to find sibling after ellipsis jump");

        let found = results[0].canonicalize().expect("Resolved path missing");
        let expected = target.canonicalize().unwrap();

        assert_eq!(found, expected, "Walker resolved to wrong sibling path");
    }
}

mod battery_1_mk2 {
    use std::{fs};
    use crate::*;
    use crate::unit_tests_local::{get_opts, get_opts_fuzzy, setup_test_env, CwdGuard};
    #[test]
    fn test_primitive_dot_resolution() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));

        // Test "." - current directory resolution
        let res_dot = evaluate_jump(".", &opts);
        assert!(!res_dot.is_empty(), "Failed to resolve single dot");
        assert_eq!(res_dot[0], root, "Single dot did not resolve to CWD");

        // Test " ." - check if the engine handles leading/trailing whitespace
        let res_space = evaluate_jump(" . ", &opts);
        assert!(!res_space.is_empty(), "Failed to resolve dot with whitespace");
        assert_eq!(res_space[0], root, "Whitespace dot did not resolve to CWD");
    }
    #[test]
    fn test_ellipsis_sibling_resolution() {
        let (_tmp, root) = setup_test_env();
        let root = root.canonicalize().unwrap();
        let target = root.join("SiblingTarget");
        let cwd_mock = root.join("CurrentDir");

        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&cwd_mock).unwrap();

        // handle_ellipsis logic check: climbing up from 'CurrentDir' to its parent
        let matches = handle_ellipsis("..", cwd_mock);

        assert!(!matches.is_empty(), "Matches should not be empty");

        let found = matches[0].canonicalize().unwrap();
        let expected = root.canonicalize().unwrap();

        assert_eq!(found, expected, "Failed to resolve to parent of CWD");
    }
    #[test]
    fn test_wildcard_regex_logic() {
        let (_tmp, root) = setup_test_env();
        // Create a specific pattern-matchable directory in the mock root
        let target = root.join("testing.1");
        fs::create_dir_all(&target).unwrap();

        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));
        let res = search_cdpath("test*.*", &opts);

        assert!(!res.is_empty(), "Wildcard search failed for 'test*.*'");
        let path_str = res[0].to_string_lossy();
        assert!(path_str.contains("testing.1"), "Resolved path '{}' did not match pattern", path_str);
        assert!(res[0].starts_with(&root), "Wildcard search escaped mock root");
    }
    #[test]
    fn test_cdpath_exact_vs_fuzzy() {
        let (_tmp, root) = setup_test_env();
        let name = "MixedCase123";
        let proj = root.join(name);
        fs::create_dir_all(&proj).unwrap();

        let mock = Some(root.into_os_string());

        // Test Fuzzy logic using the specific fuzzy helper
        let opts_f = get_opts_fuzzy(CdMode::Origin, false, mock.clone());
        let res_f = search_cdpath("mixedcase123", &opts_f);
        assert!(!res_f.is_empty(), "Fuzzy match failed for lowercase input");

        // Test Exact logic
        let opts_e = get_opts(CdMode::Origin, true, mock);
        let res_e = search_cdpath("mixedcase123", &opts_e);

        // On case-sensitive filesystems (Linux CI), exact match for wrong case must fail
        if cfg!(unix) {
            assert!(res_e.is_empty(), "Exact match should have failed on case-sensitive OS");
        } else {
            // Windows behavior depends on actual disk casing
            let actual_name = proj.file_name().unwrap().to_str().unwrap();
            if actual_name != "mixedcase123" {
                assert!(res_e.is_empty());
            }
        }
    }
    #[test]
    fn test_dot_traversal() {
        let (_tmp, root) = setup_test_env();
        let root2 = root.clone();
        // Move deep into the mock tree so '...' has room to move up
        let deep_dir = root.join("Projects/ncd/src");
        let _guard = CwdGuard::new(&deep_dir);

        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));
        let result = evaluate_jump("...", &opts);

        assert!(!result.is_empty(), "Triple dot traversal failed");

        // '...' should go up two levels: src -> ncd -> Projects
        let expected = root2.join("Projects");
        assert_eq!(result[0], expected, "Did not climb exactly two levels up");
    }
    #[test]
    fn test_wildcard_case_insensitivity() {
        let (_tmp, root) = setup_test_env();
        let test_dir = root.join("CaseSensitiveDir");
        fs::create_dir_all(&test_dir).unwrap();

        let _guard = CwdGuard::new(&root);
        let opts = get_opts_fuzzy(CdMode::Hybrid, false, Some(root.into_os_string()));

        // Search using lowercase on a potentially uppercase directory
        let res = evaluate_jump("casesensitive*", &opts);

        assert!(!res.is_empty(), "Fuzzy wildcard failed to ignore case");
        assert!(res[0].to_string_lossy().contains("CaseSensitiveDir"));
    }
    #[test]
    fn test_cdpath_exact_vs_fuzzy_tmp() {
        let (_tmp, root) = setup_test_env();
        let proj_path = root.join("MyProject");
        fs::create_dir(&proj_path).unwrap();
        let mock_env = Some(root.into_os_string());

        // Fuzzy match: should find "MyProject" from "myproject"
        let opts_f = get_opts_fuzzy(CdMode::Origin, false, mock_env.clone());
        let res_fuzzy = search_cdpath("myproject", &opts_f);
        assert!(!res_fuzzy.is_empty(), "Fuzzy search should be case-insensitive");

        // Exact match: should fail on case-sensitive systems (Linux/GitHub)
        let opts_e = get_opts(CdMode::Origin, true, mock_env);
        let res_exact = search_cdpath("myproject", &opts_e);

        if cfg!(unix) {
            assert!(res_exact.is_empty(), "Exact match should fail on case-sensitive OS for 'myproject'");
        }
    }
}

mod battery_1_mk3 {
    use std::fs;
    use std::path::PathBuf;
    use crate::{evaluate_jump, resolve_path_segments, search_cdpath, CdMode};
    use crate::unit_tests_local::{get_opts, setup_test_env, test_opts, CwdGuard};

    #[test]
    fn test_walker_jump_persistence() {
        let (_tmp, root) = setup_test_env();
        let start = root.join("Projects/ncd/src");
        // Jump '..' to ncd, then find 'src' again
        let segments = vec!["..", "src"];

        let results = resolve_path_segments(vec![start.clone()], segments, &test_opts());

        assert!(!results.is_empty());
        assert_eq!(results[0].canonicalize().unwrap(), start.canonicalize().unwrap());
    }
    #[test]
    fn test_single_level_authority() {
        // Use setup_test_env to avoid DEFAULT_TEST_ROOT permission issues on CI
        let (_tmp, root) = setup_test_env();
        let inner_folder = root.join("Projects");
        fs::create_dir_all(&inner_folder).expect("Could not create test sandbox");

        // Use the standard get_opts to ensure library defaults are applied
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into_os_string()));

        let results = search_cdpath("pro*", &opts);

        assert!(!results.is_empty(), "Engine failed to find 'Projects' in mock root");
        let path_str = results[0].to_string_lossy();
        assert!(path_str.contains("Projects"), "Result '{}' missing expected folder", path_str);

        // Final check that the authority of the mock_path was respected
        assert!(results[0].starts_with(&root), "Search results escaped the authority of the mock root");
    }
    #[test]
    fn test_drive_root_regression() {
        let (_tmp, root) = setup_test_env();
        let tail = vec!["Projects"];

        // Use the dynamic root to ensure the engine doesn't mangle separators
        // like the old "V:Projects" vs "V:\Projects" bug.
        let results = resolve_path_segments(vec![root.clone()], tail, &test_opts());

        assert!(!results.is_empty(), "Resolution failed");
        let output = results[0].to_string_lossy();

        // Verify separator logic: The resulting path must be a proper child of root
        assert!(results[0].starts_with(&root), "Path lost its root prefix: {}", output);
        assert!(results[0].ends_with("Projects"), "Path lost its tail: {}", output);

        // Ensure standard PathBuf joining (no manual string hacking)
        assert_eq!(results[0], root.join("Projects"));
    }
    #[test]
    fn test_drive_root_regression_two_mocked() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let opts = get_opts(CdMode::Origin, true, Some(root.clone().into_os_string()));

        // The environment already has root/Projects/ncd/src
        let tail = vec!["Projects"];
        let results = resolve_path_segments(vec![root.clone()], tail, &opts);

        assert!(!results.is_empty(), "Search failed to find Projects in mock root");
        let output = results[0].to_string_lossy();

        // Check absolute path integrity instead of raw component counting
        assert!(results[0].is_absolute(), "Path must remain absolute: {}", output);
        assert!(results[0].ends_with("Projects"), "Tail lost in resolution: {}", output);

        // Ensure the path is physically reachable in the temp env
        assert!(results[0].exists(), "Resolved path does not exist: {}", output);
    }
    #[test]
    fn test_drive_root_regression_mk3() {
        let (_tmp, v_drive_mock) = setup_test_env(); // Projects/ncd/src exists here
        let sub_dir = v_drive_mock.join("Projects");

        // Guard the CWD to the sub-directory to test relative-to-drive behavior
        let _guard = CwdGuard::new(&sub_dir);

        // We pass the "Drive Root" as the mock_path to simulate a root boundary
        let opts = get_opts(CdMode::Hybrid, true, Some(v_drive_mock.clone().into_os_string()));

        // The tail represents a deep path starting from the current sub_dir
        let tail = vec!["Projects", "ncd", "src"];
        let results = resolve_path_segments(vec![sub_dir], tail, &opts);

        assert!(!results.is_empty(), "Failed to resolve from sub-dir of mocked drive");
        assert!(results[0].ends_with("src"), "Path resolution broken: {:?}", results[0]);

        // Critical CI Check: Ensure the resolution logic didn't pop above the temp root
        assert!(results[0].starts_with(&v_drive_mock), "Escaped the virtual drive boundary!");
    }
    #[test]
    fn test_drive_root_regression_mk4() {
        let (_tmp, v_drive_mock) = setup_test_env();
        let opts = get_opts(CdMode::Origin, true, Some(v_drive_mock.clone().into_os_string()));

        // Start from the root so the tail "Projects/ncd/src" aligns perfectly
        let tail = vec!["Projects", "ncd", "src"];
        let results = resolve_path_segments(vec![v_drive_mock.clone()], tail, &opts);

        assert!(!results.is_empty(), "Failed to resolve perfect-fit tail from root");

        // Ensure the path is valid on the current OS (handles / vs \ automatically)
        let expected = v_drive_mock.join("Projects").join("ncd").join("src");
        assert_eq!(results[0], expected, "Resolved path does not match OS-specific expected path");
    }
    #[test]
    fn test_dot_slash_resolution() {
        let (_tmp, root) = setup_test_env();
        let target = root.join("Projects");
        std::fs::create_dir_all(&target).unwrap();
        let _guard = CwdGuard::new(&root);

        // Mimic a query like "./Projects"
        let segments = vec![".", "Projects"];
        let results = resolve_path_segments(vec![root.clone()], segments, &test_opts());

        assert!(!results.is_empty());
        assert_eq!(results[0].canonicalize().unwrap(), target.canonicalize().unwrap());
    }
}

mod battery_1_mk4 {
    use std::{env, fs};
    use serial_test::serial;
    use crate::{evaluate_jump, search_cdpath, CdMode};
    use crate::unit_tests_local::{get_opts, get_test_root, setup_test_env, CwdGuard};

    #[test]
    fn test_junction_follow() {
        let (_tmp, root) = setup_test_env();
        let target = root.join("RealDir");
        let link = root.join("JunctionFollow");

        fs::create_dir_all(&target).ok();
        // Create an OS-specific link to test the "follow" logic
        #[cfg(windows)] { let _ = std::os::windows::fs::symlink_dir(&target, &link); }
        #[cfg(unix)] { let _ = std::os::unix::fs::symlink(&target, &link); }

        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));
        let res = search_cdpath("JunctionFollow", &opts);

        assert!(!res.is_empty(), "Failed to find junction/link in mock root");
        assert!(res[0].ends_with("JunctionFollow"));
    }
    #[test]
    fn test_extreme_ellipsis() {
        let (_tmp, root) = setup_test_env();
        // Setup a path deep enough for 4 levels of ascent: root/Users/Guest/Desktop
        let deep_dir = root.join("Users/Guest/Desktop");
        let _guard = CwdGuard::new(&deep_dir);

        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));
        let result = evaluate_jump(".....", &opts);

        assert!(!result.is_empty(), "Extreme ellipsis failed to return a path");

        // Calculation: Desktop(0) -> Guest(1) -> Users(2) -> root(3) -> parent_of_root(4)
        let mut expected = deep_dir.clone();
        for _ in 0..4 {
            if let Some(parent) = expected.parent() {
                expected = parent.to_path_buf();
            }
        }

        assert_eq!(result[0], expected, "Extreme ellipsis did not reach expected parent");
    }
    #[test]
    fn test_hybrid_mode() {
        let (_tmp, root) = setup_test_env();
        let bookmark = root.join("Work");
        fs::create_dir(&bookmark).unwrap();

        // Pass the specific bookmark folder as the CDPATH mock
        let opts = get_opts(CdMode::Hybrid, true, Some(bookmark.clone().into_os_string()));
        let res = search_cdpath("Work", &opts);

        assert!(!res.is_empty(), "Hybrid mode failed to find directory in CDPATH");
        assert_eq!(
            res[0].canonicalize().unwrap(),
            bookmark.canonicalize().unwrap(),
            "Resolved path does not match the bookmarked directory"
        );
    }
    #[test]
    fn test_hybrid_mode_priority() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root); // Stay at root

        // Projects exists in the mock env root.
        // We want to ensure Hybrid mode finds it via CDPATH or relative logic.
        let opts = get_opts(CdMode::Hybrid, true, Some(root.clone().into_os_string()));
        let res = search_cdpath("Projects", &opts);

        assert!(!res.is_empty(), "Hybrid mode failed to resolve 'Projects'");
        assert!(res[0].ends_with("Projects"));
        assert!(res[0].starts_with(&root));
    }
    #[test]
    fn test_root_anchored_logic_mk2() {
        let (_tmp, root) = setup_test_env();
        // Passing 'root' as mock_path allows the engine to treat the temp dir as the drive root
        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));

        let query = format!("{}Projects", std::path::MAIN_SEPARATOR);
        let result = evaluate_jump(&query, &opts);

        assert!(!result.is_empty(), "Search failed to return results for root anchor");

        let path = &result[0];
        let path_str = path.to_string_lossy();

        assert!(path_str.contains("Projects"), "Path missing 'Projects': {}", path_str);
        assert!(path.is_absolute(), "Resulting path must be absolute");
    }
    #[test]
    fn test_wildcard_regex_logic() {
        let (_tmp, root) = setup_test_env();
        // Create a specific pattern-matchable directory in the mock root
        let target = root.join("testing.1");
        fs::create_dir_all(&target).unwrap();

        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));
        let res = search_cdpath("test*.*", &opts);

        assert!(!res.is_empty(), "Wildcard search failed for 'test*.*'");
        let path_str = res[0].to_string_lossy();
        assert!(path_str.contains("testing.1"), "Resolved path '{}' did not match pattern", path_str);
        assert!(res[0].starts_with(&root), "Wildcard search escaped mock root");
    }
    #[serial]
    #[test]
    fn test_parent_globbing() {
        let (_tmp, root) = setup_test_env();
        let parent = root.join("parent_dir");
        let child = parent.join("child_glob");
        fs::create_dir_all(&child).unwrap();

        // Use CwdGuard to handle environment cleanup automatically
        let _guard = CwdGuard::new(&child);

        // Construct a portable query using the system's main separator
        let query = format!("..{}child*", std::path::MAIN_SEPARATOR);
        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));

        let res = evaluate_jump(&query, &opts);

        assert!(!res.is_empty(), "Parent globbing failed for query: {}", query);
        let path_str = res[0].to_string_lossy();
        assert!(path_str.contains("child_glob"), "Result path '{}' does not contain 'child_glob'", path_str);
    }
    #[serial]
    #[test]
    fn test_root_anchored_wildcard_mk2() {
        let (_tmp, root) = setup_test_env();
        let test_dir = root.join("WildcardTarget");
        fs::create_dir_all(&test_dir).unwrap();

        // CwdGuard handles the push/pop of the directory automatically for CI safety
        let _guard = CwdGuard::new(&root);

        let query = "Wildcard*";
        // Pass the temp root as the mock_path to anchor the search correctly
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into_os_string()));
        let res = evaluate_jump(query, &opts);

        assert!(!res.is_empty(), "Wildcard expansion failed for query: {}", query);
        assert!(res[0].to_string_lossy().contains("WildcardTarget"));
        assert!(res[0].starts_with(&root), "Wildcard resolution escaped the mock root");
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
}
#[cfg(test)]
mod battery_2 {
    use std::env;
    use crate::{evaluate_jump, handle_ellipsis, resolve_path_segments, CdMode, DirMatch, SearchOptions};
    use std::path::PathBuf;
    use crate::unit_tests_local::{get_opts, get_opts_fuzzy, setup_test_env, CwdGuard};

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
        let (_tmp, root) = setup_test_env();
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
        let res = evaluate_jump("Pro*s/nc?/.../Dri*", &opts);
        assert!(!res.is_empty(), "7 Mixed wildcards with parent jump failed");
        assert!(res[0].to_string_lossy().contains("Drivers"));
    }

    #[test]
    fn test_edge_mixed_wildcards_mk3() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Testing both * and ? together
        let res = evaluate_jump("Projects/././ncd/./../ncd", &opts);
        assert!(!res.is_empty(), "1 Mixed wildcards with parent jump failed");
        assert!(res[0].to_string_lossy().contains("ncd"));

    }

    #[test]
    fn test_edge_mixed_wildcards_mk4() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Testing both * and ? together
        let res = evaluate_jump("Projec*/././ncd/./../ncd", &opts);
        assert!(!res.is_empty(), "1 Mixed wildcards with parent jump failed");
        assert!(res[0].to_string_lossy().contains("ncd"));

    }

    #[test]
    fn test_edge_mixed_wildcards_mk5() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Testing both * and ? together
        let res = evaluate_jump("Projec*/././ncd/./../nc?", &opts);
        assert!(!res.is_empty(), "1 Mixed wildcards with parent jump failed");
        assert!(res[0].to_string_lossy().contains("ncd"));

    }

    #[test]
    fn test_edge_mixed_wildcards_mk6() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Testing both * and ? together
        let res = evaluate_jump("Projec*/././ncd/./../nc*", &opts);
        assert!(!res.is_empty(), "1 Mixed wildcards with parent jump failed");
        assert!(res[0].to_string_lossy().contains("ncd"));

    }

    #[test]
    fn test_edge_mixed_wildcards_mk7() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));
        // Testing both * and ? together
        let res = evaluate_jump("Projec*/././ncd/./../nc*/.../Drivers", &opts);
        assert!(!res.is_empty(), "1 Mixed wildcards with parent jump failed");
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

mod battery_3_elipses {
    use std::{env, fs};
    use std::path::PathBuf;
    use crate::{evaluate_jump, handle_ellipsis, resolve_path_segments, CdMode};
    use crate::unit_tests_local::{get_opts, setup_test_env, CwdGuard};

    #[test]
    fn test_hyphen_jump() {
        let (_tmp, root) = setup_test_env();
        let prev_dir = root.join("Projects");
        fs::create_dir_all(&prev_dir).unwrap();

        // Mock the OLDPWD environment variable for the hyphen jump
        std::env::set_var("OLDPWD", &prev_dir);
        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));

        let res = evaluate_jump("-", &opts);

        assert!(!res.is_empty(), "Hyphen jump failed to resolve OLDPWD");
        assert_eq!(res[0].canonicalize().unwrap(), prev_dir.canonicalize().unwrap());
    }
    #[test]
    fn test_ellipsis_climb_clamping_mk2() {
        let (_tmp, root) = setup_test_env();
        let depth_2 = root.join("Projects/ncd");

        // "...." pops 3 levels.
        // Hierarchy: ncd (0) -> Projects (1) -> root (2) -> [STOP]
        let matches = handle_ellipsis("....", depth_2);

        assert!(!matches.is_empty());
        let found = matches[0].canonicalize().unwrap();
        let expected = root.canonicalize().unwrap();

        // The engine should clamp to the root rather than continuing to D:\Users
        assert_eq!(found, expected, "Failed to clamp ellipsis traversal at root boundary");
    }
    #[test]
    fn test_ellipsis_root_identity() {
        let (_tmp, root) = setup_test_env();
        let root = root.canonicalize().unwrap();

        // Testing that the engine doesn't return an empty vec when at root
        let matches = handle_ellipsis("..", root.clone());

        assert!(!matches.is_empty(), "Engine should return the current path if it cannot go higher");
        let found = matches[0].canonicalize().unwrap();
        assert_eq!(found, root, "Engine should stay at root when climbing from root");
    }
    #[test]
    fn test_ellipsis_relative_climb() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // Start at a relative depth within the temp folder.
        let start = PathBuf::from("Projects/ncd/src");
        let matches = handle_ellipsis("....", start);

        assert!(!matches.is_empty());
        // Should land exactly on "." (the temp root).
        assert_eq!(matches[0], PathBuf::from("."), "Relative climb failed to stop at dot-root");
    }
    #[test]
    fn test_root_anchored_logic_with_cwd() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // If the query is absolute-style r"\Projects", we must decide
        // if that means the Drive root or the CWD root.
        let query = r"\Projects";
        let stripped = query.trim_start_matches('\\');

        // Joining to "." keeps us in the temp folder.
        let target = PathBuf::from(".").join(stripped);

        assert!(root.join(stripped).exists(), "Physical path should exist in temp");
        assert!(target.is_relative(), "Target should remain relative to CWD");
    }
    #[test]
    fn test_dual_floor_logic() {
        // Scenario A: Physical halt
        let mut drive = PathBuf::from(r"C:\");
        assert!(!drive.pop(), "Drive root cannot be popped");
        assert_eq!(drive, PathBuf::from(r"C:\"));

        // Scenario B: Logical halt
        let mut rel = PathBuf::from("Projects");
        assert!(rel.pop(), "Relative segment can be popped");
        assert!(rel.as_os_str().is_empty(), "Resulting buffer is empty string");
    }
    #[test]
    fn test_ellipsis_relative_to_dot() {
        // In your test environment where base is "."
        let base = PathBuf::from(".");
        let matches = handle_ellipsis("...", base);

        // 1. Pop removes "." -> Path is now empty ""
        // 2. Logic sees empty and sets it back to "."
        // 3. Break prevents further pops.
        assert_eq!(matches[0], PathBuf::from("."));
    }
    #[test]
    fn test_ellipsis_drive_persistence() {
        // On Windows, the drive is the floor.
        // We want to make sure the loop doesn't spin forever if it hits C:\
        let base = PathBuf::from(r"C:\Short\Path");
        let matches = handle_ellipsis(".......", base);

        // Should stop at C:\ and not try to become an empty string.
        assert_eq!(matches[0], PathBuf::from(r"C:\"));
        assert!(matches[0].is_absolute());
    }
    #[test]
    fn test_ellipsis_relative_floor() {
        // Relative path
        let mut base = PathBuf::from("Projects");

        // pop() returns true because it successfully removed "Projects"
        let success = base.pop();

        assert!(success, "pop() should return true when moving to empty string");
        assert!(base.as_os_str().is_empty(), "PathBuf should now be an empty string");
    }
    #[test]
    fn test_ellipsis_absolute_floor() {
        // Windows absolute path
        let mut base = PathBuf::from(r"C:\");

        // pop() returns false and DOES NOT change the path
        let success = base.pop();

        assert!(!success, "pop() should return false at the drive root");
        assert_eq!(base, PathBuf::from(r"C:\"), "PathBuf should remain at drive root");
    }
    #[test]
    fn test_ellipsis_multi_pop_safety() {
        let base = PathBuf::from("a/b");
        // "...." = 3 levels up. b -> a -> .
        let matches = handle_ellipsis("....", base);

        assert_eq!(matches[0], PathBuf::from("."), "Failed to pin at dot root after over-popping");
    }
    #[test]
    fn test_ellipsis_drive_vs_relative_stop() {
        // Case 1: Relative path
        let rel_base = PathBuf::from("Projects");
        let rel_matches = handle_ellipsis("..", rel_base);
        assert_eq!(rel_matches[0], PathBuf::from("."), "Relative should stop at dot");

        // Case 2: Absolute path (Windows)
        // Note: pop() on "C:\" does nothing and returns false,
        // but it never becomes "empty", so it stays "C:\"
        let abs_base = PathBuf::from(r"C:\");
        let abs_matches = handle_ellipsis("..", abs_base);
        assert_eq!(abs_matches[0], PathBuf::from(r"C:\"), "Absolute should stop at drive root");
    }
    #[test]
    fn test_ellipsis_deep_relative_climb() {
        let base = PathBuf::from("a/b/c");
        // "...." = 3 pops.
        // 1: a/b, 2: a, 3: (becomes empty -> fixed to ".")
        let matches = handle_ellipsis("....", base);

        assert_eq!(matches[0], PathBuf::from("."), "Should have landed on dot after 3 pops");
    }
    #[test]
    fn test_ellipsis_off_by_one_safety() {
        // Path has 1 segment.
        let base = PathBuf::from("Projects");
        // ".." means 1 pop.
        let matches = handle_ellipsis("..", base);

        // Without the is_empty() check inside the loop,
        // this would return "" instead of "."
        assert_eq!(matches[0], PathBuf::from("."), "Popping last segment should result in dot, not empty string");
    }
    #[test]
    fn test_ellipsis_exact_boundary_stop() {
        // Path with 2 segments
        let base = PathBuf::from("one/two");
        // query "...." is 3 pops: two -> one -> ""
        let matches = handle_ellipsis("....", base);

        // We expect it to stop at "." instead of disappearing into ""
        assert_eq!(matches[0], PathBuf::from("."), "Should have pinned to dot-root");
    }
    #[test]
    fn test_ellipsis_empty_base_handling() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // Scenario: Base is already empty or just a dot
        let base = PathBuf::new();
        let matches = handle_ellipsis("...", base);

        assert_eq!(matches[0], PathBuf::from("."), "Empty base should default to dot root");
    }
    #[test]
    fn test_ellipsis_drive_root_safety() {
        // Test that even on a real drive root, we don't crash or return invalid paths
        let base = PathBuf::from(r"C:\");
        let matches = handle_ellipsis("...", base);

        assert_eq!(matches[0], PathBuf::from(r"C:\"),
                   "Popping from drive root should simply return drive root");
    }
    #[test]
    fn test_ellipsis_boundary_preservation() {
        let (_tmp, root) = setup_test_env();
        // root is D:\Users\...\Temp\.tmpVRNoQd
        let start = root.join("Projects/ncd");

        // We want to go up 5 levels from a path only 2 levels deep from root.
        // It should stop at root, not D:\Users.
        let matches = handle_ellipsis("......", start);

        assert_eq!(matches[0].canonicalize().unwrap(), root.canonicalize().unwrap(),
                   "Logic overshot the test root and escaped to parent!");
    }
    #[test]
    fn test_ellipsis_pushback_logic() {
        let path = PathBuf::from("root/sub/dir");
        // query "...." is 3 pops.
        // dir -> sub -> root -> (would be empty, but we want to stay at root)
        let matches = handle_ellipsis("....", path);

        assert_eq!(matches[0], PathBuf::from("root"), "Did not preserve the logical root segment");
    }
    #[test]
    fn test_ellipsis_accumulator_boundary() {
        let (_tmp, root) = setup_test_env();
        // Assume 'root' is our "X:\"
        let start = root.join("projects/ncd");

        // We want to pop 4 times from: [root]/projects/ncd
        // 1. ncd, 2. projects, 3. [root] -> STOP.
        let matches = handle_ellipsis(".....", start.clone());

        for m in matches {
            assert!(m.starts_with(&root), "Accumulator failed to catch the overshoot!");
        }
    }
    #[test]
    fn test_ellipsis_climb_from_relative() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // Sitting in Temp, we go down:
        let relative_base = PathBuf::from("a/b/c");
        // Up 4 levels from a/b/c: c -> b -> a -> "."
        let matches = handle_ellipsis(".....", relative_base);

        assert_eq!(matches[0], PathBuf::from("."));
    }
    #[test]
    fn test_ellipsis_climb_from_absolute() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // Even if we are physically in Temp, providing an absolute path
        // to handle_ellipsis allows it to climb that specific tree.
        let absolute_base = PathBuf::from(r"X:\projects\ncd");
        let matches = handle_ellipsis(".....", absolute_base);

        // It climbs: ncd -> projects -> X:\ -> X:\
        assert_eq!(matches[0], PathBuf::from(r"X:\"));
    }
    #[test]
    fn test_ellipsis_with_guarded_cwd() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // Logic: handle_ellipsis sees "." and treats it as the start line.
        let base = PathBuf::from(".");
        let matches = handle_ellipsis("...", base);

        assert!(!matches.is_empty());
        // Should be "." or "" depending on your 5-line fix implementation.
        assert!(!matches[0].is_absolute(), "Relative base must not result in absolute escape");
    }
    #[test]
    fn test_guard_restoration() {
        let original_cwd = env::current_dir().unwrap();
        {
            let (_tmp, root) = setup_test_env();
            let _guard = CwdGuard::new(&root);
            assert_ne!(env::current_dir().unwrap(), original_cwd, "Guard failed to change directory");
        }
        // Drop happens here
        assert_eq!(env::current_dir().unwrap(), original_cwd, "Guard failed to restore directory");
    }
    #[test]
    fn test_root_anchored_logic_mk2() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // Testing the '\Projects' query again with the fix in place
        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));
        let result = evaluate_jump(r"\Projects", &opts);

        assert!(!result.is_empty(), "Result should not be empty");
        // Result should be root + Projects, not D:\Projects
        assert!(result[0].starts_with(&root), "Result escaped the mock root: {:?}", result[0]);
    }
    #[test]
    fn test_ellipsis_accumulator_boundary_mk2() {
        let (_tmp, root) = setup_test_env();
        let start = root.join("projects/ncd");

        // Pass 'root' as the floor to ensure the ellipsis stops there.
        let matches = handle_ellipsis(".....", start);

        for m in matches {
            assert_eq!(m, root, "Accumulator failed to stop at the specific mock root!");
            assert!(m.starts_with(&root), "Overshot the sandbox!");
        }
    }
    #[test]
    fn test_ellipsis_climb_clamping() {
        let (_tmp, root) = setup_test_env();
        let start = root.join("projects/ncd");

        // This query tries to go up 5 levels.
        // Our new logic in resolve_path_segments will catch the result
        // of handle_ellipsis and clamp it to 'root'.
        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));
        let result = resolve_path_segments(vec![start], vec!["....."], &opts);

        assert_eq!(result[0], root, "Did not clamp to the mock_path boundary");
    }
}

mod battery_last {
    use std::fs;
    use std::path::PathBuf;
    use crate::{evaluate_jump, handle_ellipsis, resolve_path_segments, search_cdpath, CdMode};
    use crate::unit_tests_local::{get_opts, get_opts_fuzzy, setup_test_env, test_opts, CwdGuard};

    #[test]
    fn test_ellipsis_overflow_safety() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // handle_ellipsis sees "." as the base.
        let matches = handle_ellipsis(".......", PathBuf::from("."));

        for m in matches {
            assert!(
                !m.is_absolute(),
                "Escaped sandbox! Found absolute path: {:?}", m
            );
            // The fix: We now return "." instead of "" to keep the path valid.
            assert_eq!(m, PathBuf::from("."), "Should have pinned to dot-root");
        }
    }
    #[test]
    fn test_ellipsis_traversal_limit() {
        let (_tmp, root) = setup_test_env();
        let child = root.join("Projects");
        fs::create_dir_all(&child).unwrap();

        // Starting 1 level deep, '...' (climb 2) should land exactly on root.
        let matches = handle_ellipsis("...", child);

        assert!(!matches.is_empty());
        let found = matches[0].canonicalize().unwrap();
        let expected = root.canonicalize().unwrap();

        assert_eq!(found, expected, "Should have stopped at sandbox root");
    }
    #[test]
    fn test_fuzzy_match_depth_integrity() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts_fuzzy(CdMode::Origin, false, Some(root.into_os_string()));

        // Searching for a partial match deep in the setup_test_env tree
        let res = search_cdpath("guest", &opts);

        assert!(!res.is_empty(), "Fuzzy search failed to find 'Guest' folder");
        assert!(res[0].to_string_lossy().contains("Guest"));
        assert!(res[0].is_absolute());
    }
    #[test]
    fn test_junction_traversal_integrity() {
        let (_tmp, root) = setup_test_env();
        let root2=root.clone();
        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));

        // Verify that the search actually resolves through the directory structure
        let res = search_cdpath("ncd", &opts);

        assert!(!res.is_empty(), "Search failed to traverse into Projects/ncd");
        assert!(res[0].to_string_lossy().contains("ncd"));
        assert!(res[0].starts_with(&root2), "Path escaped the temp sandbox");
    }
    #[test]
    fn test_authority_depth_limit() {
        let (_tmp, root) = setup_test_env();
        let deep = root.join("Level1/Level2/Level3");
        fs::create_dir_all(&deep).ok();

        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));
        // Searching for 'Level3' from the root authority
        let results = search_cdpath("Level3", &opts);

        assert!(!results.is_empty(), "Engine failed to traverse depth under authority");
        assert!(results[0].ends_with("Level3"));
    }
    #[test]
    fn test_boundary_ellipsis_protection() {
        let (_tmp, root) = setup_test_env();
        // Start only one level deep
        let child = root.join("Projects");
        let _guard = CwdGuard::new(&child);
        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));

        // '...' attempts to climb 2 levels, but we only have 1 level above us in the sandbox
        let res = evaluate_jump("...", &opts);

        assert!(!res.is_empty());
        // The engine should settle on the root rather than panicking or escaping to CI system temp
        assert_eq!(res[0].canonicalize().unwrap(), root.canonicalize().unwrap());
    }
    #[test]
    fn test_drive_root_regression_three() {
        let (_tmp, root) = setup_test_env();
        let root = root.canonicalize().unwrap();
        std::fs::create_dir_all(root.join("Projects")).unwrap();

        // Reverts to original CWD on drop, ensuring CI process stability
        let _guard = CwdGuard::new(&root);

        // Test relative resolution starting from the current directory "."
        let results = resolve_path_segments(vec![PathBuf::from(".")], vec!["Projects"], &test_opts());

        assert!(!results.is_empty(), "Relative search failed to resolve 'Projects' from '.'");
        assert!(results[0].ends_with("Projects"));
        assert!(results[0].is_absolute(), "Resolved path should be absolute even when starting from '.'");
    }
    #[test]
    fn test_root_protection_logic() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // Use the temp root as our logical 'V:\' anchor
        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));

        // Attempting to go '..' from our mocked root
        let res = evaluate_jump("..", &opts);

        assert!(!res.is_empty(), "Root protection should still return a valid path (the root itself)");

        // Ensure the logic doesn't escape the mock root sandbox
        let result_path = &res[0];
        assert!(
            result_path == &root || !result_path.starts_with(root.parent().unwrap_or(&root)),
            "Path escaped the protected mock root: {:?}", result_path
        );
    }
    #[test]
    fn test_dot_traversal_boundary() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // Testing '..' from the root of the mock environment
        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));
        let result = evaluate_jump("..", &opts);

        if !result.is_empty() {
            // Ensure we never climb above the mocked root if boundary logic is active
            assert!(result[0].starts_with(&root) || result[0] == root);
        }
    }
    #[test]
    fn test_parent_globbing_absolute_integrity() {
        let (_tmp, root) = setup_test_env();
        let child = root.join("parent_dir").join("child_glob");
        fs::create_dir_all(&child).unwrap();
        let _guard = CwdGuard::new(&child);

        let query = format!("..{}child*", std::path::MAIN_SEPARATOR);
        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));
        let res = evaluate_jump(&query, &opts);

        assert!(!res.is_empty());
        // Verify the path is fully resolved and absolute
        assert!(res[0].is_absolute(), "Glob resolution should return absolute paths");
        assert_eq!(res[0], child.canonicalize().unwrap_or(child));
    }
    #[test]
    fn test_root_anchored_logic() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // Match the working mk2 test exactly: use the raw string anchor
        // The mock_path in opts will redirect this from physical V:\ to your temp root
        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));
        let result = evaluate_jump(r"\Projects", &opts);

        assert!(!result.is_empty(), "Root anchor failed: result was empty for query r'\\Projects'");

        let path_str = result[0].to_string_lossy();
        assert!(path_str.contains("Projects"), "Path missing 'Projects': {}", path_str);

        // Since we are mocked, we check against the temp root, not a drive letter
        assert!(result[0].starts_with(&root), "Result stayed outside of mock root: {:?}", result[0]);
    }

}
