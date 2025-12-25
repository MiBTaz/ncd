#[cfg(test)]
mod tests {
    use std::{env, fs};
    use std::path::PathBuf;
    use crate::*;
    use tempfile::tempdir;
    use crate::DEFAULT_TEST_ROOT;

    #[cfg(test)]
    fn get_test_root() -> PathBuf {
        // 1. Try Environment Variable override
        if let Ok(env_path) = env::var("NCD_TEST_DIR") {
            let p = PathBuf::from(env_path);
            if fs::create_dir_all(&p).is_ok() { return p; }
        }

        // 2. Try the preferred persistent root (your variable)
        let persistent_root = PathBuf::from(DEFAULT_TEST_ROOT);
        if fs::create_dir_all(&persistent_root).is_ok() {
            return persistent_root;
        }

        // 3. Absolute fallback: OS Temp Directory
        let temp = env::temp_dir().join("ncd_tests");
        fs::create_dir_all(&temp).ok();
        temp
    }


    // Detect valid persistent test root
    #[test]
    fn test_junction_follow() {
        // V:\temp is a junction to V:\tmp
        let root = PathBuf::from("V:\\temp\\ncd_tests");
        if !root.exists() { return; } // Skip if environment differs

        let test_dir = root.join("JunctionFollow");
        fs::create_dir_all(&test_dir).ok();

        let res = search_cdpath("JunctionFollow", CdMode::Origin, false, false, Some(root.into_os_string()));
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
        let mock = Some(root.into_os_string());

        // Fuzzy
        let res_f = search_cdpath("mixedcase123", CdMode::Origin, false, false, mock.clone());
        assert!(!res_f.is_empty());

        // Exact
        let res_e = search_cdpath("mixedcase123", CdMode::Origin, true, false, mock);
        if actual_name == "mixedcase123" { assert!(!res_e.is_empty()); }
        else { assert!(res_e.is_empty()); }
    }

    #[test]
    fn test_dot_traversal() {
        let result = evaluate_jump("...", CdMode::Origin, false, false);
        assert!(!result.is_empty());
        let current = env::current_dir().unwrap();
        let expected = current.parent().unwrap().parent().unwrap();
        assert_eq!(result[0], expected);
    }

    #[test]
    fn test_extreme_ellipsis() {
        let result = evaluate_jump(".....", CdMode::Origin, false, false);
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

        let res_fuzzy = search_cdpath("myproject", CdMode::Origin, false, false, mock_env.clone());
        assert!(!res_fuzzy.is_empty());

        let res_exact = search_cdpath("myproject", CdMode::Origin, true, false, mock_env);
        assert!(res_exact.is_empty());
    }

    #[test]
    fn test_hybrid_mode() {
        let dir = tempdir().unwrap();
        let bookmark = dir.path().join("Work");
        fs::create_dir(&bookmark).unwrap();
        let mock_cdpath = Some(bookmark.as_os_str().to_os_string());

        let res = search_cdpath("Work", CdMode::Hybrid, true, false, mock_cdpath);
        assert!(!res.is_empty());
        assert_eq!(res[0].canonicalize().unwrap(), bookmark.canonicalize().unwrap());
    }

    #[test]
    fn test_root_anchored_logic() {
        let result = evaluate_jump("\\Projects", CdMode::Origin, false, false);
        assert!(!result.is_empty());
        let path_str = result[0].to_string_lossy();
        assert!(path_str.contains(":\\Projects"));
    }

    #[test]
    fn test_wildcard_regex_logic() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("testing.1")).unwrap();
        let mock_path = Some(dir.path().as_os_str().to_os_string());

        let res = search_cdpath("test*.*", CdMode::Origin, false, false, mock_path);
        assert!(!res.is_empty());
        assert!(res[0].to_string_lossy().contains("testing.1"));
    }

    #[test]
    fn test_parent_globbing() {
        let dir = tempdir().unwrap();
        let parent = dir.path().join("parent_dir");
        let child = parent.join("child_glob");
        fs::create_dir_all(&child).unwrap();

        // Set CWD to the child
        let original_cwd = env::current_dir().unwrap();
        env::set_current_dir(&child).unwrap();

        // Try to jump up one level and find "child_glob" via glob
        // '..' is parent, 'child*' is the search
        let res = evaluate_jump("..\\child*", CdMode::Origin, false, false);

        env::set_current_dir(original_cwd).unwrap();

        assert!(!res.is_empty());
        assert!(res[0].to_string_lossy().contains("child_glob"));
    }
    #[test]
    fn test_root_anchored_wildcard() {
        let root = get_test_root();
        let test_dir = root.join("WildcardTarget");
        let _ = fs::create_dir_all(&test_dir);

        // Navigate to the root of our test space
        let original_cwd = env::current_dir().unwrap();
        env::set_current_dir(&root).unwrap();

        // Search for the wildcard relative to where we are
        let query = "Wildcard*";
        let res = evaluate_jump(query, CdMode::Hybrid, false, false);

        // Cleanup
        env::set_current_dir(original_cwd).unwrap();

        assert!(!res.is_empty(), "Wildcard expansion failed in test root");
        assert!(res[0].to_string_lossy().contains("WildcardTarget"));
    }
}

