// Expected behaviour in these tests was checked against GNU grep 3.11 (`/usr/bin/grep`).
// The pattern is a fixed string, not a regex — i.e. this clone behaves like `grep -F`.
use super::*;

fn args(a: &[&str]) -> Vec<String> {
    a.iter().map(|s: &&str| s.to_string()).collect()
}

fn cfg(pattern: &str) -> Config {
    Config { pattern: pattern.into(), ..Default::default() }
}

const TEXT: &str = "Hello World\nfoo bar\nhello again\nbaz\n";

// ---- parse_args ----

#[test]
fn parse_pattern_and_file() {
    let c: Config = parse_args(args(&["foo", "a.txt"])).unwrap();
    assert_eq!(c, Config { pattern: "foo".into(), paths: vec!["a.txt".into()], ..Default::default() });
}

#[test]
fn parse_pattern_only_means_stdin() {
    let c = parse_args(args(&["foo"])).unwrap();
    assert_eq!(c.pattern, "foo");
    assert!(c.paths.is_empty());
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
    assert_eq!(c.pattern, "foo");
    assert_eq!(c.paths, vec!["a"]);
}

#[test]
fn parse_no_flags_means_all_false() {
    let c = parse_args(args(&["foo", "a"])).unwrap();
    assert!(!c.ignore_case && !c.invert && !c.line_numbers && !c.count);
}

#[test]
fn parse_combined_flags() {
    let c = parse_args(args(&["-inv", "foo", "a"])).unwrap();
    assert!(c.ignore_case && c.invert && c.line_numbers && !c.count);
}

#[test]
fn parse_repeated_flag_is_fine() {
    let c = parse_args(args(&["-i", "-i", "foo"])).unwrap();
    assert!(c.ignore_case);
}

#[test]
fn parse_long_flags() {
    let c = parse_args(args(&["--ignore-case", "--invert-match", "--line-number", "--count", "foo"])).unwrap();
    assert!(c.ignore_case && c.invert && c.line_numbers && c.count);
    assert_eq!(c.pattern, "foo");
}

#[test]
fn parse_flags_after_pattern() {
    let c = parse_args(args(&["foo", "-i", "a"])).unwrap();
    assert!(c.ignore_case);
    assert_eq!(c.pattern, "foo");
    assert_eq!(c.paths, vec!["a"]);
}

#[test]
fn parse_flags_after_file() {
    let c = parse_args(args(&["foo", "a", "-n"])).unwrap();
    assert!(c.line_numbers);
    assert_eq!(c.paths, vec!["a"]);
}

#[test]
fn parse_dash_inside_word_is_not_a_flag() {
    // "re-cover" contains "-c" and "my-notes.txt" contains "-n", but neither starts with '-'.
    let c = parse_args(args(&["re-cover", "my-notes.txt"])).unwrap();
    assert_eq!(c.pattern, "re-cover");
    assert_eq!(c.paths, vec!["my-notes.txt"]);
    assert!(!c.count && !c.line_numbers);
}

#[test]
fn parse_empty_string_pattern() {
    let c = parse_args(args(&["", "a"])).unwrap();
    assert_eq!(c.pattern, "");
    assert_eq!(c.paths, vec!["a"]);
}

#[test]
fn parse_lone_dash_is_a_pattern_not_a_flag() {
    let c = parse_args(args(&["-", "a"])).unwrap();
    assert_eq!(c.pattern, "-");
}

#[test]
fn parse_lone_dash_is_a_path_meaning_stdin() {
    let c = parse_args(args(&["foo", "-"])).unwrap();
    assert_eq!(c.paths, vec!["-"]);
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
fn parse_double_dash_only_is_error() {
    assert!(parse_args(args(&["--"])).is_err());
}

#[test]
fn parse_unknown_flag_is_error() {
    assert!(parse_args(args(&["-z", "foo"])).is_err());
}

#[test]
fn parse_unknown_letter_in_combined_flags_is_error() {
    assert!(parse_args(args(&["-iz", "foo"])).is_err());
}

#[test]
fn parse_unknown_long_flag_is_error() {
    assert!(parse_args(args(&["--bogus", "foo"])).is_err());
}

#[test]
fn parse_double_dash_makes_pattern_literal() {
    let c = parse_args(args(&["--", "-v", "a"])).unwrap();
    assert_eq!(c.pattern, "-v");
    assert!(!c.invert);
}

#[test]
fn parse_double_dash_after_pattern_makes_paths_literal() {
    let c = parse_args(args(&["foo", "a", "--", "-i"])).unwrap();
    assert_eq!(c.paths, vec!["a", "-i"]);
    assert!(!c.ignore_case);
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
fn search_ignore_case_returns_original_line_not_lowercased() {
    let c = Config { ignore_case: true, ..cfg("WORLD") };
    assert_eq!(search(TEXT, &c), vec![(1, "Hello World")]);
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
fn search_pattern_at_start_and_end_of_line() {
    assert_eq!(search("foo x\nx foo\nxfoox", &cfg("foo")).len(), 3);
}

#[test]
fn search_pattern_equal_to_whole_line() {
    assert_eq!(search("foo", &cfg("foo")), vec![(1, "foo")]);
}

#[test]
fn search_pattern_longer_than_line() {
    assert!(search("ab", &cfg("abc")).is_empty());
}

#[test]
fn search_match_does_not_span_lines() {
    assert!(search("ab\ncd", &cfg("bc")).is_empty());
}

#[test]
fn search_whitespace_pattern() {
    assert_eq!(search("  \n\t\nx\n", &cfg(" ")), vec![(1, "  ")]);
}

#[test]
fn search_invert() {
    let c = Config { invert: true, ..cfg("o") };
    assert_eq!(search(TEXT, &c), vec![(4, "baz")]);
}

#[test]
fn search_invert_keeps_original_line_numbers() {
    let c = Config { invert: true, ..cfg("foo") };
    assert_eq!(search("foo\nbar\nfoo\nbaz", &c), vec![(2, "bar"), (4, "baz")]);
}

#[test]
fn search_invert_with_ignore_case() {
    let c = Config { invert: true, ignore_case: true, ..cfg("HELLO") };
    assert_eq!(search(TEXT, &c), vec![(2, "foo bar"), (4, "baz")]);
}

#[test]
fn search_invert_everything_matches_gives_nothing() {
    let c = Config { invert: true, ..cfg("") };
    assert!(search("a\nb\n", &c).is_empty());
}

#[test]
fn search_empty_text() {
    assert!(search("", &cfg("a")).is_empty());
}

#[test]
fn search_empty_text_empty_pattern_has_no_lines() {
    assert!(search("", &cfg("")).is_empty());
}

#[test]
fn search_empty_pattern_matches_every_line() {
    assert_eq!(search("a\n\nb", &cfg("")).len(), 3);
}

#[test]
fn search_trailing_newline_does_not_add_an_extra_line() {
    assert_eq!(search("a\n", &cfg("")), vec![(1, "a")]);
}

#[test]
fn search_only_newlines_are_empty_lines() {
    assert_eq!(search("\n", &cfg("")), vec![(1, "")]);
    assert_eq!(search("\n\n", &cfg("")), vec![(1, ""), (2, "")]);
}

#[test]
fn search_line_numbers_count_empty_lines() {
    assert_eq!(search("\n\nfoo\n", &cfg("foo")), vec![(3, "foo")]);
}

#[test]
fn search_no_trailing_newline() {
    assert_eq!(search("a\nfoo", &cfg("foo")), vec![(2, "foo")]);
}

#[test]
fn search_crlf_keeps_carriage_return_like_real_grep() {
    // GNU grep only splits on '\n'; the '\r' stays part of the line.
    assert_eq!(search("foo\r\nbar\r\n", &cfg("foo")), vec![(1, "foo\r")]);
}

#[test]
fn search_lone_carriage_return_is_not_a_line_break() {
    assert_eq!(search("a\rfoo\n", &cfg("foo")), vec![(1, "a\rfoo")]);
}

#[test]
fn search_pattern_is_literal_not_regex() {
    assert_eq!(search("a.c\nabc", &cfg("a.c")), vec![(1, "a.c")]);
    assert_eq!(search("abc\naac\na*c", &cfg("a*c")), vec![(3, "a*c")]);
    assert_eq!(search("x[y\nxy", &cfg("[")), vec![(1, "x[y")]);
    assert_eq!(search("a\\b\nab", &cfg("\\")), vec![(1, "a\\b")]);
}

#[test]
fn search_unicode_ignore_case() {
    let c = Config { ignore_case: true, ..cfg("é") };
    assert_eq!(search("CAFÉ\ncafe", &c), vec![(1, "CAFÉ")]);
}

#[test]
fn search_unicode_ignore_case_not_ascii_only() {
    // Fails if you use to_ascii_lowercase instead of to_lowercase.
    let c = Config { ignore_case: true, ..cfg("ПРИВЕТ") };
    assert_eq!(search("привет мир\nhello", &c), vec![(1, "привет мир")]);
}

#[test]
fn search_unicode_ignore_case_greek_final_sigma() {
    let c = Config { ignore_case: true, ..cfg("σας") };
    assert_eq!(search("ΣΑΣ\nσας\n", &c).len(), 2);
}

#[test]
fn search_ignore_case_sharp_s_does_not_match_ss() {
    let c = Config { ignore_case: true, ..cfg("ß") };
    assert_eq!(search("STRASSE\nstraße\n", &c), vec![(2, "straße")]);
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

#[test]
fn format_empty_line_with_line_number() {
    let c = Config { line_numbers: true, ..cfg("") };
    assert_eq!(format_match(None, 7, "", &c), "7:");
}

#[test]
fn format_line_containing_colons_is_untouched() {
    let c = Config { line_numbers: true, ..cfg("") };
    assert_eq!(format_match(Some("a"), 1, "x:y:z", &c), "a:1:x:y:z");
}

// ---- run (files, stdin, exit codes) ----

fn tmp_file(name: &str, body: impl AsRef<[u8]>) -> String {
    let path = std::env::temp_dir().join(format!("grep_clone_{}_{}", std::process::id(), name));
    std::fs::write(&path, body).unwrap();
    path.to_string_lossy().into_owned()
}

/// Runs `run` with `stdin` as standard input. Returns (exit code, stdout, stderr).
fn run_with(c: &Config, stdin: &[u8]) -> (i32, String, String) {
    let (mut input, mut out, mut err) = (stdin, Vec::new(), Vec::new());
    let code = run(c, &mut input, &mut out, &mut err);
    (code, String::from_utf8(out).unwrap(), String::from_utf8(err).unwrap())
}

fn files(c: Config, paths: &[&String]) -> Config {
    Config { paths: paths.iter().map(|p| p.to_string()).collect(), ..c }
}

#[test]
fn run_single_file_has_no_path_prefix() {
    let p = tmp_file("single", TEXT);
    assert_eq!(run_with(&files(cfg("foo"), &[&p]), b""), (0, "foo bar\n".into(), "".into()));
}

#[test]
fn run_multiple_files_prefix_paths() {
    let a = tmp_file("multi_a", "foo\n");
    let b = tmp_file("multi_b", "nope\nfoo\n");
    let (code, out, _) = run_with(&files(cfg("foo"), &[&a, &b]), b"");
    assert_eq!(code, 0);
    assert_eq!(out, format!("{a}:foo\n{b}:foo\n"));
}

#[test]
fn run_exit_zero_if_any_file_matches() {
    let a = tmp_file("any_a", "nope\n");
    let b = tmp_file("any_b", "foo\n");
    let (code, out, _) = run_with(&files(cfg("foo"), &[&a, &b]), b"");
    assert_eq!(code, 0);
    assert_eq!(out, format!("{b}:foo\n"));
}

#[test]
fn run_same_file_twice_is_searched_twice() {
    let a = tmp_file("twice", "foo\n");
    let (_, out, _) = run_with(&files(cfg("foo"), &[&a, &a]), b"");
    assert_eq!(out, format!("{a}:foo\n{a}:foo\n"));
}

#[test]
fn run_no_match_exits_one_and_prints_nothing() {
    let p = tmp_file("nomatch", TEXT);
    assert_eq!(run_with(&files(cfg("zzz"), &[&p]), b""), (1, "".into(), "".into()));
}

#[test]
fn run_adds_newline_when_file_has_none() {
    let p = tmp_file("nonl", "a\nfoo");
    assert_eq!(run_with(&files(cfg("foo"), &[&p]), b"").1, "foo\n");
}

#[test]
fn run_crlf_output_keeps_carriage_return() {
    let p = tmp_file("crlf", "foo\r\nbar\r\n");
    assert_eq!(run_with(&files(cfg("foo"), &[&p]), b"").1, "foo\r\n");
}

#[test]
fn run_empty_file() {
    let p = tmp_file("empty", "");
    assert_eq!(run_with(&files(cfg("x"), &[&p]), b""), (1, "".into(), "".into()));
}

#[test]
fn run_empty_pattern_on_empty_file_matches_nothing() {
    let p = tmp_file("empty_pat_empty", "");
    assert_eq!(run_with(&files(cfg(""), &[&p]), b"").0, 1);
}

#[test]
fn run_empty_pattern_on_single_newline_prints_empty_line() {
    let p = tmp_file("empty_pat_nl", "\n");
    assert_eq!(run_with(&files(cfg(""), &[&p]), b""), (0, "\n".into(), "".into()));
}

#[test]
fn run_invert_empty_pattern_selects_nothing() {
    let p = tmp_file("inv_empty", "x\n");
    let c = Config { invert: true, ..cfg("") };
    assert_eq!(run_with(&files(c, &[&p]), b"").0, 1);
}

#[test]
fn run_line_numbers() {
    let p = tmp_file("ln", TEXT);
    let c = Config { line_numbers: true, ..cfg("baz") };
    assert_eq!(run_with(&files(c, &[&p]), b"").1, "4:baz\n");
}

#[test]
fn run_line_numbers_restart_for_each_file() {
    let a = tmp_file("lnr_a", "foo\n");
    let b = tmp_file("lnr_b", "x\nfoo\n");
    let c = Config { line_numbers: true, ..cfg("foo") };
    assert_eq!(run_with(&files(c, &[&a, &b]), b"").1, format!("{a}:1:foo\n{b}:2:foo\n"));
}

#[test]
fn run_count_single_file() {
    let p = tmp_file("count", TEXT);
    let c = Config { count: true, ..cfg("o") };
    assert_eq!(run_with(&files(c, &[&p]), b""), (0, "3\n".into(), "".into()));
}

#[test]
fn run_count_zero_prints_zero_and_exits_one() {
    let p = tmp_file("count0", TEXT);
    let c = Config { count: true, ..cfg("zzz") };
    assert_eq!(run_with(&files(c, &[&p]), b""), (1, "0\n".into(), "".into()));
}

#[test]
fn run_count_empty_file_prints_zero() {
    let p = tmp_file("count_empty", "");
    let c = Config { count: true, ..cfg("") };
    assert_eq!(run_with(&files(c, &[&p]), b""), (1, "0\n".into(), "".into()));
}

#[test]
fn run_count_multiple_files_prefixes_paths() {
    let a = tmp_file("countm_a", "x\nx\n");
    let b = tmp_file("countm_b", "y\n");
    let c = Config { count: true, ..cfg("x") };
    assert_eq!(run_with(&files(c, &[&a, &b]), b"").1, format!("{a}:2\n{b}:0\n"));
}

#[test]
fn run_count_with_invert_counts_non_matching_lines() {
    let a = tmp_file("countv_a", "foo\n");
    let b = tmp_file("countv_b", "a\nfoo");
    let c = Config { count: true, invert: true, ..cfg("foo") };
    assert_eq!(run_with(&files(c, &[&a, &b]), b""), (0, format!("{a}:0\n{b}:1\n"), "".into()));
}

#[test]
fn run_count_ignores_line_numbers() {
    let p = tmp_file("count_n", "foo\n");
    let c = Config { count: true, line_numbers: true, ..cfg("foo") };
    assert_eq!(run_with(&files(c, &[&p]), b"").1, "1\n");
}

// -- stdin --

#[test]
fn run_no_paths_reads_stdin() {
    assert_eq!(run_with(&cfg("foo"), b"foo\nbar\n"), (0, "foo\n".into(), "".into()));
}

#[test]
fn run_dash_path_reads_stdin() {
    let c = files(cfg("foo"), &[&"-".to_string()]);
    assert_eq!(run_with(&c, b"foo\nbar\n"), (0, "foo\n".into(), "".into()));
}

#[test]
fn run_stdin_mixed_with_files_is_called_standard_input() {
    let a = tmp_file("stdin_mix", "foo\n");
    let c = files(cfg("foo"), &[&"-".to_string(), &a]);
    assert_eq!(run_with(&c, b"foo\n").1, format!("(standard input):foo\n{a}:foo\n"));
}

#[test]
fn run_count_stdin_mixed_with_files() {
    let a = tmp_file("stdin_mix_c", "foo\n");
    let c = Config { count: true, ..files(cfg("foo"), &[&"-".to_string(), &a]) };
    assert_eq!(run_with(&c, b"x\n").1, format!("(standard input):0\n{a}:1\n"));
}

#[test]
fn run_empty_stdin_exits_one() {
    assert_eq!(run_with(&cfg("foo"), b""), (1, "".into(), "".into()));
}

// -- errors: report, keep going, exit 2 --

#[test]
fn run_missing_file_exits_two_and_reports_on_stderr() {
    let c = files(cfg("x"), &[&"/definitely/not/here".to_string()]);
    let (code, out, err) = run_with(&c, b"");
    assert_eq!(code, 2);
    assert_eq!(out, "");
    assert!(err.starts_with("grep_clone: /definitely/not/here: "), "stderr was: {err:?}");
}

#[test]
fn run_missing_file_does_not_stop_other_files() {
    let a = tmp_file("miss_a", "foo\n");
    let missing = "/definitely/not/here".to_string();
    let (code, out, err) = run_with(&files(cfg("foo"), &[&a, &missing]), b"");
    assert_eq!(code, 2, "an error wins over a match");
    assert_eq!(out, format!("{a}:foo\n"));
    assert!(err.contains(&missing));
}

#[test]
fn run_missing_file_first_still_searches_the_rest() {
    let a = tmp_file("miss_first", "foo\n");
    let missing = "/definitely/not/here".to_string();
    let (code, out, _) = run_with(&files(cfg("foo"), &[&missing, &a]), b"");
    assert_eq!(code, 2);
    assert_eq!(out, format!("{a}:foo\n"));
}

#[test]
fn run_count_with_missing_file() {
    let a = tmp_file("miss_count", "foo\n");
    let missing = "/definitely/not/here".to_string();
    let c = Config { count: true, ..files(cfg("foo"), &[&a, &missing]) };
    let (code, out, _) = run_with(&c, b"");
    assert_eq!(code, 2);
    assert_eq!(out, format!("{a}:1\n"));
}

#[test]
fn run_directory_is_an_error() {
    let dir = std::env::temp_dir().to_string_lossy().into_owned();
    let (code, out, err) = run_with(&files(cfg("foo"), &[&dir]), b"");
    assert_eq!(code, 2);
    assert_eq!(out, "");
    assert!(err.contains(&dir));
}

// -- binary files: contain a NUL byte, or are not valid UTF-8 --

#[test]
fn run_binary_file_match_prints_notice_not_the_line() {
    let p = tmp_file("bin_match", b"foo\0bar\n");
    let (code, out, err) = run_with(&files(cfg("foo"), &[&p]), b"");
    assert_eq!(code, 0);
    assert_eq!(out, "");
    assert_eq!(err, format!("grep_clone: {p}: binary file matches\n"));
}

#[test]
fn run_binary_file_no_match_is_silent() {
    let p = tmp_file("bin_nomatch", b"abc\0def\n");
    assert_eq!(run_with(&files(cfg("foo"), &[&p]), b""), (1, "".into(), "".into()));
}

#[test]
fn run_binary_file_with_count_prints_count() {
    let p = tmp_file("bin_count", b"foo\0\nfoo\n");
    let c = Config { count: true, ..cfg("foo") };
    assert_eq!(run_with(&files(c, &[&p]), b""), (0, "2\n".into(), "".into()));
}

#[test]
fn run_non_utf8_file_is_treated_as_binary_not_an_error() {
    let p = tmp_file("latin1", b"x\xff\xfe foo\n");
    let (code, out, err) = run_with(&files(cfg("foo"), &[&p]), b"");
    assert_eq!(code, 0);
    assert_eq!(out, "");
    assert_eq!(err, format!("grep_clone: {p}: binary file matches\n"));
}

#[test]
fn run_binary_stdin_uses_standard_input_name() {
    let (code, _, err) = run_with(&cfg("foo"), b"foo\0\n");
    assert_eq!(code, 0);
    assert_eq!(err, "grep_clone: (standard input): binary file matches\n");
}

#[test]
fn run_binary_and_text_files_together() {
    let bin = tmp_file("mix_bin", b"foo\0\n");
    let txt = tmp_file("mix_txt", "foo\n");
    let (code, out, err) = run_with(&files(cfg("foo"), &[&bin, &txt]), b"");
    assert_eq!(code, 0);
    assert_eq!(out, format!("{txt}:foo\n"));
    assert_eq!(err, format!("grep_clone: {bin}: binary file matches\n"));
}
