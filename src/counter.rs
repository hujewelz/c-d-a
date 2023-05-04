use std::{fmt::Display, fs, path::Path};

use crate::runner;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lang {
    Swift,
    Java,
    Html,
    Kotlin,
    Rust,
}

impl Lang {
    pub fn extension(&self) -> &'static str {
        match *self {
            Lang::Swift => "swift",
            Lang::Java => "java",
            Lang::Html => "html",
            Lang::Kotlin => "kt",
            Lang::Rust => "rs",
        }
    }

    fn lang(&self) -> &'static str {
        match *self {
            Lang::Swift => "swift",
            Lang::Java => "java",
            Lang::Html => "html",
            Lang::Kotlin => "kotlin",
            Lang::Rust => "rust",
        }
    }
}

impl Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lang())
    }
}

#[derive(Debug, Clone)]
pub struct SourceCode {
    pub file: String,
    pub lines: usize,
}

impl SourceCode {
    pub fn new() -> Self {
        SourceCode {
            file: String::new(),
            lines: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.file.is_empty()
    }
}

#[derive(Debug)]
pub struct Scanner {
    sources: Vec<SourceCode>,
}

impl Scanner {
    pub fn scan<P: AsRef<Path>>(root_dir: P, includes: &[Lang]) -> Self {
        let sources = Self::read_dir(root_dir.as_ref(), includes);

        Scanner { sources }
    }

    pub fn source_codes(&self) -> &[SourceCode] {
        &self.sources
    }

    pub fn num_of_files(&self) -> usize {
        self.sources.len()
    }

    pub fn all_lines(&self) -> usize {
        self.sources
            .iter()
            .map(|s| s.lines)
            .reduce(|acc, e| acc + e)
            .unwrap_or_default()
    }

    pub fn pretty_printed(&self) {
        let max_width_of_file_path = self
            .sources
            .iter()
            .map(|s| s.file.len())
            .max()
            .unwrap_or_default();

        let mut result = String::from("\n");
        for s in self.sources.iter() {
            let a = max_width_of_file_path - s.file.len();
            result.push_str(&s.file);
            for _ in 0..a {
                result.push(' ');
            }
            result.push_str(&format!("\t{}\n", s.lines));
        }
        println!("{result}");
    }

    fn read_dir(path: &Path, includes: &[Lang]) -> Vec<SourceCode> {
        if path.is_file() {
            let ext = match path.extension() {
                Some(ext) => ext,
                None => return vec![],
            };

            let ext = match ext.to_str() {
                Some(ext) => ext,
                None => return vec![],
            };

            let lang_exts = includes
                .iter()
                .map(|l| l.extension())
                .collect::<Vec<&'static str>>();

            if !lang_exts.contains(&ext) {
                return vec![];
            }

            let lines = runner::count_lines(path, true);

            return match path.to_str() {
                Some(file_path) => vec![SourceCode {
                    file: file_path.to_owned(),
                    lines,
                }],
                None => vec![],
            };
        }

        if let Ok(red_dir) = fs::read_dir(path) {
            let mut files: Vec<SourceCode> = vec![];
            for entry in red_dir {
                match entry {
                    Ok(en) => files.append(&mut Self::read_dir(&en.path(), &includes)),
                    Err(_) => files.append(&mut vec![]),
                };
            }

            return files;
        } else {
            return vec![];
        }
    }
}

impl Display for SourceCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.file, self.lines)
    }
}

pub struct Counter {}

#[cfg(test)]
mod tests {
    #[test]
    fn lang() {
        assert!(format!("{}", super::Lang::Swift) == "swift");
    }
}
