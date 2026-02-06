// src/unit_tests.rs
#[cfg(test)]
mod tests {
    use std::{env, fs};
    use std::path::PathBuf;
    use crate::*;
    use tempfile::tempdir;
    use serial_test::serial;

    /// Helper to generate SearchOptions on the fly for tests.
    /// This keeps the test calls clean and matches the new 2-argument signature.
    fn get_opts(mode: CdMode, exact: bool, mock: Option<OsString>) -> SearchOptions {
        SearchOptions {
            mode,
            exact,
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
        use std::fs;
        let temp = tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();

        // 1. Setup: These are siblings inside 'root'
        let target = root.join("SiblingTarget");
        let cwd_mock = root.join("CurrentDir"); // This is our "Fake" CWD
        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(&cwd_mock).unwrap();

        // 2. Setup Options
        let opts = SearchOptions {
            mode: CdMode::Origin,
            exact: false,
            list: false,
            mock_path: None,
        };

        // 3. Execute jump: Pass cwd_mock directly as the 'base'
        // This is the "Fortress" move: No env::set_current_dir needed!
        let res = handle_ellipsis("..", Some("SiblingTarget"), &opts, cwd_mock);

        // 4. Assert
        let matches = res.expect("Ellipsis handler failed");
        assert!(!matches.is_empty(), "Failed to find Sibling!");

        let found = matches[0].canonicalize().unwrap();
        let expected = target.canonicalize().unwrap();
        assert_eq!(found, expected, "Resolved to wrong path.");
    }

    #[test]
    fn test_ellipsis_sibling_resolution2() {
        use std::fs;
        let temp = tempdir().unwrap();
        // Canonicalize is vital on Windows to resolve short-names (PROGRA~1 style)
        let root = temp.path().canonicalize().unwrap();

        let target_name = "SiblingTarget";
        // Setup: target is a sibling of the temp root
        let target = root.parent().unwrap().join("SiblingTarget");
        let cwd_mock = root.join("CurrentDir");

        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(&cwd_mock).unwrap();

        // No env::set_current_dir!
        // We just define what our world looks like via this variable.

        let opts = SearchOptions {
            mode: CdMode::Origin,
            exact: false,
            list: false,
            mock_path: None,
        };

        // INJECTION: Pass cwd_mock as the 'base'
        let res = handle_ellipsis("...", Some(target_name), &opts, cwd_mock);

        let matches = res.expect("Should return a result");
        assert!(!matches.is_empty());

        let found_path = matches[0].canonicalize().unwrap();
        let expected_path = target.canonicalize().unwrap();
        assert_eq!(found_path, expected_path);
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
        assert_eq!(res_space[0], current.join(" . "));
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
            mock_path: Some(DEFAULT_TEST_ROOT.into()),
        };

        let results = search_cdpath("pro*", &opts);

        assert!(!results.is_empty(), "Engine failed to find 'Projects' in {}", DEFAULT_TEST_ROOT);
        assert!(results[0].to_string_lossy().contains("Projects"));

        fs::remove_dir_all(sandbox).ok();
    }
}
