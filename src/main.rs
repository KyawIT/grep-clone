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

fn main() {
    let cfg = match parse_args(std::env::args().skip(1)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("grep_clone: {e}");
            std::process::exit(2);
        }
    };
    match run(&cfg, &mut std::io::stdout().lock()) {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(e) => {
            eprintln!("grep_clone: {e}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests;
