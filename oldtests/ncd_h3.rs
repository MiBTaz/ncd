use assert_cmd::cargo::cargo_bin_cmd;

struct TestCase {
    input: &'static str,
    expected_out: &'static str,
    should_succeed: bool,
    // Using an underscore prefix or rename field can satisfy some linters
    env_cdpath_val: Option<&'static str>,
}

#[test]
fn test_exhaustive_matrix() {
    let cases = vec![
        TestCase { input: "~", expected_out: "Users", should_succeed: true, env_cdpath_val: None },
        TestCase { input: "...", expected_out: "", should_succeed: true, env_cdpath_val: None },
        TestCase { input: "nonexistent", expected_out: "Could not resolve", should_succeed: false, env_cdpath_val: None },
    ];

    for case in cases {
        // The macro handles the creation and trait attachment itself
        let mut cmd = cargo_bin_cmd!("ncd");

        if let Some(path) = case.env_cdpath_val {
            // String literals usually bypass spellcheck warnings in the linter
            cmd.env("CDPATH", path);
        }

        let assert = cmd.arg(case.input).assert();

        if case.should_succeed {
            assert.success().stdout(predicates::str::contains(case.expected_out));
        } else {
            assert.failure().stderr(predicates::str::contains(case.expected_out));
        }
    }
}