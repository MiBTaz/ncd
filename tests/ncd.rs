#[cfg(test)]
fn create_integrated_sandbox() -> (CwdGuard, tempfile::TempDir, String, String) {
    let guard = CwdGuard::new(); // Capture V:\Projects\ncd
    let tmp = tempfile::tempdir().unwrap();
    let root_path = tmp.path();

    // Force the test process onto the sandbox drive/folder
    std::env::set_current_dir(root_path).unwrap();

    let proj_path = root_path.join("Projects");
    std::fs::create_dir_all(&proj_path).unwrap();

    let root_str = root_path.to_string_lossy().into_owned();
    let sep = std::path::MAIN_SEPARATOR;

    // Standardized Anchor: \Users\...\Temp
    let anchor = if let Some(pos) = root_str.find(':') {
        format!("{}{}", sep, root_str[pos + 1..].trim_start_matches(sep))
    } else {
        root_str.clone()
    };

    (guard, tmp, root_str, anchor)
}

#[cfg(test)]
pub struct CwdGuard {
    old_cwd: std::path::PathBuf,
}
#[cfg(test)]
impl CwdGuard {
    pub fn new() -> Self {
        Self {
            old_cwd: std::env::current_dir().expect("Failed to get CWD"),
        }
    }
}
#[cfg(test)]
impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.old_cwd);
    }
}

pub const MAIN_SEPARATOR: char = std::path::MAIN_SEPARATOR;

struct TestCase {
    input: &'static str,
    expected_out: &'static str,
    should_succeed: bool,
    env_cdpath_val: Option<String>,
}

mod remotes {
    use std::fs;
    use std::path::{Path};
    use assert_cmd::{cargo_bin_cmd};
//    use assert_cmd::assert::OutputAssertExt;
//    use assert_cmd::prelude::CommandCargoExt;
    use predicates::boolean::PredicateBooleanExt;
    use predicates::prelude::predicate;
    use crate::{create_integrated_sandbox, TestCase, MAIN_SEPARATOR};
//    use assert_cmd::prelude::*;


    #[test]
    fn test_home_jump() {
        // 1. Setup: Direct binary invocation via cargo_bin_cmd
        let mut cmd = cargo_bin_cmd!("ncd");

        // 2. Execution: The tilde expansion command
        cmd.arg("~")
            .assert()
            .success()
            // 3. Verification: Ensure it resolves to a standard OS home root
            .stdout(
                predicate::str::contains("Users") // Windows/macOS
                    .or(predicate::str::contains("home")) // Linux
                    .or(predicate::str::contains("Documents and Settings")) // Legacy Win
            );
    }
    #[test]
    fn test_invalid_path_fails() {
        // 1. Setup: Direct binary invocation
        let mut cmd = cargo_bin_cmd!("ncd");

        // 2. Execution: Provide a query guaranteed to fail
        cmd.arg("non_existent_path_999")
            .assert()
            // 3. Verification: Exit code must be non-zero
            .failure()
            // 4. Verification: Error message must be descriptive
            .stderr(predicate::str::contains("Could not resolve"));
    }
    #[test]
    fn test_root_anchored_drive_resolution_remote() {
        let (_guard, _tmp, mock_root, _drv) = create_integrated_sandbox();
        let mut cmd = cargo_bin_cmd!("ncd");

        let leaf = "Projects";
        let p = format!("{}/{}", mock_root, leaf);
        println!("{}", p);
        cmd.arg(p)
            .env("NCD_MOCK_ROOT", &mock_root) // Redirects binary's root logic
            .assert()
            .success()
            .stdout(predicates::str::contains(leaf));
    }
    #[test]
    fn test_root_anchored_path_resolution_remote() {
        let (_guard, _tmp, _root_abs, _path_abs) = create_integrated_sandbox();
        let mut cmd = cargo_bin_cmd!("ncd");
        let leaf = "Projects";
        let p = format!("{}/{}", _path_abs, leaf);
        println!("{}", p);
        cmd.arg(p)
            .env("NCD_MOCK_ROOT", &_root_abs) // Redirects binary's root logic
            .assert()
            .success()
            .stdout(predicates::str::contains(leaf));
    }
    #[test]
    fn test_anchored_hybrid_sandbox() {
        let (_guard, _tmp, _root, _path) = create_integrated_sandbox();
        let root = Path::new(&_root);
        let deep_path = root.join("Projects").join("ncd").join("src");
        fs::create_dir_all(&deep_path).unwrap();

        let sep = std::path::MAIN_SEPARATOR;
        let _root_str = root.to_string_lossy();

        // Strip "D:" but ALSO strip the leading "\" to prevent the "\\" UNC trap
        let anchored_root = _path;

        // Build the query starting with a SINGLE slash
        let query = format!("{}{}Projects{}nc{}sr", anchored_root, sep, sep, sep);

        cargo_bin_cmd!("ncd")
            .arg("--cd=hybrid")
            .arg("-#")
            .arg(query)
            .assert()
            .success()
            .stdout(predicates::str::contains("src"));
    }
    #[test]
    fn test_root_anchored_resolution_portable() {
        let (_guard, _tmpdir, _root, _path) = create_integrated_sandbox();
        let _dir = _tmpdir;
        let root = Path::new(&_root);

        // Create a dummy folder in our temp path
        let test_folder = "ncd_integration_test_root";
        fs::create_dir_all(root.join(test_folder)).unwrap();

        // Strip drive to get the \Anchor
        let _root_str = root.to_string_lossy();
        let anchor = _path;

        // Query: \Users\...\Temp\ncd_integration_test_root
        let sep = std::path::MAIN_SEPARATOR;
        let query = format!("{}{}{}", anchor, sep, test_folder);

        let mut cmd = cargo_bin_cmd!("ncd");
        cmd.arg(query)
            .assert()
            .success()
            .stdout(predicate::str::contains(test_folder));
    }
    #[test]
    fn test_root_anchored_resolution() {
        // 1. Setup a real directory to find
        let (_guard, _tmpdir, _root, _path) = create_integrated_sandbox();
        let root = Path::new(&_root);
        let test_folder = "ncd_cluster_test";
        fs::create_dir_all(root.join(test_folder)).unwrap();

        // 2. Create the anchor (strip 'C:' or 'D:')
        let _root_str = root.to_string_lossy();
        let anchor = _path;

        // 3. Build the absolute-but-anchored query: \Users\...\Temp\ncd_cluster_test
        let sep = std::path::MAIN_SEPARATOR;
        let query = format!("{}{}{}", anchor, sep, test_folder);

        let mut cmd = cargo_bin_cmd!("ncd");
        cmd.arg(query)
            .assert()
            .success()
            // Ensure the output contains our specific test folder
            .stdout(predicate::str::contains(test_folder));
    }
    #[test]
    fn exhaustive_matrix_check() {
        let (_guard, _tmp, mock_root, _anchor) = create_integrated_sandbox();
        let sep = std::path::MAIN_SEPARATOR;

        // Build the physical playground inside the sandbox
        let playground = std::path::Path::new(&mock_root).join("test_playground");
        let sub_dir = playground.join("project").join("src");
        fs::create_dir_all(&sub_dir).unwrap();

        // Test 1: Empty input (Home/Current Dir)
        cargo_bin_cmd!("ncd")
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .success()
            .stdout(predicates::str::is_empty().not());

        // Test 2: Dots traversal
        // Using current_dir ensures "..." has a context to traverse from
        cargo_bin_cmd!("ncd")
            .current_dir(&sub_dir)
            .arg("...")
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .success();

        // Test 3: CDPATH & Tail logic
        // We use mock_root as the CDPATH so 'test_playground' is discoverable
        cargo_bin_cmd!("ncd")
            .env("CDPATH", &mock_root)
            .arg(format!("test_playground{}project{}src", sep, sep))
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .success()
            .stdout(predicates::str::contains("src"));
    }
    #[test]
    fn test_mode_switching_logic_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();
        let _sep = std::path::MAIN_SEPARATOR;

        // 1. Create a isolated structure: [mock_root]/search_base/target_dir
        let search_base = std::path::Path::new(&mock_root).join("search_base");
        let target_dir = search_base.join("ncd_test_dir");
        std::fs::create_dir_all(&target_dir).unwrap();

        // 2. Test 'target' mode specifically
        // We point CDPATH to search_base so ncd finds ncd_test_dir inside it
        cargo_bin_cmd!("ncd")
            .arg("--cd=target")
            .arg("ncd_test_dir")
            .env("CDPATH", &search_base)
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .success()
            .stdout(predicates::str::contains("ncd_test_dir"));
    }
    #[test]
    fn test_target_vs_origin_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();

        let level1 = std::path::Path::new(&mock_root).join("level1");
        let level2 = level1.join("level2");
        let empty_sibling = std::path::Path::new(&mock_root).join("empty_dir");

        std::fs::create_dir_all(&level2).unwrap();
        std::fs::create_dir_all(&empty_sibling).unwrap();

        // Test 1: Target mode SUCCESS
        // It finds 'level1' because it looks AT the path provided in CDPATH
        cargo_bin_cmd!("ncd")
            .arg("--cd=target")
            .arg("level1")
            .env("CDPATH", &level1)
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .success();

        // Test 2: Origin mode FAILURE
        // Looking INSIDE 'empty_sibling' for 'level2' -> result: Not Found
        cargo_bin_cmd!("ncd")
            .arg("--cd=origin")
            .arg("level2")
            .env("CDPATH", &empty_sibling)
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .failure();
    }
    #[test]
    fn test_hyphen_jump_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();

        // Create a dummy "previous" directory in the sandbox
        let fake_old_pwd = std::path::Path::new(&mock_root).join("old_dir");
        std::fs::create_dir_all(&fake_old_pwd).unwrap();
        let old_pwd_str = fake_old_pwd.to_string_lossy();

        cargo_bin_cmd!("ncd")
            .arg("-")
            .env("OLDPWD", &*old_pwd_str)
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .success()
            .stdout(predicates::str::contains(old_pwd_str));
    }
    #[test]
    fn test_exhaustive_matrix() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();
        let deep_dir = std::path::Path::new(&mock_root).join("a").join("b").join("c");
        std::fs::create_dir_all(&deep_dir).unwrap();

        let cases = vec![
            TestCase { input: "~", expected_out: "Users", should_succeed: true, env_cdpath_val: None },
            TestCase { input: "...", expected_out: "", should_succeed: true, env_cdpath_val: None },
            TestCase { input: "nonexistent_abc_123", expected_out: "Could not resolve", should_succeed: false, env_cdpath_val: None },
        ];

        for case in cases {
            let mut cmd = cargo_bin_cmd!("ncd");
            // Force process into the deep sandbox for traversal tests
            cmd.current_dir(&deep_dir).env("HOME", &mock_root).env("NCD_MOCK_ROOT", &mock_root);

            if let Some(path) = case.env_cdpath_val { cmd.env("CDPATH", path); }

            let assert = cmd.arg(case.input).assert();
            if case.should_succeed {
                assert.success().stdout(predicates::str::contains(case.expected_out));
            } else {
                assert.failure().stderr(predicates::str::contains(case.expected_out));
            }
        }
    }
    #[test]
    fn test_ambiguity_protection_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();

        // 1. Setup multiple colliding directories in the sandbox
        let base = std::path::Path::new(&mock_root);
        std::fs::create_dir(base.join("Project_Alpha")).unwrap();
        std::fs::create_dir(base.join("Project_Alpha_Beta")).unwrap();

        // 2. ncd should fail if a glob matches both without a clear winner
        cargo_bin_cmd!("ncd")
            .env("CDPATH", &mock_root)
            .env("NCD_MOCK_ROOT", &mock_root)
            .arg("Project_Alpha*")
            .assert()
            .failure()
            .stderr(predicates::str::contains("Ambiguous match"));
    }
    #[test]
    fn test_list_mode_returns_multiple_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();

        // Setup colliding directories
        let base = std::path::Path::new(&mock_root);
        std::fs::create_dir(base.join("Alpha_One")).unwrap();
        std::fs::create_dir(base.join("Alpha_Two")).unwrap();

        // Verify --list collects all matches instead of failing
        cargo_bin_cmd!("ncd")
            .arg("--list")
            .arg("Alpha*")
            .env("CDPATH", &mock_root)
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .success()
            .stdout(predicates::str::contains("Alpha_One"))
            .stdout(predicates::str::contains("Alpha_Two"));
    }
    #[test]
    fn test_parent_ambiguity_guard_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();
        let root = std::path::Path::new(&mock_root);
        let work_dir = root.join("root").join("work_dir");

        std::fs::create_dir_all(root.join("root").join("match_1")).unwrap();
        std::fs::create_dir_all(root.join("root").join("match_2")).unwrap();
        std::fs::create_dir_all(&work_dir).unwrap();

        cargo_bin_cmd!("ncd")
            .current_dir(&work_dir)
            .arg("..\\match*")
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .failure()
            .stderr(predicates::str::contains("Ambiguous match"));
    }
    #[test]
    fn test_invalid_path_fails_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();

        cargo_bin_cmd!("ncd")
            .arg("non_existent_path_999")
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .failure()
            .stderr(predicates::str::contains("Could not resolve"));
    }
    #[test]
    fn test_parent_glob_isolation_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();
        let root = std::path::Path::new(&mock_root);

        // Setup: root/neighbor_target
        // Setup: root/current_work_dir/neighbor_target (The Distractor)
        let neighbor = root.join("neighbor_target");
        let work_dir = root.join("current_work_dir");
        let distractor = work_dir.join("neighbor_target");

        std::fs::create_dir_all(&neighbor).unwrap();
        std::fs::create_dir_all(&distractor).unwrap();

        cargo_bin_cmd!("ncd")
            .current_dir(&work_dir)
            .arg("..\\neigh*")
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .success()
            .stdout(predicates::str::contains("current_work_dir").not())
            .stdout(predicates::str::contains("neighbor_target"));
    }
    #[test]
    fn test_ambiguity_aborts_stdout_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();

        // Create colliding targets
        let base = std::path::Path::new(&mock_root);
        std::fs::create_dir(base.join("Match_A")).unwrap();
        std::fs::create_dir(base.join("Match_B")).unwrap();

        cargo_bin_cmd!("ncd")
            .arg("Match_*")
            .env("CDPATH", &mock_root)
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .failure()
            .stdout(predicates::str::is_empty())
            .stderr(predicates::str::contains("Ambiguous match"));
    }
    #[test]
    fn test_trailing_separator_scrub_cluster() {
        let (_guard, _tmp, _, anchor) = create_integrated_sandbox();

        // 1. Test deep path scrubbing (trailing slash)
        let path_str = format!("{}Projects{}ncd{}", &anchor, MAIN_SEPARATOR, MAIN_SEPARATOR);
        let cleaned = if path_str.len() > 3 {
            path_str.trim_end_matches(|c| c == '\\' || c == '/').to_string()
        } else {
            path_str.clone()
        };
        // It should have removed the LAST separator
        assert!(!cleaned.ends_with(MAIN_SEPARATOR));

        // 2. Test Root Protection (Should NOT scrub the C:\)
        let root = &anchor;
        let cleaned_root = if root.len() > 3 {
            root.trim_end_matches(|c| c == '\\' || c == '/').to_string()
        } else {
            root.clone()
        };
        // Anchor is likely "C:\" or "\tmp", so it should remain intact
        assert_eq!(cleaned_root, *root);
    }
    #[test]
    fn test_crazy_dot_resolution_cluster() {
        let (_guard, _tmp, mock_root, _anchor) = create_integrated_sandbox();

        // The current directory is now the temp sandbox
        let mut cmd = cargo_bin_cmd!("ncd");
        let assert = cmd.arg(" . ").assert();

        let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

        // We expect the output to be the cleaned current dir (mock_root)
        // Trim both to ignore trailing newlines/separators
        assert_eq!(output.trim().trim_end_matches('\\'), mock_root.trim_end_matches('\\'));
    }
    #[test]
    fn test_crazy_parent_resolution_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();
        let deep_dir = std::path::Path::new(&mock_root).join("subdir");
        std::fs::create_dir(&deep_dir).unwrap();

        // Move into the subdir so ".." has a valid target within our sandbox
        let mut cmd = cargo_bin_cmd!("ncd");
        cmd.current_dir(&deep_dir).arg(" .. ");

        let assert = cmd.assert().success();
        let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

        // The parent of subdir is mock_root
        assert_eq!(output.trim().trim_end_matches('\\'), mock_root.trim_end_matches('\\'));
    }
    #[test]
    fn test_target_vs_origin_mk2_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();

        // Create the actual directory we want to match
        let target_path = std::path::Path::new(&mock_root).join("ncd_project");
        std::fs::create_dir_all(&target_path).unwrap();

        // Target mode: should find 'ncd_project' because it is the folder in CDPATH
        cargo_bin_cmd!("ncd")
            .arg("--cd=target")
            .arg("ncd_project")
            .env("CDPATH", &target_path)
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .success()
            .stdout(predicates::str::contains("ncd_project"));
    }
    #[test]
    fn test_target_vs_origin_mk3_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();
        let root_path = std::path::Path::new(&mock_root);

        // CDPATH points here
        let isolated_path = root_path.join("isolated_origin");
        std::fs::create_dir_all(&isolated_path).unwrap();

        // The target is tucked away in a sibling folder that is NOT in CDPATH
        let hidden_nest = root_path.join("hidden_nest");
        let target = hidden_nest.join("should_not_find_me");
        std::fs::create_dir_all(&target).unwrap();

        cargo_bin_cmd!("ncd")
            .arg("--cd=origin")
            .arg("should_not_find_me")
            .env("CDPATH", &isolated_path)
            .env("NCD_MOCK_ROOT", &mock_root)
            .assert()
            .failure();
    }
    #[test]
    fn test_primitive_dot_resolution_integration_cluster() {
        let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();

        let output_raw = cargo_bin_cmd!("ncd").arg(" . ").assert().success().get_output().stdout.clone();
        let output = String::from_utf8(output_raw).unwrap();

        // Clean the expected path of the Windows UNC prefix if present
        let current = mock_root.replace(r"\\?\", "");

        assert_eq!(output.trim().trim_end_matches('\\'), current.trim_end_matches('\\'));
    }
}

mod final_series {
    mod last_series {
        use assert_cmd::cargo_bin_cmd;
        use predicates::boolean::PredicateBooleanExt;
        use predicates::prelude::predicate;
        use crate::create_integrated_sandbox;

        #[test]
        fn test_dot_resolution_respects_origin_mode() {
            let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();
            let cdpath_target = std::path::Path::new(&mock_root).join("origin_dir");
            std::fs::create_dir_all(&cdpath_target).unwrap();

            // If we are in origin mode, '.' should potentially point to the CDPATH entry
            // Currently, your 'run' logic returns CWD immediately. This test checks that behavior.
            cargo_bin_cmd!("ncd")
                .arg("--cd=origin")
                .arg(".")
                .env("CDPATH", &cdpath_target)
                .assert()
                .success()
                .stdout(predicate::str::contains(&mock_root)); // Check if it returns CWD or CDPATH
        }
        #[test]
        fn test_hyphen_jump_variable_priority() {
            let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();
            let fake_old_pwd = std::path::Path::new(&mock_root).join("previously_here");
            std::fs::create_dir_all(&fake_old_pwd).unwrap();

            cargo_bin_cmd!("ncd")
                .arg("-")
                .env("OLDPWD", &fake_old_pwd)
                .assert()
                .success()
                .stdout(predicate::str::contains("previously_here"));
        }
        #[test]
        fn test_ambiguity_exit_status() {
            let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();
            // Create two identical targets to force ambiguity
            std::fs::create_dir_all(std::path::Path::new(&mock_root).join("dir1/target")).unwrap();
            std::fs::create_dir_all(std::path::Path::new(&mock_root).join("dir2/target")).unwrap();

            cargo_bin_cmd!("ncd")
                .arg("target")
                .env("CDPATH", &mock_root)
                .assert()
                .failure(); // Testing if your report_ambiguity triggers an exit 1
        }
        #[test]
        fn test_run_short_circuit_returns_fqpn() {
            let (_guard, tmp, mock_root, _) = create_integrated_sandbox();
            let sub = tmp.path().join("level1");
            std::fs::create_dir(&sub).unwrap();
            std::env::set_current_dir(&sub).unwrap();

            cargo_bin_cmd!("ncd").arg(".").assert().success()
                .stdout(predicate::eq(format!("{}\n", sub.display().to_string().replace(r"\\?\", ""))));

            cargo_bin_cmd!("ncd").arg("..").assert().success()
                .stdout(predicate::eq(format!("{}\n", mock_root.replace(r"\\?\", ""))));
        }
        #[test]
        fn test_run_scrubs_unc_prefix() {
            let (_guard, _tmp, _mock_root, _) = create_integrated_sandbox();
            // We simulate a case where the internal resolver might return a UNC path
            // and verify the println! in run() strips it.
            cargo_bin_cmd!("ncd").arg(".").assert().success()
                .stdout(predicate::str::starts_with(r"\\?\").not());
        }
        #[test]
        fn test_run_special_char_dispatch() {
            let (_guard, _tmp, mock_root, _) = create_integrated_sandbox();
            let old_dir = std::path::Path::new(&mock_root).join("old_location");
            std::fs::create_dir(&old_dir).unwrap();

            cargo_bin_cmd!("ncd").arg("-").env("OLDPWD", &old_dir).assert().success()
                .stdout(predicate::str::contains("old_location"));

            // Home jump should succeed if HOME/USERPROFILE is set
            cargo_bin_cmd!("ncd").arg("~").assert().success();
        }
    }
}