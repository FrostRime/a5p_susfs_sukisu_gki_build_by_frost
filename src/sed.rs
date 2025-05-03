use regex::Regex;
use std::io::BufRead;


fn sed_replace(text: &str, pattern: &str, replacement: &str) -> String {
    Regex::new(pattern).unwrap().replace_all(text, replacement).into_owned()
}

fn process_stream<F: Fn(&str) -> String>(input: impl BufRead, processor: F) {
    for line in input.lines() {
        let line = line.unwrap();
        println!("{}", processor(&line));
    }
}

enum Address {
    All,
    LineNumber(usize),
    Pattern(String)
}

fn match_address(line: &str, addr: &Address, line_num: usize) -> bool {
    match addr {
        Address::All => true,
        Address::LineNumber(n) => *n == line_num,
        Address::Pattern(p) => Regex::new(p).unwrap().is_match(line)
    }
}