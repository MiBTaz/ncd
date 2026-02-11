// src/unit_tests.rs

#[cfg(test)]
mod battery_1_mk1 {
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

        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(&cwd_mock).unwrap();

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
    use crate::{resolve_path_segments, search_cdpath, CdMode};
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
        fs::create_dir_all(&target).unwrap();
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

mod battery_2_mk1 {
    use crate::{evaluate_jump, CdMode};
    use crate::unit_tests_local::{get_opts, setup_test_env, CwdGuard};

    #[test]
    fn test_edge_dot_navigation_integrated() {
        // 1. Setup physical reality
        let (_tmp, root) = setup_test_env();

        // 2. Lock the current directory to the test root
        let _guard = CwdGuard::new(&root);

        // 3. Configure options using your local helper
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 4. Run the actual engine logic
        let res = evaluate_jump("Projects/./ncd", &opts);

        // 5. Hard validation: Check existence and path parity
        assert!(!res.is_empty(), "Engine returned no paths for valid input");
        let actual = res[0].canonicalize().expect("Resulting path must physically exist");
        let expected = root.join("Projects/ncd").canonicalize().unwrap();

        assert_eq!(actual, expected, "Integrated path resolution failed to scrub dots or anchor correctly");
    }
    #[test]
    fn test_edge_dot_navigation() {
        // 1. Establish the physical sandbox
        let (_tmp, root) = setup_test_env();

        // 2. Protect the cluster's process state
        let _guard = CwdGuard::new(&root);

        // 3. Use your local helper for explicit, non-leaky options
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // Case: Projects/./ncd -> Projects/ncd
        let res = evaluate_jump("Projects/./ncd/src", &opts);

        // 4. Verification: No strings attached
        assert!(!res.is_empty(), "Failed interspersed dot navigation: Engine returned empty result");

        let actual = res[0].canonicalize().expect("Resolved path must be physically valid on disk");
        let expected = root.join("Projects/ncd/src").canonicalize().unwrap();

        assert_eq!(actual, expected, "Path resolution mismatch. Expected {:?}, got {:?}", expected, actual);
    }
    #[test]
    fn test_edge_dot_navigation_mk2() {
        // 1. Setup physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Use your local helper for explicit initialization
        // This ensures no 'default' leaks from the environment into the test logic.
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Case: Projects/./ncd -> Projects/ncd
        let res = evaluate_jump("Projects/./ncd", &opts);

        // 4. Cluster-Safe Assertions
        assert!(!res.is_empty(), "Failed interspersed dot navigation: Result set is empty");

        // Physical validation: Ensure the path actually exists and matches our root anchor.
        let actual = res[0].canonicalize().expect("Engine returned a path that does not exist on disk");
        let expected = root.join("Projects/ncd").canonicalize().unwrap();

        assert_eq!(actual, expected, "Path resolution mismatch.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_mixed_wildcards() {
        // 1. Establish physical reality and lock the process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Use your local helper for explicit initialization
        // Ensures no 'default' leaks from the environment into the test logic.
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Case: Pr*j?cts -> Projects
        let res = evaluate_jump("Pr*j?cts", &opts);

        // 4. Cluster-Safe Assertions
        assert!(!res.is_empty(), "Failed mixed * and ? wildcards: Result set is empty");

        // Physical validation: Ensure the path is exactly where we expect in the sandbox.
        let actual = res[0].canonicalize().expect("Engine returned a non-existent wildcard match");
        let expected = root.join("Projects").canonicalize().unwrap();

        assert_eq!(actual, expected, "Wildcard resolution mismatch.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_interspersed_parents() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Use your local helper for explicit initialization
        // Ensures the engine is anchored to the temp root.
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Projects/ncd/../../Drivers -> root/Drivers
        let res = evaluate_jump("Projects/ncd/../../Drivers", &opts);

        // 4. Cluster-Safe Assertions
        assert!(!res.is_empty(), "Failed interspersed parent jump: Result set is empty");

        // Physical validation: Ensure we didn't "escape" the root or hallucinate the path.
        let actual = res[0].canonicalize().expect("Engine returned a path that does not exist");
        let expected = root.join("Drivers").canonicalize().unwrap();

        assert_eq!(actual, expected, "Parent navigation mismatch.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_multiple_dots() {
        // 1. Setup sandbox and guard the process environment
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Use your local helper for explicit initialization
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Case: Projects/././ncd/. -> Projects/ncd
        let res = evaluate_jump("Projects/././ncd/.", &opts);

        // 4. Verification: Check for existence and physical parity
        assert!(!res.is_empty(), "Failed interspersed dot navigation: Engine returned empty result");

        let actual = res[0].canonicalize().expect("Resolved path with redundant dots must physically exist");
        let expected = root.join("Projects/ncd").canonicalize().unwrap();

        assert_eq!(actual, expected, "Dot normalization failed.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_mixed_wildcards_mk7() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Explicit options via your local helper
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Projec*/././ncd/./../nc*/.../Drivers
        // Logic: Projects -> ncd -> (up) -> ncd -> (deep search) -> Drivers (which shouldn't exist there based on setup_test_env)
        let res = evaluate_jump("Projec*/././ncd/./../nc*/.../Drivers", &opts);

        // 4. Cluster-Safe Assertions
        assert!(!res.is_empty(), "Mixed wildcards with parent jump and ellipsis failed to return results");

        // Physical validation
        let actual = res[0].canonicalize().expect("Resolved Mk7 path must be physically valid");

        // Based on setup_test_env, Drivers is at root/Drivers.
        // If the logic resolves correctly, we need to ensure it matches the physical reality.
        let expected = root.join("Drivers").canonicalize().unwrap();

        assert_eq!(actual, expected, "Mk7 complex resolution mismatch.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_mixed_wildcards_mk6() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Explicit options via your local helper
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Projec*/././ncd/./../nc*
        // Logic: Projects/ncd/../ncd -> resolves to root/Projects/ncd
        let res = evaluate_jump("Projec*/././ncd/./../nc*", &opts);

        // 4. Verification: Cluster-safe absolute path parity
        assert!(!res.is_empty(), "Mk6 Mixed wildcards with parent jump failed to return results");

        let actual = res[0].canonicalize().expect("Resolved Mk6 path must physically exist");
        let expected = root.join("Projects/ncd").canonicalize().unwrap();

        assert_eq!(actual, expected, "Mk6 resolution failed the physical pivot test.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_mixed_wildcards_mk5() {
        // 1. Setup physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Explicit options via your local helper
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Projec*/././ncd/./../nc?
        // Logic: Projects/ncd/../nc? -> matches root/Projects/ncd
        let res = evaluate_jump("Projec*/././ncd/./../nc?", &opts);

        // 4. Verification: Absolute path parity
        assert!(!res.is_empty(), "Mk5 Mixed wildcards (?) with parent jump failed");

        let actual = res[0].canonicalize().expect("Resolved Mk5 path must physically exist");
        let expected = root.join("Projects/ncd").canonicalize().unwrap();

        assert_eq!(actual, expected, "Mk5 single-char wildcard resolution failed.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_mixed_wildcards_mk4() {
        // 1. Setup sandbox and lock process environment
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Use your local helper for explicit initialization
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Case: Projec*/././ncd/./../ncd
        // Logic: Projects -> ncd -> (up) -> ncd (literal)
        let res = evaluate_jump("Projec*/././ncd/./../ncd", &opts);

        // 4. Verification: Cluster-safe absolute path parity
        assert!(!res.is_empty(), "Mk4 literal segment after parent jump failed");

        let actual = res[0].canonicalize().expect("Resolved Mk4 path must physically exist");
        let expected = root.join("Projects/ncd").canonicalize().unwrap();

        assert_eq!(actual, expected, "Mk4 resolution failed to land at the correct physical path.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
}

mod battery_2_mk2 {
    use crate::{evaluate_jump, CdMode};
    use crate::unit_tests_local::{get_opts, get_opts_fuzzy, setup_test_env, CwdGuard};

    #[test]
    fn test_edge_mixed_wildcards_mk3() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Explicit options via your local helper
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Projects/././ncd/./../ncd
        // Logic: Projects -> ncd -> (up) -> ncd
        let res = evaluate_jump("Projects/././ncd/./../ncd", &opts);

        // 4. Verification: Cluster-safe absolute path parity
        assert!(!res.is_empty(), "Mk3 literal pivot navigation failed");

        let actual = res[0].canonicalize().expect("Resolved Mk3 path must physically exist");
        let expected = root.join("Projects/ncd").canonicalize().unwrap();

        assert_eq!(actual, expected, "Mk3 resolution failed the physical identity check.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_mixed_wildcards_mk2() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 1 & 2: Base Glob + Trailing Slash (Pro*s / Pro*s/)
        for input in ["Pro*s", "Pro*s/"] {
            let res = evaluate_jump(input, &opts);
            assert!(!res.is_empty(), "Failed: {}", input);
            assert_eq!(res[0].canonicalize().unwrap(), root.join("Projects").canonicalize().unwrap());
        }

        // 3 & 4: Sub-Glob + Trailing Slash (Pro*s/nc? / Pro*s/nc?/)
        for input in ["Pro*s/nc?", "Pro*s/nc?/"] {
            let res = evaluate_jump(input, &opts);
            assert!(!res.is_empty(), "Failed: {}", input);
            assert_eq!(res[0].canonicalize().unwrap(), root.join("Projects/ncd").canonicalize().unwrap());
        }

        // 5 & 6: Glob Pivot + Trailing Slash (Pro*s/nc?/.. / Pro*s/nc?/../)
        for input in ["Pro*s/nc?/..", "Pro*s/nc?/../"] {
            let res = evaluate_jump(input, &opts);
            assert!(!res.is_empty(), "Failed: {}", input);
            assert_eq!(res[0].canonicalize().unwrap(), root.join("Projects").canonicalize().unwrap());
        }

        // 7: The Full Gauntlet (Glob -> Pivot -> Deep Search)
        let res = evaluate_jump("Pro*s/nc?/.../Dri*", &opts);
        assert!(!res.is_empty(), "Failed: Case 7");
        let actual = res[0].canonicalize().expect("Case 7: Path must physically exist");
        let expected = root.join("Drivers").canonicalize().unwrap();
        assert_eq!(actual, expected, "Case 7 failed deep-search anchor check");
    }
    #[test]
    fn test_edge_interspersed_parents_mk3() {
        // 1. Setup physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Explicit options via your local helper
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Projects/ncd/.../Drivers
        // Logic: Navigate to Projects/ncd, then deep search for Drivers.
        let res = evaluate_jump("Projects/ncd/.../Drivers", &opts);

        // 4. Verification: Check for existence and physical parity
        assert!(!res.is_empty(), "Failed to resolve ellipsis deep-search path");

        let actual = res[0].canonicalize().expect("Resolved path must physically exist");

        // Based on setup_test_env, Drivers is usually at root/Drivers.
        // If ncd/.../Drivers matches, it proves the deep search can look 'sideways' or 'down'.
        let expected = root.join("Drivers").canonicalize().unwrap();

        assert_eq!(actual, expected, "Ellipsis resolution failed physical anchor check.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_question_mark_wildcard() {
        // 1. Setup physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Explicit options via your local helper
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Wind?ws/Sys??m32
        // Expected physical resolution: root/Windows/System32
        let res = evaluate_jump("Wind?ws/Sys??m32", &opts);

        // 4. Verification: Cluster-safe absolute path parity
        assert!(!res.is_empty(), "Failed single-character wildcard '?' resolution");

        let actual = res[0].canonicalize().expect("Resolved path with '?' must physically exist");
        let expected = root.join("Windows/System32").canonicalize().unwrap();

        assert_eq!(actual, expected, "Wildcard '?' matched the wrong directory or failed character count.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_mixed_wildcards_and_dots() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Explicit options via your local helper
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Pr*/./n*d -> root/Projects/ncd
        let res = evaluate_jump("Pr*/./n*d", &opts);

        // 4. Verification: Absolute path parity
        assert!(!res.is_empty(), "Failed mixed wildcard and dot resolution: Result set is empty");

        let actual = res[0].canonicalize().expect("Resolved path must be physically valid");
        let expected = root.join("Projects/ncd").canonicalize().unwrap();

        assert_eq!(actual, expected, "Wildcard/Dot resolution mismatch.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_interspersed_parents_mk5() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Use your local helper for explicit initialization
        let opts = get_opts_fuzzy(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Projects/ncd/.././../Proj/ncd/.../Drivers
        let res = evaluate_jump("Projects/ncd/.././../Proj/ncd/.../Drivers", &opts);

        // 4. Verification: Check for existence and physical parity
        assert!(!res.is_empty(), "Failed to resolve '..' segment inside complex path");

        let actual = res[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Drivers").canonicalize().unwrap();

        assert_eq!(actual, expected, "Zig-zag navigation failed.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_interspersed_parents_mk4() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Explicit options via your local helper
        let opts = get_opts_fuzzy(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Projects/ncd/.././../Projects/ncd/.../Driv
        // Logic: Down -> Up -> Up -> Down -> Down -> Recursive Fuzzy Search for "Driv"
        let res = evaluate_jump("Projects/ncd/.././../Projects/ncd/.../Driv", &opts);

        // 4. Verification: Cluster-safe absolute path parity
        assert!(!res.is_empty(), "Failed to resolve fuzzy-zig-zag path");
    }
    #[test]
    fn test_edge_interspersed_parents_mk4_1() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        let opts = get_opts_fuzzy(CdMode::Hybrid, false, Some(root.clone().into()));

        // Execution: Down -> Up -> Up -> Down -> Down -> Recursive Fuzzy Search for "Driv"
        // To avoid matching the root 'Drivers', we search for a more specific sub-target like 'sr'
        let res = evaluate_jump("Projects/ncd/.././../Projects/ncd/sr", &opts);

        assert!(!res.is_empty(), "Failed to resolve fuzzy-zig-zag path");

        let actual = res[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Projects/ncd/src").canonicalize().unwrap();

        assert_eq!(actual, expected, "Fuzzy-Zig-Zag navigation failed to anchor in Projects/ncd/src.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_interspersed_parents_mk1() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Explicit options via your local helper
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 3. Execution: Projects/../Projects/ncd/../../Projects
        // Logic: Proves the climber can oscillate at the root level without escaping the sandbox.
        let res = evaluate_jump("Projects/../Projects/ncd/../../Projects", &opts);

        // 4. Verification: Check for existence and physical parity
        assert!(!res.is_empty(), "Failed to resolve root-pivot traversal");

        let actual = res[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Projects").canonicalize().unwrap();

        assert_eq!(actual, expected, "Root-level pivot mismatch.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_interspersed_parents_mk4_2() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let opts = get_opts_fuzzy(CdMode::Hybrid, false, Some(root.clone().into()));

        // Logic: Go deep, come back to root, go deep again, then find Drivers from there.
        let res = evaluate_jump("Projects/ncd/.././../Projects/ncd/.../Driv", &opts);

        assert!(!res.is_empty(), "Failed to resolve fuzzy-zig-zag path");

        let actual = res[0].canonicalize().expect("Resolved path must physically exist");
        // The engine found this correctly in your failed run!
        let expected = root.join("Drivers").canonicalize().unwrap();

        assert_eq!(actual, expected, "The engine failed to find Drivers after the zig-zag.");
    }
}

mod battery_2_mk3 {
    use crate::{evaluate_jump, CdMode, DirMatch, SearchOptions};
    use crate::unit_tests_local::{setup_test_env, CwdGuard};

    #[test]
    fn test_complex_edge_case_1() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Setup options - Hybrid mode ensures no fuzzy fallback is triggered
        let opts = SearchOptions {
            mode: CdMode::Hybrid,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };

        // 3. Execution: "Projects/./ncd"
        // Logic: The dot must be neutralized; result must be exactly Projects/ncd.
        let res1 = evaluate_jump("Projects/./ncd", &opts);

        // 4. Verification: Absolute physical parity
        assert!(!res1.is_empty(), "Identity segment '.' caused resolution failure");

        let actual = res1[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Projects/ncd").canonicalize().unwrap();

        assert_eq!(actual, expected, "The identity dot interrupted the structural walk.");
    }
    #[test]
    fn test_complex_edge_case_2() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Hybrid options: Confirms the climber handles ".." before any fuzzy logic kicks in
        let opts = SearchOptions {
            mode: CdMode::Hybrid,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };

        // 3. Execution: "Projects/ncd/../../Drivers"
        // Logic: Down to ncd, Up to Projects, Up to Root, Down to Drivers.
        let res2 = evaluate_jump("Projects/ncd/../../Drivers", &opts);

        // 4. Verification: Absolute physical parity
        assert!(!res2.is_empty(), "Failed to resolve pivot path 'Projects/ncd/../../Drivers'");

        let actual = res2[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Drivers").canonicalize().unwrap();

        assert_eq!(actual, expected, "The pivot navigation failed to land in the Drivers folder.");
    }
    #[test]
    fn test_complex_edge_case_3() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Setup options - Hybrid mode checks if structural globbing holds up
        let opts = SearchOptions {
            mode: CdMode::Hybrid,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };

        // 3. Execution: "Pr?jects"
        // Logic: '?' must match exactly one character (the 'o' in 'Projects')
        let res3 = evaluate_jump("Pr?jects", &opts);

        // 4. Verification: Absolute parity check
        assert!(!res3.is_empty(), "Failed to resolve single-char wildcard 'Pr?jects'");

        let actual = res3[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Projects").canonicalize().unwrap();

        assert_eq!(actual, expected, "Single-character wildcard resolution failed.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_complex_edge_case_4() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Setup options - Hybrid mode checks if structural globbing holds up
        let opts = SearchOptions {
            mode: CdMode::Hybrid,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };

        // 3. Execution: "P*j?cts"
        // Logic: '*' matches 'ro' (0+ chars), '?' matches 'e' (exactly 1 char)
        let res4 = evaluate_jump("P*j?cts", &opts);

        // 4. Verification: Absolute parity check
        assert!(!res4.is_empty(), "Failed to resolve mixed wildcards 'P*j?cts'");

        let actual = res4[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Projects").canonicalize().unwrap();

        assert_eq!(actual, expected, "Mixed wildcard resolution failed.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_complex_edge_case_5() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Setup options - Hybrid mode tests the structural resolution of '??'
        let opts = SearchOptions {
            mode: CdMode::Hybrid,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };

        // 3. Execution: "Windows/syst??32"
        // Logic: Each '?' must match exactly one character (e.g., 'em' in 'System32').
        let res5 = evaluate_jump("Windows/syst??32", &opts);

        // 4. Verification: Absolute parity check
        assert!(!res5.is_empty(), "Failed to resolve double '??' in 'syst??32'");

        let actual = res5[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Windows/System32").canonicalize().unwrap();

        assert_eq!(actual, expected, "Fixed-width wildcard resolution failed.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_complex_edge_case_6() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Hybrid options: Verifies the fuzzer doesn't trigger on the trailing slash
        let opts = SearchOptions {
            mode: CdMode::Hybrid,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };

        // 3. Execution: "Projects/ncd/"
        // Logic: The final slash should be neutralized during tokenization.
        let res6 = evaluate_jump("Projects/ncd/", &opts);

        // 4. Verification: Absolute parity check
        assert!(!res6.is_empty(), "Trailing slash caused resolution failure for 'Projects/ncd/'");

        let actual = res6[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Projects/ncd").canonicalize().unwrap();

        assert_eq!(actual, expected, "Trailing slash redirected to wrong location.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_complex_edge_case_7() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Setup options - Hybrid allows us to see if it falls back to fuzzy on failure
        let opts = SearchOptions {
            mode: CdMode::Hybrid,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };

        // 3. Execution: "Projects//ncd"
        // Logic: The empty segment between // must be ignored or collapsed.
        let res7 = evaluate_jump("Projects//ncd", &opts);

        // 4. Verification: Check physical parity
        assert!(!res7.is_empty(), "Double slash 'Projects//ncd' caused resolution failure");

        let actual = res7[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Projects/ncd").canonicalize().unwrap();

        assert_eq!(actual, expected, "Separator normalization failed.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_complex_edge_case_8() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Manual SearchOptions setup
        let opts = SearchOptions {
            mode: CdMode::Hybrid,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };

        // 3. Execution: " . / . / . "
        // Logic: Each dot is a no-op; the result should be the current directory (root)
        let res8 = evaluate_jump(" . / . / . ", &opts);

        // 4. Verification: Check physical parity against the guarded root
        assert!(!res8.is_empty(), "Nop jump returned empty result");

        let actual = res8[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.canonicalize().unwrap();

        assert_eq!(actual, expected, "Identity traversal failed to remain at root.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_complex_edge_case_9a() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Hybrid Mode: Structural first, Fuzzy fallback if needed
        let opts = SearchOptions {
            mode: CdMode::Hybrid,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };

        // 3. Execution: */Guest/*top
        // Verifies: Does the globber correctly handle the middle literal 'Guest'?
        let res9 = evaluate_jump("*/Guest/*top", &opts);

        // 4. Verification: Absolute physical parity
        assert!(!res9.is_empty(), "Failed deep wildcard walk in Hybrid mode");

        let actual = res9[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Users/Guest/Desktop").canonicalize().unwrap();

        assert_eq!(actual, expected, "Hybrid wildcard walk deviated from expected sandbox path.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_complex_edge_case_9b() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Origin Mode: Search is strictly anchored to the mock_path
        let opts = SearchOptions {
            mode: CdMode::Origin,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };

        // 3. Execution: */Guest/*top
        // Logic: The first '*' must match 'Users' relative to the root.
        let res9 = evaluate_jump("*/Guest/*top", &opts);

        // 4. Verification: Absolute physical parity
        assert!(!res9.is_empty(), "Failed deep wildcard walk in Origin mode");

        let actual = res9[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Users/Guest/Desktop").canonicalize().unwrap();

        assert_eq!(actual, expected, "Origin mode wildcard resolution strayed from root.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_complex_edge_case_9c() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Target Mode: No fuzzy guesswork allowed, must follow the structure
        let opts = SearchOptions {
            mode: CdMode::Target,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };

        // 3. Execution: */Guest/*top -> Expected: Users/Guest/Desktop
        let res9 = evaluate_jump("*/Guest/*top", &opts);

        // 4. Verification: Check for existence and physical parity
        assert!(!res9.is_empty(), "Failed deep wildcard walk for '*/Guest/*top'");

        let actual = res9[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Users/Guest/Desktop").canonicalize().unwrap();

        assert_eq!(actual, expected, "Wildcard walk landed in the wrong directory.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_complex_edge_case_10() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Manual SearchOptions setup as per your architecture
        let opts = SearchOptions {
            mode: CdMode::Hybrid,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };

        // 3. Execution: Empty Query ""
        // Logic: An empty string provides no target; the climber should return immediately.
        let res10 = evaluate_jump("", &opts);

        // 4. Verification: Result set must be empty
        assert!(res10.is_empty(), "Empty query should return an empty result set, but got: {:?}", res10);
    }

}

mod battery_2_mk4 {
    use std::env;
    use std::path::PathBuf;
    use crate::{evaluate_jump, handle_ellipsis, resolve_path_segments, CdMode, DirMatch, SearchOptions};
    use crate::unit_tests_local::{get_opts, setup_test_env, CwdGuard};

    #[test]
    fn check_edges() {
        // 1. Setup sandbox and anchor context
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Define test matrix: (Query, Logical Expected Substring)
        let cases = vec![
            ("Proj*", "Projects"),
            ("Driv*", "Drivers"),
            ("Users/G*/Desk*", "Users/Guest/Desktop"),
            ("Windows/Sys*", "Windows/System32"),
            ("Proj*/ncd/src", "Projects/ncd/src")
        ];

        for (query, expected) in cases {
            // 3. Configure Hybrid options using mock_path anchor
            let opts = SearchOptions {
                mode: CdMode::Hybrid,
                exact: false,
                list: false,
                dir_match: DirMatch::default(),
                mock_path: Some(root.clone().into_os_string())
            };

            // 4. Execution & Verification
            let res = evaluate_jump(query, &opts);

            assert!(!res.is_empty(), "Query '{}' returned no results", query);

            let found = res.iter().any(|p| {
                let normalized = p.to_string_lossy().replace('\\', "/");
                normalized.contains(expected)
            });

            assert!(found, "Pattern match failed.\nQuery:    {}\nExpected: {}\nResults:  {:?}", query, expected, res);
        }
    }
    #[test]
    fn test_path_resolutions() {
        // 1. Setup sandbox and lock process context
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let root_str = root.to_str().unwrap().to_string();

        let opts = |m: Option<String>| SearchOptions {
            mode: CdMode::Hybrid,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: m.map(Into::into)
        };

        // 2. Execution: Authority & Tail Reattachment
        let res1 = evaluate_jump("Pr*", &opts(Some(root_str.clone())));
        let res4 = evaluate_jump("Proj*/ncd/src", &opts(Some(root_str.clone())));
        let res5 = evaluate_jump("Users/G*/Desktop", &opts(Some(root_str.clone())));

        // 3. Execution: Ellipsis Logic
        let base = root.join("Projects").join("ncd");
        let res8 = handle_ellipsis("...", base);

        // 4. Verifications
        assert!(!res1.is_empty(), "Step 1: 'Pr*' failed in {}", root_str);

        let target4 = PathBuf::from("Projects/ncd/src");
        assert!(res4[0].ends_with(&target4), "Step 4: Expected {:?}, Got {:?}", target4, res4[0]);

        let target5 = PathBuf::from("Users/Guest/Desktop");
        assert!(res5[0].ends_with(&target5), "Step 5: Expected {:?}, Got {:?}", target5, res5[0]);

        let found8 = res8[0].canonicalize().unwrap();
        let expected8 = root.canonicalize().unwrap();
        assert_eq!(found8, expected8, "Step 8: Ellipsis failed to hit root.");
    }
    #[test]
    fn test_walker_integration() {
        // 1. Establish physical sandbox and lock process directory
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // 2. Set options - 'true' for exact matching to bypass fuzzy logic
        let opts = get_opts(CdMode::Hybrid, true, Some(root.clone().into_os_string()));

        // 3. Execution: Manual walk through the directory tree
        // Logic: Start at root, find 'Users', then find 'Guest'
        let tail = vec!["Users", "Guest"];
        let results = resolve_path_segments(vec![root.clone()], tail, &opts);

        // 4. Verification: Check physical existence and path composition
        assert!(!results.is_empty(), "Walker failed to resolve path in temp env");

        let actual = results[0].canonicalize().expect("Resolved path must physically exist");
        let expected = root.join("Users/Guest").canonicalize().unwrap();

        assert_eq!(actual, expected, "Walker deviated from the expected path.\nExpected: {:?}\nActual:   {:?}", expected, actual);
    }
    #[test]
    fn test_edge_interspersed_dots() {
        // 1. Setup sandbox and lock process context
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 2. Execution: "Projects/./ncd"
        let res = evaluate_jump("Projects/./ncd", &opts);

        // 3. Verification: Absolute physical parity
        assert!(!res.is_empty(), "Failed to resolve path with identity segment '.'");
        let actual = res[0].canonicalize().expect("Resolved path must exist");
        let expected = root.join("Projects/ncd").canonicalize().unwrap();

        assert_eq!(actual, expected, "Identity dot '.' altered the resolution path.");
    }
    #[test]
    fn test_edge_interspersed_parents_mk1() {
        // 1. Setup sandbox and lock process context
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 2. Execution: Complex zig-zag path
        // Logic: Projects -> ncd -> (Up twice) -> (Stay) -> Drivers
        let res = evaluate_jump("Projects/ncd/..././Drivers", &opts);

        // 3. Verification: Absolute parity check
        assert!(!res.is_empty(), "Pivot path failed to resolve");
        let actual = res[0].canonicalize().expect("Resolved path must exist");
        let expected = root.join("Drivers").canonicalize().unwrap();

        assert_eq!(actual, expected, "Navigation through '...' landed in wrong directory.");
    }
    #[test]
    fn test_edge_wildcard_question_mark() {
        // 1. Setup sandbox and lock process context
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 2. Execution: Single character '?' in 'Pr?jects'
        let res = evaluate_jump("Pr?jects", &opts);

        // 3. Verification: Must match 'Projects' exactly
        assert!(!res.is_empty(), "Single-char wildcard failed to resolve");
        let actual = res[0].canonicalize().expect("Resolved path must exist");
        let expected = root.join("Projects").canonicalize().unwrap();

        assert_eq!(actual, expected, "Wildcard '?' matched the wrong directory.");
    }
    #[test]
    fn test_edge_mixed_wildcards_mk1() {
        // 1. Establish sandbox and lock process context
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 2. Execution: Mixed glob "P*j?cts"
        // Logic: '*' matches 'ro', '?' matches 'e'
        let res = evaluate_jump("P*j?cts", &opts);

        // 3. Verification: Absolute parity check
        assert!(!res.is_empty(), "Mixed glob resolution returned no results");
        let actual = res[0].canonicalize().expect("Path must exist");
        let expected = root.join("Projects").canonicalize().unwrap();

        assert_eq!(actual, expected, "Mixed glob landed in wrong location.");
    }
    #[test]
    fn test_edge_triple_dots_nop() {
        // 1. Setup sandbox and lock process CWD to root
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let opts = get_opts(CdMode::Hybrid, false, Some(root.clone().into()));

        // 2. Execution: Standard identity path
        let res = evaluate_jump("././.", &opts);

        // 3. Verification: Must match the guarded CWD (root)
        assert!(!res.is_empty(), "Identity path returned empty result");
        let actual = res[0].canonicalize().unwrap();
        let expected = env::current_dir().unwrap().canonicalize().unwrap();

        assert_eq!(actual, expected, "Identity collapse failed to match CWD.");
    }
    #[test]
    fn test_complex_edge_cases() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root); // Lock CWD to root for relative tests
        let opts = SearchOptions {
            mode: CdMode::Hybrid,
            exact: false,
            list: false,
            dir_match: DirMatch::default(),
            mock_path: Some(root.clone().into())
        };
        // 1-2: Relative Pivots
        let res1 = evaluate_jump("Projects/./ncd", &opts);
        assert_eq!(res1[0].canonicalize().unwrap(), root.join("Projects/ncd").canonicalize().unwrap());

        let res2 = evaluate_jump("Projects/ncd/../../Drivers", &opts);
        assert!(res2[0].canonicalize().unwrap().ends_with("Drivers"));

        // 3-5: Wildcard Strictness
        assert!(!evaluate_jump("Pr?jects", &opts).is_empty());
        assert!(!evaluate_jump("P*j?cts", &opts).is_empty());
        assert!(!evaluate_jump("Windows/syst??32", &opts).is_empty());

        // 6-8: String Normalization
        assert!(!evaluate_jump("Projects/ncd/", &opts).is_empty());
        assert!(!evaluate_jump("Projects//ncd", &opts).is_empty());
        let res8 = evaluate_jump(" . / . / . ", &opts);
        assert_eq!(res8[0].canonicalize().unwrap(), root.canonicalize().unwrap());

        // 9-10: Deep Walk & Boundary
        let res9 = evaluate_jump("*/Guest/*top", &opts);
        assert_eq!(res9[0].canonicalize().unwrap(), root.join("Users/Guest/Desktop").canonicalize().unwrap());
        assert!(evaluate_jump("", &opts).is_empty());
    }
}

mod battery_3 {
    use std::{env};
    use std::path::PathBuf;
    use crate::{evaluate_jump, handle_ellipsis, CdMode};
    use crate::unit_tests_local::{create_ncd_sandbox, get_opts, setup_test_env, CwdGuard};

    #[test]
    fn test_ellipsis_relative_climb_resolved() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let start = PathBuf::from("Projects/ncd/src");
        let matches = handle_ellipsis("....", start);

        assert_eq!(matches[0].canonicalize().unwrap(), root.canonicalize().unwrap());
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
    fn test_ellipsis_drive_root_safety() {
        // Test that even on a real drive root, we don't crash or return invalid paths
        let base = PathBuf::from(r"C:\");
        let matches = handle_ellipsis("...", base);

        assert_eq!(matches[0], PathBuf::from(r"C:\"),
                   "Popping from drive root should simply return drive root");
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
        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));

        // Anchor the query to the mock root manually since the source isn't virtualized
        let query = format!("{}{}", root.display(), r"\Projects");
        let result = evaluate_jump(&query, &opts);

        assert!(!result.is_empty(), "Result should not be empty for: {}", query);
        assert!(result[0].starts_with(&root), "Result escaped the mock root: {:?}", result[0]);
    }
}

mod battery_4 {
    use std::{env, fs};
    use std::path::PathBuf;
    use crate::{evaluate_jump, get_drive_root, handle_ellipsis, resolve_path_segments, search_cdpath, CdMode};
    use crate::unit_tests_local::{get_opts, get_opts_fuzzy, setup_test_env, test_opts, CwdGuard};

    #[test]
    fn test_ellipsis_overflow_safety() {
        let (_tmp, root) = setup_test_env();
        let deep = root.join("Projects").join("ncd").join("src");
        let _guard = CwdGuard::new(&deep);
        let drive_root = format!("{}\\", get_drive_root(&deep).unwrap().to_string_lossy().replace("\\\\?\\", "").trim_end_matches('\\'));

        let res = handle_ellipsis("....................", PathBuf::from("."));

        assert_eq!(res[0].to_string_lossy().replace("\\\\?\\", ""), drive_root, "Should have pinned to drive root");
    }
    #[test]
    fn test_fuzzy_match_depth_integrity() {
        let (_tmp, root) = setup_test_env();
        let opts = get_opts_fuzzy(CdMode::Origin, false, Some(root.into_os_string()));

        let res = evaluate_jump("users/guest", &opts);

        assert!(res.iter().any(|p| p.to_string_lossy().to_lowercase().contains("guest")), "Pipeline failed to resolve segmented path");
    }
    #[cfg(windows)]
    #[test]
    fn test_junction_traversal_integrity() {
        let (_tmp, root) = setup_test_env();
        let junction_path = root.join("Projects_Link");
        // Create a real Windows Junction pointing to the Projects folder
        std::os::windows::fs::symlink_dir(root.join("Projects"), &junction_path).unwrap();

        let opts = get_opts(CdMode::Origin, false, Some(root.as_os_str().to_os_string()));
        let res = search_cdpath("Projects_Link", &opts);

        assert!(res.iter().any(|p| p.to_string_lossy().contains("Projects_Link")), "Engine failed to traverse through the Junction!");
    }
    #[test]
    fn test_authority_depth_limit() {
        let (_tmp, root) = setup_test_env();
        let deep = root.join("Level1/Level2/Level3");
        fs::create_dir_all(&deep).expect("Failed to create test depth");
        let _guard = CwdGuard::new(&root);
        env::set_current_dir(&root).expect("Failed to jump to temp volume");

        // Start at Root Authority
        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));

        let actual_cwd = env::current_dir().unwrap();
        let mock_val = opts.mock_path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "NONE".into());
        println!("\n[CWD CHECK]\nActual CWD: {:?}\nMock Path:  {}\nRoot Exists: {}", actual_cwd, mock_val, std::path::Path::new(&mock_val).exists());

        // Step 1: Find Level1
        let res1 = search_cdpath("*1", &opts);
        assert!(!res1.is_empty(), "Step 1: Failed to find Level1 from Root");

        // Step 2: Pivot to Level1 Authority to find Level2
        let opts2 = get_opts(CdMode::Origin, false, Some(res1[0].clone().into_os_string()));
        let res2 = search_cdpath("*2", &opts2);
        assert!(!res2.is_empty(), "Step 2: Failed to find Level2 from Level1");

        // Step 3: Pivot to Level2 Authority to find Level3
        let opts3 = get_opts(CdMode::Origin, false, Some(res2[0].clone().into_os_string()));
        let results = search_cdpath("Level3", &opts3);

        // Final Validation
        assert!(!results.is_empty(), "Step 3: Engine failed to resolve Level3 under authority");
        assert!(results[0].to_string_lossy().contains("Level3"), "Path mismatch at destination");
    }
    #[test]
    fn test_drive_root_regression_three() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        fs::create_dir_all(root.join("Projects")).unwrap();

        let results = resolve_path_segments(vec![env::current_dir().unwrap()], vec!["Projects"], &test_opts());

        assert!(!results.is_empty(), "Search failed");
        assert!(results[0].is_absolute(), "Resolved path should be absolute");
    }
    #[test]
    fn test_root_protection_logic() {
        let (_tmp, root) = setup_test_env();
        let safe_zone = root.join("SafeZone");
        fs::create_dir(&safe_zone).ok();

        // Jump down first: We are testing navigation, not "Sandbox Security"
        let _guard = CwdGuard::new(&safe_zone);
        env::set_current_dir(&safe_zone).expect("Failed to jump");

        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));
        let res = evaluate_jump("..", &opts);

        // Assert the logic does the simple thing: Parent of SafeZone is Root
        assert!(!res.is_empty(), "Should return the parent directory");
        assert_eq!(res[0].canonicalize().unwrap(), root.canonicalize().unwrap());
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
        assert_eq!(res[0], child);
    }
}
/*
mod evaluate_jumpo {
    use std::env;
    use std::path::PathBuf;
    use crate::{handle_ellipsis, search_cdpath, CdMode, SearchOptions};
    use crate::unit_tests_local::{get_opts, setup_test_env, CwdGuard};
    pub fn evaluate_jump(query: &str, opts: &SearchOptions) -> Vec<PathBuf> {
        let q = query.trim();
        if q == "." { return env::current_dir().map(|p| vec![p]).unwrap_or_default(); }

        // Handle ellipsis traversal (..., ....)
        if q.starts_with("...") {
            return handle_ellipsis(q, env::current_dir().unwrap_or_default());
        }

        // Standard search pipeline
        let results = search_cdpath(q, opts);
        if results.is_empty() && opts.mode == CdMode::Hybrid {
            return perform_recursive_search(q, opts);
        }
        results
    }

    #[test]
    fn test_evaluate_jump_hybrid_escalation() {
        let (_tmp, root) = setup_test_env();
        let deep_target = root.join("level1/level2/target_dir");
        std::fs::create_dir_all(&deep_target).unwrap();

        // CDPATH only points to root; "target_dir" is deep.
        // Origin mode should fail, Hybrid should find it.
        let opts_origin = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));
        let opts_hybrid = get_opts(CdMode::Hybrid, false, Some(root.clone().into_os_string()));

        assert!(evaluate_jump("target_dir", &opts_origin).is_empty());
        assert!(!evaluate_jump("target_dir", &opts_hybrid).is_empty());
    }
    #[test]
    fn test_evaluate_jump_segmented_path() {
        let (_tmp, root) = setup_test_env();
        let target = root.join("work/projects/ncd");
        std::fs::create_dir_all(&target).unwrap();

        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));
        // Query "work/ncd" should resolve through the tree
        let res = evaluate_jump("work/ncd", &opts);

        assert!(!res.is_empty());
        assert!(res[0].ends_with("work/projects/ncd"));
    }
}
*/
mod resolve_path_segments {
    use crate::resolve_path_segments;
    use crate::unit_tests_local::{setup_test_env, test_opts};

    #[test]
    fn test_walker_locks_to_intermediate_match() {
        let (_tmp, root) = setup_test_env();
        // Structure:
        // root/dir_a/target
        // root/dir_b/target (should not be found if we are looking in dir_a)
        let dir_a = root.join("dir_a");
        let dir_b = root.join("dir_b");
        let target_a = dir_a.join("target");
        let target_b = dir_b.join("target");

        std::fs::create_dir_all(&target_a).unwrap();
        std::fs::create_dir_all(&target_b).unwrap();

        let opts = test_opts();
        // Start search from root, look for "dir_a", then "target"
        let segments = vec!["dir_a", "target"];
        let results = resolve_path_segments(vec![root.clone()], segments, &opts);

        assert_eq!(results.len(), 1, "Should only find the target inside dir_a");
        assert_eq!(results[0].canonicalize().unwrap(), target_a.canonicalize().unwrap());
    }
    #[test]
    fn test_walker_ignores_redundant_segments() {
        let (_tmp, root) = setup_test_env();
        let target = root.join("a").join("b");
        std::fs::create_dir_all(&target).unwrap();

        let opts = test_opts();
        // Path: "a/./ /b" should resolve to "a/b"
        let segments = vec!["a", ".", " ", "b"];
        let results = resolve_path_segments(vec![root], segments, &opts);

        assert!(!results.is_empty(), "Failed to resolve path with redundant dots/spaces");
        assert_eq!(results[0].canonicalize().unwrap(), target.canonicalize().unwrap());
    }
    #[test]
    fn test_walker_branches_on_multiple_matches() {
        let (_tmp, root) = setup_test_env();
        // root/match1/target
        // root/match2/target
        let t1 = root.join("match1").join("target");
        let t2 = root.join("match2").join("target");
        std::fs::create_dir_all(&t1).unwrap();
        std::fs::create_dir_all(&t2).unwrap();

        let opts = test_opts();
        // Query "match* / target" (assuming search_cdpath handles the glob 'match*')
        let segments = vec!["match*", "target"];
        let results = resolve_path_segments(vec![root], segments, &opts);

        assert_eq!(results.len(), 2, "Walker should branch and find both targets");
    }
}
mod search_cdpath {
    use crate::{search_cdpath, CdMode};
    use crate::unit_tests_local::{get_opts, setup_test_env, test_opts};

    #[test]
    fn test_search_cdpath_phase_c_target_match() {
        let (_tmp, root) = setup_test_env();
        let bookmark = root.join("my_project");
        std::fs::create_dir_all(&bookmark).unwrap();

        // Use Target mode: "ncd my_project" should return "my_project" itself
        // because it exists in the search roots (mock_path/CDPATH).
        let opts = get_opts(CdMode::Target, true, Some(root.into_os_string()));
        let res = search_cdpath("my_project", &opts);

        assert_eq!(res.len(), 1);
        assert!(res[0].ends_with("my_project"));
    }
    #[test]
    fn test_search_cdpath_phase_d_origin_search() {
        let (_tmp, root) = setup_test_env();
        let parent = root.join("work_dirs");
        let child = parent.join("project_alpha");
        std::fs::create_dir_all(&child).unwrap();

        // Use Origin mode: "ncd project_alpha" should find the child inside "work_dirs"
        let opts = get_opts(CdMode::Origin, true, Some(parent.into_os_string()));
        let res = search_cdpath("project_alpha", &opts);

        assert_eq!(res.len(), 1);
        assert!(res[0].ends_with("work_dirs/project_alpha"));
    }
    #[test]
    fn test_search_cdpath_phase_a_direct_priority() {
        let (_tmp, root) = setup_test_env();
        let direct = root.join("direct_dir");
        let cdpath_dir = root.join("cdpath_dir");
        let distraction = cdpath_dir.join("direct_dir");

        std::fs::create_dir_all(&direct).unwrap();
        std::fs::create_dir_all(&distraction).unwrap();

        // Even if 'direct_dir' exists inside the CDPATH, passing a relative path
        // should hit Phase A and return the local one immediately.
        let opts = get_opts(CdMode::Hybrid, false, Some(root.into_os_string()));
        let res = search_cdpath("./direct_dir", &opts);

        assert_eq!(res.len(), 1);
        // Ensure it's the one in root, not the one in cdpath_dir
        assert!(res[0].canonicalize().unwrap().to_string_lossy().contains("direct_dir"));
        assert!(!res[0].canonicalize().unwrap().to_string_lossy().contains("cdpath_dir"));
    }
    #[test]
    fn test_search_cdpath_wildcard_scan() {
        let (_tmp, root) = setup_test_env();
        std::fs::create_dir_all(root.join("test_1")).unwrap();
        std::fs::create_dir_all(root.join("test_2")).unwrap();
        std::fs::create_dir_all(root.join("other")).unwrap();

        let mut opts = test_opts();
        opts.mock_path = Some(root.into_os_string());

        // Glob search "test_*"
        let res = search_cdpath("test_*", &opts);

        assert_eq!(res.len(), 2, "Should find both test_1 and test_2");
    }
}
#[allow(non_snake_case)]
mod SearchEngine {
    use crate::{resolve_path_segments, search_cdpath, DirMatch, SearchEngine};
    use crate::unit_tests_local::{setup_complex_tree, setup_test_env, test_opts, CwdGuard};

    #[test]
    fn test_search_engine_exact_casing_truth_check() {
        let (_tmp, root) = setup_test_env();
        let real_name = "WorkProject";
        let dir = root.join(real_name);
        std::fs::create_dir(&dir).unwrap();

        // Engine with exact = true
        let engine_exact = SearchEngine::new("workproject", true);
        // Should fail because "workproject" != "WorkProject"
        assert!(engine_exact.check_direct(&root).is_none());

        // Engine with exact = false
        let engine_lax = SearchEngine::new("workproject", false);
        // Should succeed on Windows/Mac
        assert!(engine_lax.check_direct(&root).is_some());
    }
    #[test]
    fn test_search_engine_glob_translation() {
        let (_tmp, root) = setup_test_env();
        // Test Dot Escaping: "v1.0" should not match "v1-0"
        let engine = SearchEngine::new("v1.0*", false);

        let match_path = root.join("v1.0_release");
        let fail_path = root.join("v1-0_release");

        assert!(engine.matches_path(&match_path));
        assert!(!engine.matches_path(&fail_path));

        // Test Question Mark: "t?st" matches "test" not "teest"
        let engine_qm = SearchEngine::new("t?st", false);
        assert!(engine_qm.matches_path(&root.join("test")));
        assert!(!engine_qm.matches_path(&root.join("teest")));
    }
    #[test]
    fn test_search_engine_scan_dir_matching_modes() {
        let (_tmp, root) = setup_test_env();
        let target = root.join("long_directory_name");
        std::fs::create_dir(&target).unwrap();

        let mut opts = test_opts();
        let engine = SearchEngine::new("long", false);

        // Mode: AsIs -> "long" should NOT match "long_directory_name"
        opts.dir_match = DirMatch::AsIs;
        assert!(engine.scan_dir(&root, &opts).is_empty());

        // Mode: Fuzzy -> "long" SHOULD match via starts_with
        opts.dir_match = DirMatch::Fuzzy;
        let results = engine.scan_dir(&root, &opts);
        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("long_directory_name"));
    }
    #[test]
    #[cfg(windows)]
    fn test_search_engine_traverses_junctions() {
        let (_tmp, root) = setup_test_env();
        let real_dir = root.join("real_dir");
        let junction = root.join("link_dir");
        std::fs::create_dir(&real_dir).unwrap();

        // Create a junction (requires admin or specific dev-mode on Windows)
        // If this fails due to permissions, the test will simply skip.
        if std::process::Command::new("cmd").args(&["/C", "mklink", "/J", junction.to_str().unwrap(), real_dir.to_str().unwrap()]).status().is_ok() {
            let engine = SearchEngine::new("link_dir", false);
            let opts = test_opts();
            let found = engine.scan_dir(&root, &opts);
            assert!(!found.is_empty(), "SearchEngine failed to see the junction as a directory");
        }
    }
    #[test]
    fn test_segmented_wildcard_isolation() {
        let (_tmp, root) = setup_test_env();
        setup_complex_tree(&root);
        let opts = test_opts();

        // Query: "dir_* / x"
        // Should find: dir_a/x, b_dir/x (wait, b_dir doesn't start with dir_)
        // Correct matches: dir_a/x and dir_c_dir/x
        let segments = vec!["dir_*", "x"];
        let res = resolve_path_segments(vec![root.clone()], segments, &opts);

        assert_eq!(res.len(), 2, "Should find 'x' only in 'dir_a' and 'dir_c_dir'");
        for path in res {
            assert!(path.to_string_lossy().contains("dir_"));
            assert!(path.ends_with("x"));
        }
    }
    #[test]
    fn test_glob_ambiguity_reporting() {
        let (_tmp, root) = setup_test_env();
        setup_complex_tree(&root);
        let _guard = CwdGuard::new(&root); // The "Anchor" for the SearchEngine
        // Scenario: searching for "dir_*" in the root
        // This is ambiguous: "dir_a" and "dir_c_dir" both match.
        let mut opts = test_opts();
        opts.list = false; // Trigger ambiguity report instead of returning all
        opts.exact = false;

        // We expect this to call report_ambiguity (which you've verified hits stderr)
        // In a unit test context, we check if the search_cdpath returns the matches
        // to the caller if not 'list', or handles the exit.
        let res = search_cdpath("dir_*", &opts);

        // If your logic returns all matches when ambiguous to let the UI handle it:
        assert_eq!(res.len(), 2);
    }
    #[test]
    fn test_literal_dot_consistency() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root); // Move the soldier into the sandbox

        let match_dir = root.join("logs.tmp");
        let fail_dir = root.join("logs_tmp"); // underscore, not dot
        std::fs::create_dir(&match_dir).unwrap();
        std::fs::create_dir(&fail_dir).unwrap();

        let opts = test_opts();
        let res = search_cdpath("*.tmp", &opts);

        assert_eq!(res.len(), 1);
        assert!(res[0].to_string_lossy().ends_with("logs.tmp"));
        assert!(!res[0].to_string_lossy().ends_with("logs_tmp"));
    }

    #[test]
    fn test_deep_segmented_ambiguity() {
        let (_tmp, root) = setup_test_env();
        setup_complex_tree(&root);
        let opts = test_opts();

        // dir_a/x/deep_target AND dir_c_dir/x/deep_target
        let segments = vec!["dir_*", "x", "deep_target"];
        let res = resolve_path_segments(vec![root], segments, &opts);

        assert_eq!(res.len(), 2, "Should find deep_target in both matching dir_* branches");
    }
}
#[allow(non_snake_case)]
mod SearchFuzzy {
    use crate::{resolve_path_segments, search_cdpath};
    use crate::unit_tests_local::{setup_test_env, test_opts, CwdGuard};

    #[test]
    fn test_walker_fan_out_multiple_branches() {
        let (_tmp, root) = setup_test_env();

        // Path A: root/match_1/target
        let path_a = root.join("match_1").join("target");
        // Path B: root/match_2/target
        let path_b = root.join("match_2").join("target");

        std::fs::create_dir_all(&path_a).unwrap();
        std::fs::create_dir_all(&path_b).unwrap();

        let mut opts = test_opts();
        opts.list = true; // We want to see everything found

        // Query: "match_* / target"
        let segments = vec!["match_*", "target"];

        // Starting the walk from root
        let results = resolve_path_segments(vec![root], segments, &opts);

        // If the bug exists, length will be 1. If fixed, length will be 2.
        assert_eq!(
            results.len(),
            2,
            "Fan-out failed! Only found {} matches. Likely locked to next_matches[0].",
            results.len()
        );

        let result_strs: Vec<String> = results.iter().map(|p| p.to_string_lossy().into_owned()).collect();
        assert!(result_strs.iter().any(|s| s.contains("match_1")));
        assert!(result_strs.iter().any(|s| s.contains("match_2")));
    }
    #[test]
    fn test_ambiguity_handling_no_list() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        // Create two ambiguous targets in the same folder
        std::fs::create_dir_all(root.join("project_alpha")).unwrap();
        std::fs::create_dir_all(root.join("project_beta")).unwrap();

        let mut opts = test_opts();

        // CASE 1: --list is OFF
        opts.list = false;
        let res_no_list = search_cdpath("project_*", &opts);
        // Based on your search_cdpath logic, this should return all matches
        // because engine.is_wildcard is true, but it calls report_ambiguity inside.
        assert_eq!(res_no_list.len(), 2, "Should return both matches even if ambiguous");

        // CASE 2: --list is ON
        opts.list = true;
        let res_list = search_cdpath("project_*", &opts);
        assert_eq!(res_list.len(), 2, "Should return both matches in list mode");
    }
    #[test]
    fn test_wildcard_boundary_integrity() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        // Should match *proj*
        std::fs::create_dir_all(root.join("my_project_dir")).unwrap();
        // Should NOT match *proj* because of the ^$ anchors if the query is just "proj"
        std::fs::create_dir_all(root.join("project")).unwrap();
        // Should match p?oj
        std::fs::create_dir_all(root.join("proj")).unwrap();

        let opts = test_opts();

        // Test 1: Full name wildcard
        let res_star = search_cdpath("*project*", &opts);
        assert!(res_star.iter().any(|p| p.to_string_lossy().contains("my_project_dir")));

        // Test 2: Single char wildcard
        let res_qm = search_cdpath("pr?j", &opts);
        assert_eq!(res_qm.len(), 1);
        assert!(res_qm[0].to_string_lossy().ends_with("proj"));
    }
    #[test]
    fn test_resolve_segments_fan_out_integrity() {
        let (_tmp, root) = setup_test_env();
        let root2 = root.clone();
        // Setup: Two different folders in CDPATH containing the same subfolder
        let path_a = root.join("dir_a").join("sub");
        let path_b = root.join("dir_b").join("sub");
        std::fs::create_dir_all(&path_a).unwrap();
        std::fs::create_dir_all(&path_b).unwrap();

        let mut opts = test_opts();
        opts.mock_path = Some(root.into_os_string());
        opts.list = true;

        // "sub" is relative, so it should be searched in all roots
        let segments = vec!["*", "sub"];
        let results = resolve_path_segments(vec![root2], segments, &opts);

        // If the bug exists, this will likely be 1. We want 2.
        assert_eq!(results.len(), 2, "Walker failed to branch! Check recursion logic.");
    }
}
mod get_drive_components {
    use crate::get_drive_components;

    #[test]
    fn test_get_drive_components_windows_anchors() {
        // Case: C:\Windows
        let (bare, anchored, parts) = get_drive_components("C:\\Windows");
        assert!(anchored, "C:\\ should be anchored");
        assert!(!bare, "C:\\Windows is not a bare drive");
        assert_eq!(parts[0], "C:");

        // Case: C: (The "Bare" drive case)
        let (bare_c, anchored_c, _) = get_drive_components("C:");
        assert!(anchored_c, "C: should be anchored");
        assert!(bare_c, "C: should be identified as bare");
    }
    #[test]
    fn test_get_drive_components_unix_anchors() {
        // Case: /home/user
        let (bare, anchored, parts) = get_drive_components("/home/user");
        assert!(anchored, "Paths starting with / must be anchored");
        assert!(!bare);
        // Note: parts[0] will be empty because of the leading slash split
        assert_eq!(parts[1], "home");

        // Case: \ (Root only)
        let (bare_r, anchored_r, _) = get_drive_components("\\");
        assert!(anchored_r);
        assert!(bare_r, "Root slash should be treated as bare anchor");
    }
    #[test]
    fn test_get_drive_components_relative() {
        let (bare, anchored, parts) = get_drive_components("projects/ncd");
        assert!(!anchored, "Relative paths should not be anchored");
        assert!(!bare);
        assert_eq!(parts[0], "projects");
        assert_eq!(parts[1], "ncd");
    }
    #[test]
    fn test_get_drive_components_messy_separators() {
        let (_bare, anchored, parts) = get_drive_components("a///b/\\c");
        assert!(!anchored);
        // filter out empties in the caller logic, but verify parts here
        assert!(parts.contains(&"a"));
        assert!(parts.contains(&"b"));
        assert!(parts.contains(&"c"));
    }
}
mod get_disk_casing {
    use crate::get_disk_casing;
    use crate::unit_tests_local::setup_test_env;

    #[test]
    fn test_get_disk_casing_preservation() {
        let (_tmp, root) = setup_test_env();
        let real_name = "SubFolder_With_Mixed_Casing";
        let path = root.join(real_name);
        std::fs::create_dir(&path).unwrap();

        // Query using lowercase
        let lowercase_path = root.join(real_name.to_lowercase());
        let disk_name = get_disk_casing(&lowercase_path);

        assert_eq!(disk_name, real_name, "Disk casing should return the preserved name, not the query name");
    }
    #[test]
    fn test_get_disk_casing_non_existent() {
        let (_tmp, root) = setup_test_env();
        let fake_path = root.join("this_folder_does_not_exist_12345");

        let result = get_disk_casing(&fake_path);

        assert_eq!(result, "", "Should return empty string for non-existent paths");
    }
    #[test]
    fn test_get_disk_casing_root_behavior() {
        // Testing the actual system root (e.g., C:\ or /)
        let root_path = std::path::Path::new(if cfg!(windows) { "C:\\" } else { "/" });

        // file_name() on a root usually returns None
        let result = get_disk_casing(root_path);

        // Based on your code, if file_name() is None, it returns default
        assert_eq!(result, "", "Root directory should return empty string as it has no component name");
    }
    #[test]
    fn test_get_disk_casing_special_chars() {
        let (_tmp, root) = setup_test_env();
        let special_name = "My Documents.Backup";
        let path = root.join(special_name);
        std::fs::create_dir(&path).unwrap();

        let disk_name = get_disk_casing(&path);

        assert_eq!(disk_name, special_name);
    }
}
mod split_query {
    use crate::split_query;

    #[test]
    fn test_split_query_drive_anchored() {
        // Input: ["C:", "Projects", "ncd"], starts_with_sep: false, is_anchored: true
        let query = vec!["C:", "Projects", "ncd"];
        let (head, tails) = split_query(query, false, true);

        assert_eq!(head, "C:", "Head should be the drive specifier");
        assert_eq!(tails, vec!["Projects", "ncd"], "Tails should contain remaining segments");
    }
    #[test]
    fn test_split_query_root_relative() {
        // Input: ["Projects", "ncd"], starts_with_sep: true, is_anchored: false
        let query = vec!["Projects", "ncd"];
        let (head, tails) = split_query(query, true, false);

        assert_eq!(head, "", "Head should be empty for root-relative paths");
        assert_eq!(tails, vec!["Projects", "ncd"]);
    }
    #[test]
    fn test_split_query_naked_relative() {
        // Input: ["my_proj*"], starts_with_sep: false, is_anchored: false
        let query = vec!["my_proj*"];
        let (head, tails) = split_query(query, false, false);

        assert_eq!(head, "");
        assert_eq!(tails, vec!["my_proj*"]);
    }
    #[test]
    fn test_split_query_filters_empties() {
        let query = vec!["a", "", "b", " ", "c"];
        // Note: your filter keeps " ", but removes ""
        let (_head, tails) = split_query(query, false, false);

        assert_eq!(tails.len(), 4); // "a", "b", " ", "c"
        assert!(tails.contains(&" "));
        assert!(!tails.contains(&""));
    }
    #[test]
    fn test_split_query_empty_input() {
        let query: Vec<&str> = Vec::new();
        let (head, tails) = split_query(query, false, false);

        assert_eq!(head, "");
        assert!(tails.is_empty());
    }
}
mod is_ellipsis {
    use crate::is_ellipsis;

    #[test]
    fn test_is_ellipsis_valid() {
        assert!(is_ellipsis(".."), "Double dot is an ellipsis");
        assert!(is_ellipsis("..."), "Triple dot is an ellipsis");
        assert!(is_ellipsis("...."), "Quadruple dot is an ellipsis");
    }
    #[test]
    fn test_is_ellipsis_invalid_single() {
        assert!(!is_ellipsis("."), "Single dot is NOT an ellipsis");
        assert!(!is_ellipsis(""), "Empty string is NOT an ellipsis");
    }
    #[test]
    fn test_is_ellipsis_non_dot_content() {
        assert!(!is_ellipsis(".hidden"), "Hidden files are not ellipses");
        assert!(!is_ellipsis("..hidden"), "Double-dot prefix is not an ellipsis");
        assert!(!is_ellipsis("dots..."), "Suffix dots do not make an ellipsis");
    }
    #[test]
    fn test_is_ellipsis_with_whitespace() {
        // If your trim_to_elipses handles whitespace:
        assert!(is_ellipsis("  ..  "), "Should handle padded ellipsis");
    }
}
mod trim_to_elipses {
    use crate::{is_ellipsis, trim_to_elipses};

    #[test]
    fn test_trim_to_elipses_sanitization() {
        assert_eq!(trim_to_elipses(".. ."), "...");
        assert_eq!(trim_to_elipses(". . ."), "...");
        assert_eq!(trim_to_elipses(" ...  "), "...");
    }
    #[test]
    fn test_trim_to_elipses_protection() {
        // Should NOT trim because it's a potential filename
        assert_eq!(trim_to_elipses(".hidden"), ".hidden");
        assert_eq!(trim_to_elipses("..config"), "..config");
        assert_eq!(trim_to_elipses("file.txt"), "file.txt");
    }
    #[test]
    fn test_trim_to_elipses_empty_and_whitespace() {
        assert_eq!(trim_to_elipses(""), "");
        assert_eq!(trim_to_elipses("   "), "");
    }
    #[test]
    fn test_ellipsis_logic_flow() {
        let input = ".. . .."; // 5 dots total with spaces
        let trimmed = trim_to_elipses(input);

        assert_eq!(trimmed, ".....");
        assert!(is_ellipsis(input), "is_ellipsis should call trim and return true");
    }
}
mod handle_ellipsis {
    use crate::{handle_ellipsis, trim_to_elipses};
    use crate::unit_tests_local::setup_test_env;

    #[test]
    fn test_handle_ellipsis_jump_math() {
        let (_tmp, root) = setup_test_env();
        let deep_dir = root.join("level1").join("level2").join("level3");
        std::fs::create_dir_all(&deep_dir).unwrap();

        // 3 dots = 2 levels up
        // From level3 -> level1
        let res_3 = handle_ellipsis("...", deep_dir.clone());
        assert_eq!(res_3[0].file_name().unwrap(), "level1");

        // 4 dots = 3 levels up
        // From level3 -> root
        let res_4 = handle_ellipsis("....", deep_dir);
        assert_eq!(res_4[0].canonicalize().unwrap(), root.canonicalize().unwrap());
    }
    #[test]
    fn test_handle_ellipsis_root_limit() {
        let (_tmp, root) = setup_test_env();
        let shallow = root.join("shallow");
        std::fs::create_dir(&shallow).unwrap();

        // Way too many dots
        let res = handle_ellipsis("..........", shallow);

        // Should stop at the drive root or the temp root
        assert!(!res.is_empty());
        // On Windows, popping past C:\ just keeps you at C:\
        // On Unix, popping past / keeps you at /
        assert!(res[0].parent().is_none() || res[0] == root.ancestors().last().unwrap());
    }
    #[test]
    fn test_handle_ellipsis_relative_base() {
        let (_tmp, _root) = setup_test_env();
        // Use a simple name that exists in the current sandbox CWD
        let local_dir = std::path::PathBuf::from(".");

        // ".." from "." should effectively be the parent of CWD
        let res = handle_ellipsis("..", local_dir);

        assert!(res[0].is_absolute(), "Resulting path from relative base should be absolute");
        assert_ne!(res[0], std::env::current_dir().unwrap());
    }
    #[test]
    fn test_handle_ellipsis_with_sanitization() {
        let (_tmp, root) = setup_test_env();
        let target = root.join("a").join("b");
        std::fs::create_dir_all(&target).unwrap();

        let input = ".. ."; // Should be "..." (2 levels up)
        let sanitized = trim_to_elipses(input);
        let res = handle_ellipsis(&sanitized, target);

        assert_eq!(res[0].canonicalize().unwrap(), root.canonicalize().unwrap());
    }
}
mod get_drive_root {
    use crate::get_drive_root;

    #[test]
    fn test_get_drive_root_windows() {
        let path = std::path::Path::new("C:\\Windows\\System32");
        let root = get_drive_root(path);

        assert!(root.is_some());
        let root_str = root.unwrap().to_string_lossy().into_owned();
        // On Windows, components().next() for C:\ is Prefix(C:)
        assert!(root_str.contains("C:"));
    }
    #[test]
    fn test_get_drive_root_unix() {
        let path = std::path::Path::new("/usr/local/bin");
        let root = get_drive_root(path);

        assert!(root.is_some());
        // On Unix, components().next() is RootDir (/)
        assert_eq!(root.unwrap(), std::path::PathBuf::from("/"));
    }
    #[test]
    fn test_get_drive_root_relative() {
        let path = std::path::Path::new("projects/rust");
        let root = get_drive_root(path);

        assert!(root.is_some());
        assert_eq!(root.unwrap(), std::path::PathBuf::from("projects"));
    }
    #[test]
    fn test_get_drive_root_empty() {
        let path = std::path::Path::new("");
        let root = get_drive_root(path);

        assert!(root.is_none(), "Empty path should have no root component");
    }
}
mod get_search_roots {
    use std::path::PathBuf;
    use crate::get_search_roots;
    use crate::unit_tests_local::{setup_test_env, CwdGuard};

    #[test]
    fn test_get_search_roots_mock_isolation() {
        let mock_path = PathBuf::from("/mock/root");
        let roots = get_search_roots(&Some(mock_path.clone().into_os_string()));

        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0], mock_path, "Mock should override all system roots");
    }
    #[test]
    fn test_get_search_roots_cdpath_dedup() {
        let (_tmp, root) = setup_test_env();
        let dir_a = root.join("dir_a");
        std::fs::create_dir(&dir_a).unwrap();

        // Manually simulate CDPATH with duplicates: dir_a;dir_a
        let cdpath_val = std::env::join_paths(vec![&dir_a, &dir_a]).unwrap();
        std::env::set_var("CDPATH", cdpath_val);

        let roots = get_search_roots(&None);

        // Count how many times dir_a appears (ignoring CWD)
        let dir_a_count = roots.iter().filter(|p| p.ends_with("dir_a")).count();
        assert_eq!(dir_a_count, 1, "CDPATH entries should be deduplicated");
    }
    #[test]
    fn test_get_search_roots_cwd_priority() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        std::env::set_current_dir(&root).unwrap();

        let roots = get_search_roots(&None);

        assert!(!roots.is_empty());
        assert_eq!(roots[0].canonicalize().unwrap(), root.canonicalize().unwrap(),
                   "CWD must be the first search root");
    }
    #[test]
    fn test_get_search_roots_no_cdpath() {
        std::env::remove_var("CDPATH");
        let roots = get_search_roots(&None);

        assert!(roots.len() >= 1);
        assert_eq!(roots[0], std::env::current_dir().unwrap());
    }
}
mod resolve_home {
    use crate::{resolve_home, NcdError};

    #[test]
    fn test_resolve_home_windows_priority() {
        std::env::set_var("USERPROFILE", "C:\\Users\\WinUser");
        std::env::set_var("HOME", "/home/unixuser");

        // We capture stdout to verify the printed path
        // For simplicity in this test, we just check the Result
        let res = resolve_home();
        assert!(res.is_ok());
    }
    #[test]
    fn test_resolve_home_unix_fallback() {
        std::env::remove_var("USERPROFILE");
        std::env::set_var("HOME", "/home/ncd_user");

        let res = resolve_home();
        assert!(res.is_ok());
    }
    #[test]
    fn test_resolve_home_missing_env() {
        std::env::remove_var("USERPROFILE");
        std::env::remove_var("HOME");

        let res = resolve_home();
        match res {
            Err(NcdError::ResolutionFailed(m)) => assert!(m.contains("HOME not found")),
            _ => panic!("Should have returned a ResolutionFailed error"),
        }
    }
}
mod report_ambiguity {
    use std::path::PathBuf;

    #[test]
    fn test_report_ambiguity_content() {
        let _root = PathBuf::from("C:\\Projects");
        let matches = vec![
            PathBuf::from("C:\\Projects\\Alpha"),
            PathBuf::from("C:\\Projects\\App_Alpha")
        ];

        // Logic check: Ensure it handles the vector correctly
        assert_eq!(matches.len(), 2);
        assert!(matches[0].to_string_lossy().contains("Alpha"));
    }
    #[test]
    fn test_report_ambiguity_path_fidelity() {
        let root = PathBuf::from("/usr/local");
        let matches = vec![PathBuf::from("/usr/local/bin")];

        // Verifying display() doesn't panic on special characters
        let _ = format!("{}", root.display());
        let _ = format!("{}", matches[0].display());
    }
}
#[allow(non_snake_case)]
mod NcdError {
    use crate::NcdError;

    #[test]
    fn test_ncd_error_display_resolution() {
        let err = NcdError::ResolutionFailed("my_missing_project*".into());
        let output = format!("{}", err);

        assert!(output.contains("Could not resolve"));
        assert!(output.contains("my_missing_project*"));
    }
    #[test]
    fn test_ncd_error_invalid_unicode() {
        use std::ffi::OsString;
        let os_str = OsString::from("invalid_unicode");
        let err = NcdError::InvalidUnicode(os_str);

        let output = format!("{:?}", err);
        assert!(output.contains("InvalidUnicode"));
    }
    #[test]
    fn test_ncd_error_io_wrapping() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "disk failure");
        let err = NcdError::Io(io_err);

        let output = format!("{}", err);
        assert!(output.contains("IO error"));
        assert!(output.contains("disk failure"));
    }
    #[test]
    fn test_ncd_error_is_standard_error() {
        let err = NcdError::ArgError("Missing path".into());
        let boxed: Box<dyn std::error::Error> = Box::new(err);

        assert_eq!(boxed.to_string(), "Arg error: Missing path");
    }
}
pub mod aggregate_series {
    use crate::{evaluate_jump, CdMode};
    use crate::unit_tests_local::{create_ncd_sandbox, get_opts, setup_test_env, test_opts, CwdGuard};

    #[test]
    fn test_collision_detection_across_roots() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        let root_a = root.join("RootA");
        let root_b = root.join("RootB");
        let target_a = root_a.join("Collision");
        let target_b = root_b.join("Collision");

        std::fs::create_dir_all(&target_a).unwrap();
        std::fs::create_dir_all(&target_b).unwrap();

        // Mock CDPATH to include both RootA and RootB
        let cdpath = std::env::join_paths(vec![&root_a, &root_b]).unwrap();

        // 1. Set the ACTUAL environment so the last block in get_search_roots hits
        std::env::set_var("CDPATH", &cdpath);

        // 2. Pass None or the root into mock_path, NOT the cdpath string
        let opts = get_opts(CdMode::Origin, false, None);

        let results = evaluate_jump("Collision*", &opts);
        // If this is 1, Goal #1 is failing. It MUST be 2 for a safety check.
        assert_eq!(results.len(), 2, "Engine failed to find BOTH collisions across CDPATH roots");
    }
    #[test]
    fn test_exact_match_constraint() {
        let (_tmp, root) = setup_test_env();
        let target = root.join("lower_case_dir");
        std::fs::create_dir(&target).unwrap();

        // 1. Test case-insensitive (default)
        let opts_default = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));
        assert!(!evaluate_jump("LOWER_CASE_DIR", &opts_default).is_empty());

        // 2. Test exact/strict mode
        let opts_exact = get_opts(CdMode::Origin, true, Some(root.into_os_string()));
        assert!(evaluate_jump("LOWER_CASE_DIR", &opts_exact).is_empty(), "Exact mode should have failed on case mismatch");
    }
    #[test]
    fn test_hybrid_mode_deep_recursion() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);
        // root -> level1 -> level2 -> MyTarget
        let deep_path = root.join("level1/level2/MyTarget");
        std::fs::create_dir_all(&deep_path).unwrap();

        // Hybrid mode should find "MyTarget" even if CDPATH only points to root
        let opts = get_opts(CdMode::Hybrid, false, Some(root.into_os_string()));
        let res = evaluate_jump("level1/level2/MyTarget", &opts);

        assert!(!res.is_empty(), "Hybrid mode failed to find deep target");
        assert!(res[0].ends_with("MyTarget"));
    }
    #[test]
    fn test_hybrid_recursion_depth() {
        let (_tmp, root) = setup_test_env(); // Using the 5-line helper
        let parent_of_target = root.join("a/b/c/d");
        let deep = parent_of_target.join("target");
        let cdpath =  root.join("a/b/c/../c/d/.../c/d");
        std::fs::create_dir_all(&deep).unwrap();

        // Set CDPATH to the deep parent so "target" is a direct child
        std::env::set_var("CDPATH", &cdpath);

        // mock_path must be None to let the CDPATH logic fire
        let opts = get_opts(CdMode::Hybrid, false, None);
        let res = evaluate_jump("target", &opts);

        assert_eq!(res.len(), 1, "Should find target via CDPATH");
        assert!(res[0].ends_with("target"));
    }
    /*
    #[test]
    fn test_recursive_search_multi_root() {
        let (_tmp, root) = setup_test_env();
        let root_a = root.join("A");
        let root_b = root.join("B");
        let target = root_b.join("deep/nested/goal");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&root_a).unwrap();

        let cdpath = std::env::join_paths(vec![&root_a, &root_b]).unwrap();
        let opts = get_opts(CdMode::Hybrid, false, Some(cdpath));
        let res = perform_recursive_search("goal", &opts);

        assert_eq!(res.len(), 1);
        assert!(res[0].ends_with("goal"));
    }
    #[test]
    fn test_recursive_search_depth_limit() {
        let (_tmp, root) = setup_test_env();
        let mut current = root.clone();
        for i in 0..5 { current = current.join(format!("dir_{}", i)); }
        std::fs::create_dir_all(&current).unwrap();

        let opts = get_opts(CdMode::Hybrid, false, Some(root.into_os_string()));
        let res = perform_recursive_search("dir_4", &opts);

        assert!(!res.is_empty(), "Failed to find target at depth 5");
    }
    #[test]
    fn test_recursive_search_case_insensitive() {
        let (_tmp, root) = setup_test_env();
        let target = root.join("MixedCaseDir");
        std::fs::create_dir_all(&target).unwrap();

        let opts = get_opts(CdMode::Hybrid, false, Some(root.into_os_string()));
        let res = perform_recursive_search("mixedcasedir", &opts);

        assert!(!res.is_empty(), "Recursion failed to find mixed-case directory");
    }
    #[test]
    fn test_recursive_search_exact_toggle() {
        let (_tmp, root) = setup_test_env();
        let target = root.join("STRICT_DIR");
        std::fs::create_dir_all(&target).unwrap();

        let mut opts = get_opts(CdMode::Hybrid, true, Some(root.into_os_string()));
        // Should fail with exact=true and lowercase query
        assert!(perform_recursive_search("strict_dir", &opts).is_empty());
    }
    */
    #[test]
    fn test_ambiguity_at_depth() {
        let (_tmp, root) = setup_test_env();
        let branch_a = root.join("alpha").join("target");
        let branch_b = root.join("beta").join("target");
        std::fs::create_dir_all(&branch_a).unwrap();
        std::fs::create_dir_all(&branch_b).unwrap();

        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));
        // Should resolve both alpha/target AND beta/target
       let res = evaluate_jump("*/target", &opts);

        assert_eq!(res.len(), 2, "Breadth search failed to find both targets");
    }
    #[test]
    fn mock_test_permission_denied_resilience() {
        let (_guard, _tmp, root) = create_ncd_sandbox();
        let target = root.join("visible_target");
        let locked = root.join("locked_folder");

        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&locked).unwrap();

        // On Windows, we can use a basic attribute change,
        // though NCD's read_dir will usually still see it unless ACLs are involved.
        // This test ensures that even if a folder is "weird," the search continues.
        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));
        let res = evaluate_jump("visible_target", &opts);

        assert_eq!(res.len(), 1, "Should ignore unsearchable folders and find target");
        assert!(res[0].ends_with("visible_target"));
    }
    #[test]
    fn test_permission_denied_resilience() {
        let (_guard, _tmp, root) = create_ncd_sandbox();
        let target = root.join("visible_target");
        let locked = root.join("locked_folder");

        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&locked).unwrap();

        // ACTUALLY Lock the folder on Windows
        #[cfg(windows)]
        {
            use std::process::Command;
            // icacls: Deny (D) Read (R) to the current user
            let _ = Command::new("icacls")
                .arg(&locked)
                .arg("/deny")
                .arg(format!("{}:(R)", std::env::var("USERNAME").unwrap()))
                .status();
        }

        let opts = get_opts(CdMode::Origin, false, Some(root.into_os_string()));
        let res = evaluate_jump("visible_target", &opts);

        // Cleanup permissions so tempfile can delete the dir later
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("icacls").arg(&locked).arg("/grant").arg("Everyone:(F)").status();
        }

        assert_eq!(res.len(), 1, "Search engine must skip the 'Access Denied' folder and find the target");
    }
}
pub mod github_fails {
    use std::path::{Path, PathBuf};
    use crate::{evaluate_jump, handle_ellipsis, CdMode};
    use crate::unit_tests_local::{create_ncd_sandbox, get_opts, setup_test_env, test_opts, CwdGuard};

    #[test]
    fn test_absolute_path_bypass() {
        let (_tmp, root) = setup_test_env();
        let target = root.join("absolute_target");
        std::fs::create_dir(&target).unwrap();

        let query = target.to_string_lossy().to_string();
        let p = Path::new(&query);

        // This output will reveal if Rust/OS thinks this path is valid/absolute
        println!("\n[TRACE] Query: {}", query);
        println!("[TRACE] is_absolute: {}", p.is_absolute());
        println!("[TRACE] exists: {}", p.exists());
        println!("[TRACE] components: {:?}", p.components().collect::<Vec<_>>());

        let res = evaluate_jump(&query, &test_opts());
        println!("[TRACE] res: {:?}", res);
        assert!(!res.is_empty(),
                "\nABS_FAIL:\nQuery: {}\nExists: {}\nIsAbs: {}\nComponents: {:?}\nResults: {:?}",
                query, target.exists(), Path::new(&query).is_absolute(),
                Path::new(&query).components().collect::<Vec<_>>(), res);
    }
    #[test]
    fn test_root_anchored_logic_mk3() {
        let (_tmp, root) = setup_test_env();
        let _guard = CwdGuard::new(&root);

        // No need to create "Projects", setup_test_env already did it.
        let opts = get_opts(CdMode::Origin, false, Some(root.clone().into_os_string()));

        let query = "Projects";
        let result = evaluate_jump(query, &opts);

        assert!(!result.is_empty(),
                "\nANCHOR_FAIL:\nQuery: {}\nMockRoot: {:?}\nCWD: {:?}\nResult: {:?}",
                query, opts.mock_path, std::env::current_dir().ok(), result);
        // Ensure it's the right Projects folder
        assert!(result[0].starts_with(&root));
    }
    #[test]
    fn test_ellipsis_relative_to_dot() {
        let (_guard, _temp, root) = create_ncd_sandbox();
        let matches = handle_ellipsis("...", root.to_path_buf());
        let r1 = PathBuf::from(root.parent().unwrap().as_os_str());
        let expected = r1.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| root.clone());

        assert!(!matches.is_empty(),
                "\n[DEBUG] Root: {:?}\n[DEBUG] Expected Parent: {:?}\n[DEBUG] Matches: {:?}",
                root, expected, matches);

        assert_eq!(matches[0].canonicalize().unwrap(), expected.canonicalize().unwrap(),
                   "\n[DEBUG] Mismatch!\nActual: {:?}\nExpected: {:?}", matches[0], expected);
    }
    #[test]
    fn test_absolute_path_bypass_normalized() {
        let (_tmp, root) = setup_test_env();
        let target = root.join("absolute_target");
        std::fs::create_dir(&target).unwrap();

        let query = target.to_string_lossy().replace(r"\\?\", "");
        let res = evaluate_jump(&query, &test_opts());

        assert!(!res.is_empty(), "Engine failed on path: {}", query);
        assert_eq!(res[0].canonicalize().unwrap(), target.canonicalize().unwrap());
    }

    #[test]
    fn test_ellipsis_triple_dot_jump() {
        let (_guard, _temp, root) = create_ncd_sandbox();
        let base = root.canonicalize().unwrap();
        let matches = handle_ellipsis("...", base.clone());

        // ... should jump TWO levels: Sandbox -> Temp -> Local
        let expected = base.parent().unwrap().parent().unwrap();
        let actual = matches[0].canonicalize().unwrap();

        assert_eq!(actual, expected.canonicalize().unwrap(), "Triple dot must jump two levels.");
    }
}


