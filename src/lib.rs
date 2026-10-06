use std::io::Write;

#[derive(Debug, Default, PartialEq)]
pub struct Config {
    pub pattern: String,
    pub paths: Vec<String>,
    pub ignore_case: bool,  // -i
    pub invert: bool,       // -v
    pub line_numbers: bool, // -n
    pub count: bool,        // -c
}

/// Parse CLI args (program name already stripped): `[-i -v -n -c]... PATTERN [FILE]...`
pub fn parse_args<I: IntoIterator<Item = String>>(_args: I) -> Result<Config, String> {
    todo!()
}

/// Return `(1-based line number, line)` for each selected line.
pub fn search<'a>(_text: &'a str, _cfg: &Config) -> Vec<(usize, &'a str)> {
    todo!()
}

/// Format one output line: optional `path:` prefix, optional `N:` prefix, then the line.
pub fn format_match(_path: Option<&str>, _line_no: usize, _line: &str, _cfg: &Config) -> String {
    todo!()
}

/// Search every path in `cfg` (stdin is out of scope), writing results to `out`.
/// Returns Ok(true) if anything was selected (exit code 0), Ok(false) otherwise.
/// A missing file is an Err.
pub fn run(_cfg: &Config, _out: &mut impl Write) -> std::io::Result<bool> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    fn cfg(pattern: &str) -> Config {
        Config { pattern: pattern.into(), ..Default::default() }
    }

    const TEXT: &str = "Hello World\nfoo bar\nhello again\nbaz\n";

    // ---- parse_args ----

    #[test]
    fn parse_pattern_and_file() {
        let c = parse_args(args(&["foo", "a.txt"])).unwrap();
        assert_eq!(c, Config { pattern: "foo".into(), paths: vec!["a.txt".into()], ..Default::default() });
    }

    #[test]
    fn parse_multiple_files() {
        let c = parse_args(args(&["foo", "a", "b"])).unwrap();
        assert_eq!(c.paths, vec!["a", "b"]);
    }

    #[test]
    fn parse_flags() {
        let c = parse_args(args(&["-i", "-v", "-n", "-c", "foo", "a"])).unwrap();
        assert!(c.ignore_case && c.invert && c.line_numbers && c.count);
    }

    #[test]
    fn parse_combined_flags() {
        let c = parse_args(args(&["-inv", "foo", "a"])).unwrap();
        assert!(c.ignore_case && c.invert && c.line_numbers && !c.count);
    }

    #[test]
    fn parse_flags_after_pattern() {
        let c = parse_args(args(&["foo", "-i", "a"])).unwrap();
        assert!(c.ignore_case);
        assert_eq!(c.pattern, "foo");
        assert_eq!(c.paths, vec!["a"]);
    }

    #[test]
    fn parse_no_args_is_error() {
        assert!(parse_args(args(&[])).is_err());
    }

    #[test]
    fn parse_flags_only_is_error() {
        assert!(parse_args(args(&["-i"])).is_err());
    }

    #[test]
    fn parse_unknown_flag_is_error() {
        assert!(parse_args(args(&["-z", "foo"])).is_err());
    }

    #[test]
    fn parse_double_dash_makes_pattern_literal() {
        let c = parse_args(args(&["--", "-v", "a"])).unwrap();
        assert_eq!(c.pattern, "-v");
        assert!(!c.invert);
    }

    // ---- search ----

    #[test]
    fn search_basic_match() {
        assert_eq!(search(TEXT, &cfg("foo")), vec![(2, "foo bar")]);
    }

    #[test]
    fn search_is_case_sensitive_by_default() {
        assert_eq!(search(TEXT, &cfg("hello")), vec![(3, "hello again")]);
    }

    #[test]
    fn search_ignore_case() {
        let c = Config { ignore_case: true, ..cfg("hello") };
        assert_eq!(search(TEXT, &c), vec![(1, "Hello World"), (3, "hello again")]);
    }

    #[test]
    fn search_ignore_case_pattern_uppercase() {
        let c = Config { ignore_case: true, ..cfg("HELLO") };
        assert_eq!(search(TEXT, &c).len(), 2);
    }

    #[test]
    fn search_no_match_is_empty() {
        assert!(search(TEXT, &cfg("zzz")).is_empty());
    }

    #[test]
    fn search_substring_inside_word() {
        assert_eq!(search(TEXT, &cfg("ba")), vec![(2, "foo bar"), (4, "baz")]);
    }

    #[test]
    fn search_invert() {
        let c = Config { invert: true, ..cfg("o") };
        assert_eq!(search(TEXT, &c), vec![(4, "baz")]);
    }

    #[test]
    fn search_invert_with_ignore_case() {
        let c = Config { invert: true, ignore_case: true, ..cfg("HELLO") };
        assert_eq!(search(TEXT, &c), vec![(2, "foo bar"), (4, "baz")]);
    }

    #[test]
    fn search_empty_text() {
        assert!(search("", &cfg("a")).is_empty());
    }

    #[test]
    fn search_empty_pattern_matches_every_line() {
        assert_eq!(search("a\n\nb", &cfg("")).len(), 3);
    }

    #[test]
    fn search_no_trailing_newline() {
        assert_eq!(search("a\nfoo", &cfg("foo")), vec![(2, "foo")]);
    }

    #[test]
    fn search_crlf_line_endings_are_stripped() {
        assert_eq!(search("foo\r\nbar\r\n", &cfg("foo")), vec![(1, "foo")]);
    }

    #[test]
    fn search_pattern_is_literal_not_regex() {
        assert_eq!(search("a.c\nabc", &cfg("a.c")), vec![(1, "a.c")]);
    }

    #[test]
    fn search_unicode_ignore_case() {
        let c = Config { ignore_case: true, ..cfg("é") };
        assert_eq!(search("CAFÉ\ncafe", &c), vec![(1, "CAFÉ")]);
    }

    #[test]
    fn search_match_once_per_line_even_if_repeated() {
        assert_eq!(search("aaa", &cfg("a")).len(), 1);
    }

    // ---- format_match ----

    #[test]
    fn format_plain() {
        assert_eq!(format_match(None, 3, "foo", &cfg("f")), "foo");
    }

    #[test]
    fn format_with_line_number() {
        let c = Config { line_numbers: true, ..cfg("f") };
        assert_eq!(format_match(None, 3, "foo", &c), "3:foo");
    }

    #[test]
    fn format_with_path() {
        assert_eq!(format_match(Some("a.txt"), 3, "foo", &cfg("f")), "a.txt:foo");
    }

    #[test]
    fn format_with_path_and_line_number() {
        let c = Config { line_numbers: true, ..cfg("f") };
        assert_eq!(format_match(Some("a.txt"), 3, "foo", &c), "a.txt:3:foo");
    }

    // ---- run (file I/O) ----

    fn tmp_file(name: &str, body: &str) -> String {
        let dir = std::env::temp_dir().join(format!("grep_clone_{}_{}", std::process::id(), name));
        std::fs::write(&dir, body).unwrap();
        dir.to_string_lossy().into_owned()
    }

    fn run_to_string(c: &Config) -> (std::io::Result<bool>, String) {
        let mut out = Vec::new();
        let r = run(c, &mut out);
        (r, String::from_utf8(out).unwrap())
    }

    #[test]
    fn run_single_file_has_no_path_prefix() {
        let p = tmp_file("single", TEXT);
        let c = Config { paths: vec![p], ..cfg("foo") };
        let (r, out) = run_to_string(&c);
        assert!(r.unwrap());
        assert_eq!(out, "foo bar\n");
    }

    #[test]
    fn run_multiple_files_prefix_paths() {
        let a = tmp_file("multi_a", "foo\n");
        let b = tmp_file("multi_b", "nope\nfoo\n");
        let c = Config { paths: vec![a.clone(), b.clone()], ..cfg("foo") };
        let (r, out) = run_to_string(&c);
        assert!(r.unwrap());
        assert_eq!(out, format!("{a}:foo\n{b}:foo\n"));
    }

    #[test]
    fn run_no_match_returns_false_and_prints_nothing() {
        let p = tmp_file("nomatch", TEXT);
        let c = Config { paths: vec![p], ..cfg("zzz") };
        let (r, out) = run_to_string(&c);
        assert!(!r.unwrap());
        assert_eq!(out, "");
    }

    #[test]
    fn run_count_single_file() {
        let p = tmp_file("count", TEXT);
        let c = Config { paths: vec![p], count: true, ..cfg("o") };
        let (r, out) = run_to_string(&c);
        assert!(r.unwrap());
        assert_eq!(out, "3\n");
    }

    #[test]
    fn run_count_zero_prints_zero_and_returns_false() {
        let p = tmp_file("count0", TEXT);
        let c = Config { paths: vec![p], count: true, ..cfg("zzz") };
        let (r, out) = run_to_string(&c);
        assert!(!r.unwrap());
        assert_eq!(out, "0\n");
    }

    #[test]
    fn run_count_multiple_files_prefixes_paths() {
        let a = tmp_file("countm_a", "x\nx\n");
        let b = tmp_file("countm_b", "y\n");
        let c = Config { paths: vec![a.clone(), b.clone()], count: true, ..cfg("x") };
        let (_, out) = run_to_string(&c);
        assert_eq!(out, format!("{a}:2\n{b}:0\n"));
    }

    #[test]
    fn run_line_numbers() {
        let p = tmp_file("ln", TEXT);
        let c = Config { paths: vec![p], line_numbers: true, ..cfg("baz") };
        let (_, out) = run_to_string(&c);
        assert_eq!(out, "4:baz\n");
    }

    #[test]
    fn run_missing_file_is_error() {
        let c = Config { paths: vec!["/definitely/not/here".into()], ..cfg("x") };
        let (r, _) = run_to_string(&c);
        assert!(r.is_err());
    }

    #[test]
    fn run_non_utf8_file_is_error_not_panic() {
        let dir = std::env::temp_dir().join(format!("grep_clone_{}_bin", std::process::id()));
        std::fs::write(&dir, [0xff, 0xfe, 0x00]).unwrap();
        let c = Config { paths: vec![dir.to_string_lossy().into_owned()], ..cfg("x") };
        let (r, _) = run_to_string(&c);
        assert!(r.is_err());
    }

    #[test]
    fn run_empty_file() {
        let p = tmp_file("empty", "");
        let c = Config { paths: vec![p], ..cfg("x") };
        let (r, out) = run_to_string(&c);
        assert!(!r.unwrap());
        assert_eq!(out, "");
    }
}
