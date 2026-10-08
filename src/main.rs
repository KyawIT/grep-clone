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
pub fn search<'a>(text: &'a str, cfg: &Config) -> Vec<(usize, &'a str)> {
    let pat = if cfg.ignore_case { cfg.pattern.to_lowercase() } else { cfg.pattern.clone() };
    // split_terminator: "a\n" is one line, "" is none, and '\r' stays part of the line (like GNU grep).
    text.split_terminator('\n')
        .enumerate()
        .filter(|(_, line)| {
            let hit = if cfg.ignore_case { line.to_lowercase().contains(&pat) } else { line.contains(&pat) };
            hit != cfg.invert
        })
        .map(|(i, line)| (i + 1, line))
        .collect()
}

/// Format one output line: optional `path:` prefix, optional `N:` prefix, then the line.
pub fn format_match(path: Option<&str>, line_no: usize, line: &str, cfg: &Config) -> String {
    let p = path.map(|p| format!("{p}:")).unwrap_or_default();
    let n = if cfg.line_numbers { format!("{line_no}:") } else { String::new() };
    format!("{p}{n}{line}")
}

pub fn run(cfg: &Config, stdin: &mut impl Read, out: &mut impl Write, err: &mut impl Write) -> i32 {
    let dash = ["-".to_string()];
    let paths: &[String] = if cfg.paths.is_empty() { &dash } else { &cfg.paths };
    let multi = paths.len() > 1;
    let (mut selected, mut failed) = (false, false);

    for path in paths {
        let is_stdin = path == "-";
        let name = if is_stdin { "(standard input)" } else { path.as_str() };

        let mut bytes = Vec::new();
        let read = if is_stdin {
            stdin.read_to_end(&mut bytes).map(|_| ())
        } else {
            std::fs::read(path).map(|b| bytes = b)
        };
        if let Err(e) = read {
            let _ = writeln!(err, "grep_clone: {name}: {e}");
            failed = true;
            continue;
        }

        // Binary = NUL byte or invalid UTF-8; still searched (lossily), but lines are never printed.
        let binary = bytes.contains(&0) || std::str::from_utf8(&bytes).is_err();
        let text = String::from_utf8_lossy(&bytes);
        let hits = search(&text, cfg);
        selected |= !hits.is_empty();
        let shown = multi.then_some(name);

        // ponytail: write errors (e.g. closed pipe) are ignored; propagate if exit code must reflect them.
        if cfg.count {
            let prefix = shown.map(|n| format!("{n}:")).unwrap_or_default();
            let _ = writeln!(out, "{prefix}{}", hits.len());
        } else if binary {
            if !hits.is_empty() {
                let _ = writeln!(err, "grep_clone: {name}: binary file matches");
            }
        } else {
            for (n, line) in hits {
                let _ = writeln!(out, "{}", format_match(shown, n, line, cfg));
            }
        }
    }

    if failed { 2 } else if selected { 0 } else { 1 }
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
