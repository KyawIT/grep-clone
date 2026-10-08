use std::io::{Read, Write};

#[derive(Debug, Default, PartialEq)]
pub struct Config {
    pub pattern: String,
    pub paths: Vec<String>,
    pub ignore_case: bool,  // -i
    pub invert: bool,       // -v
    pub line_numbers: bool, // -n
    pub count: bool,        // -c
}

impl Config {
    // Short flags
    const IGNORE_CASE:char = 'i';
    const INVERT:char = 'v';
    const LINE_NUMBERS:char = 'n';
    const COUNT:char = 'c';
    const FLAGS: [char; 4] = [Self::IGNORE_CASE, Self::INVERT, Self::LINE_NUMBERS, Self::COUNT];
    // Long flags
    const IGNORE_CASE_LONG:&str = "ignore-case";
    const INVERT_LONG:&str = "invert-match";
    const LINE_NUMBERS_LONG:&str = "line-number";
    const COUNT_LONG:&str = "count";

    pub fn new() -> Config{
        Config { pattern: "".to_string(), paths: vec![], ignore_case: false, invert: false, line_numbers: false, count: false }
    }

    /// Return a String Vector containing only the path, no prefix or flags
    pub fn validate_path(val:&String, prefix:&String) -> bool{
        if !val.contains(prefix) && !Self::validate_flags(val, None){
            return true;
        }
        false
    }


    
    /// Returns true if one of the given flag is included
    /// Returns false if none of the given flag is included
    pub fn validate_flags(val: &str, config: Option<&mut Config>) -> bool{
        let mut default_config:Config = Config::new();
        let config_ref = config.unwrap_or(&mut default_config);


        if let Some(name) = val.strip_prefix("--") {
            match name {
                Self::IGNORE_CASE_LONG => config_ref.ignore_case = true,
                Self::INVERT_LONG => config_ref.invert = true,
                Self::LINE_NUMBERS_LONG => config_ref.line_numbers = true,
                Self::COUNT_LONG => config_ref.count = true,
                _ => return false,
            }
            return true;
        }

        let Some(rest) = val.strip_prefix('-') else { return false };
        if rest.is_empty() || !rest.chars().all(|c| Self::FLAGS.contains(&c)) {
            return false;
        }
        for c in rest.chars() {
            match c {
                Self::IGNORE_CASE => config_ref.ignore_case = true,
                Self::INVERT => config_ref.invert = true,
                Self::LINE_NUMBERS => config_ref.line_numbers = true,
                Self::COUNT => config_ref.count = true,
                _ => unreachable!(),
            }
        }



        true
    }

}

/// Parse CLI args (program name already stripped): `[-i -v -n -c]... PATTERN [FILE]...`
pub fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Config, String> {
    let mut confg:Config = Config::new();
    let mut first: bool = false;
    let mut options_done: bool = false;


    for i in args {
        if !options_done {
            if i == "--" {
                options_done = true;
                continue;
            }
            if Config::validate_flags(&i, Some(&mut confg)) {
                continue;
            }
            if i.starts_with('-') && i != "-" {
                return Err(format!("invalid option '{i}'"));
            }
        }

        if !first {
            confg.pattern = i;
            first = true;
        } else {
            confg.paths.push(i);
        }
    }

    if !first {
        return Err("missing pattern".to_string());
    }
    Ok(confg)
}

/// Return `(1-based line number, line)` for each selected line.
pub fn search<'a>(_text: &'a str, _cfg: &Config) -> Vec<(usize, &'a str)> {
    todo!()
}

/// Format one output line: optional `path:` prefix, optional `N:` prefix, then the line.
pub fn format_match(_path: Option<&str>, _line_no: usize, _line: &str, _cfg: &Config) -> String {
    todo!()
}

/// Search every path in `cfg` like GNU grep, returning the exit code:
/// 0 = something selected, 1 = nothing selected, 2 = an error happened (even if something matched).
/// - No paths, or the path `-`, means read `stdin` (shown as `(standard input)` in prefixes).
/// - Matches go to `out`; errors and "binary file matches" notices go to `err`, prefixed `grep_clone: `.
/// - A file that can't be read is reported to `err` and skipped; the other files are still searched.
pub fn run(_cfg: &Config, _stdin: &mut impl Read, _out: &mut impl Write, _err: &mut impl Write) -> i32 {
    todo!()
}

fn main() {
    let cfg = match parse_args(std::env::args().skip(1)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("grep_clone: {e}");
            std::process::exit(2);
        }
    };
    let code = run(
        &cfg,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    );
    std::process::exit(code);
}

#[cfg(test)]
mod tests;
