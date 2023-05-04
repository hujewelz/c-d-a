use regex::Regex;

pub trait PmdAnalyzer {
    type Result;
    fn analyze(&self, source: &str) -> Self::Result;
}

pub struct LinesAnalyzer;

impl LinesAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl PmdAnalyzer for LinesAnalyzer {
    type Result = Option<usize>;

    fn analyze(&self, source: &str) -> Self::Result {
        let reg = Regex::new(r"^Found a ([1-9]\d*) line").unwrap();

        for cap in reg.captures_iter(source) {
            if cap.len() > 1 {
                return Some(cap[1].parse::<usize>().unwrap_or_default());
            }
        }
        None
    }
}

pub struct DuplicationAnalyzer;

impl DuplicationAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl PmdAnalyzer for DuplicationAnalyzer {
    type Result = Option<(u32, String)>;

    fn analyze(&self, source: &str) -> Self::Result {
        let reg = Regex::new(r"Starting at line ([1-9]\d*) of (/.+)?").unwrap();

        for cap in reg.captures_iter(source) {
            if cap.len() > 2 {
                let start = cap[1].parse::<u32>().unwrap_or_default();
                let file = cap[2].to_owned();
                return Some((start, file));
            }
        }
        None
    }
}
