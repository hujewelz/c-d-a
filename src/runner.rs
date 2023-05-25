use clap::Parser;
use regex::Regex;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    analyzer::{DuplicationAnalyzer, LinesAnalyzer, PmdAnalyzer},
    counter::{Lang, Scanner, SourceCode},
};

#[derive(Parser, Debug)]
pub struct Args {
    /// Path to a source file, or directory containing source files to analyze. Zip and Jar files are also supported
    #[arg(short, long)]
    root: String,

    /// The source code directory which to compare.
    #[arg(short, long)]
    source: String,

    /// The source code directory for which compare to.
    #[arg(short, long)]
    destination: String,

    /// The source code language.
    #[arg(short, long, default_value_t = String::from("swift"))]
    language: String,

    /// The minimum token length which should be reported as a duplicate.
    #[arg(long, default_value_t = 50)]
    minimum_tokens: usize,
}

impl Args {
    fn is_destination_soruce_file(&self, file: &str) -> bool {
        let dest_file_name = Path::new(&self.destination)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let reg = Regex::new(&format!("/{}/", dest_file_name)).unwrap();
        reg.is_match(&file)
    }
}

#[derive(Debug, Clone)]
pub struct Duplication {
    pub lines: usize,
    pub source: SourceCode,
    pub destination: Vec<SourceCode>,
}

impl Duplication {
    fn new(lines: usize) -> Self {
        Self {
            lines: lines,
            source: SourceCode::new(),
            destination: Vec::new(),
        }
    }
    fn add_destination(&mut self, des: SourceCode) {
        self.destination.push(des);
    }

    fn clear_destination(&mut self) {
        self.destination.clear();
    }

    fn add_lines(&mut self, line: usize) {
        self.lines += line;
    }

    fn dup_rate(&self) -> f32 {
        if self.destination.is_empty() {
            return 0f32;
        }

        let rate = self.lines as f32 / self.destination[0].lines as f32;
        if rate > 1.0 {
            1.0
        } else {
            rate
        }
    }

    fn rate_of_source_code(&self) -> f32 {
        let rate = self.lines as f32 / self.source.lines as f32;
        if rate > 1.0 {
            1.0
        } else {
            rate
        }
    }
}

pub struct Runner;

impl Runner {
    pub fn run() -> Result<(), &'static str> {
        let args = Args::parse();

        let output = Command::new("which")
            .arg("pmd")
            .output()
            .expect("failed to execute process");

        if !output.status.success() {
            Self::install_pmd()?;
        }
        Self::exec_cpd(&args)
    }

    fn install_pmd() -> Result<(), &'static str> {
        println!("installing pmd...");

        let err_msg = "Install 'pmd' failed, please install it manually. See: https://docs.pmd-code.org/latest/pmd_userdocs_installation.html";

        let output = Command::new("brew")
            .arg("install")
            .arg("pmd")
            .output()
            .map_err(|_| err_msg)?;

        if !output.status.success() {
            Err(err_msg)
        } else {
            Ok(())
        }
    }

    fn exec_cpd(args: &Args) -> Result<(), &'static str> {
        let root_dir = &args.root;
        let minimum_tokens = format!("{}", args.minimum_tokens);
        let output = Command::new("pmd")
            .arg("cpd")
            .arg("--minimum-tokens")
            .arg(&minimum_tokens)
            .arg("-d")
            .arg(root_dir)
            .arg("--language")
            .arg(&args.language)
            .output()
            .expect("failed to execute process");

        let code = output.status.code().unwrap_or_default();

        if code == 0 {
            println!("Everything is fine, no code duplications found.");
            Ok(())
        } else if code == 4 {
            let mut path = PathBuf::from(root_dir);
            path.push("report.txt");

            let mut report_file = File::create(&path).unwrap();
            report_file.write_all(&output.stdout).unwrap();
            Self::analyze(&path)
        } else {
            Err("exited with an exception")
        }
    }

    fn analyze<P>(pmd_report: P) -> Result<(), &'static str>
    where
        P: AsRef<Path>,
    {
        let args = Args::parse();

        let lines_ana = LinesAnalyzer::new();
        let file_ana = DuplicationAnalyzer::new();

        let mut is_new_group = false;

        let mut result = Vec::new();

        let mut dup = Duplication::new(0);

        let file = File::open(pmd_report).map_err(|_| "Cannot open pmd report")?;
        let lines = BufReader::new(file).lines();

        for line in lines {
            if line.is_err() {
                continue;
            }

            let string = &line.unwrap();

            if string == "" && is_new_group {
                if !dup.source.is_empty() {
                    result.push(dup.clone());
                }
                dup = Duplication::new(0);

                is_new_group = false;
                continue;
            }

            if let Some(value) = lines_ana.analyze(string) {
                is_new_group = true;
                dup.lines = value;
                continue;
            }

            let dup_file = match file_ana.analyze(string) {
                Some(value) => value.1,
                None => continue,
            };

            let is_source = &dup_file.contains(&args.source);
            let is_dest = args.is_destination_soruce_file(&dup_file);
            // println!("file: {}, is destination {}", &dup_file, is_dest);

            let lines = count_lines(&dup_file, true);
            let sc = SourceCode {
                file: dup_file,
                lines,
            };

            if *is_source && dup.source.is_empty() {
                dup.source = sc;
            } else if is_dest {
                dup.add_destination(sc);
            } else {
                dup.clear_destination();
            }
        }

        Self::pretty_printed(&result, &args);

        Ok(())
    }

    fn pretty_printed(dups: &[Duplication], args: &Args) {
        let mut map: HashMap<&String, Duplication> = HashMap::new();

        let mut max_width_of_source_file_name = 0;
        let mut max_width_of_dest_file_name: usize = 0;
        for dup in dups {
            if dup.destination.is_empty() {
                continue;
            }

            map.entry(&dup.source.file)
                .and_modify(|d| d.add_lines(dup.lines))
                .or_insert(dup.clone());

            if dup.source.file.len() > max_width_of_source_file_name {
                max_width_of_source_file_name = dup.source.file.len();
            }

            if dup.destination[0].file.len() > max_width_of_dest_file_name {
                max_width_of_dest_file_name = dup.destination[0].file.len();
            }
        }

        println!("Found {} results:", map.len());

        let mut total_rate = 0.0;
        let mut self_rate = 0.0;
        let mut result = String::new();
        for val in map.values() {
            // println!("destination: {:#?}", val);

            result.push_str(&val.source.file);
            for _ in 0..(max_width_of_source_file_name - &val.source.file.len()) {
                result.push(' ')
            }
            result.push_str(" ");
            result.push_str(&val.destination[0].file);
            for _ in 0..(max_width_of_dest_file_name - &val.destination[0].file.len()) {
                result.push(' ')
            }

            let code_lines = count_lines(&val.source.file, true);
            result.push_str(&format!("\t{}\t", code_lines));
            result.push_str(&format!("\t{:.2}%\t", val.rate_of_source_code() * 100.0));
            result.push_str(&format!("\t{}\t", val.lines));
            result.push_str(&format!("\t{:.2}%\n", val.dup_rate() * 100.0));

            total_rate += val.dup_rate();
            self_rate += val.rate_of_source_code();
        }
        println!("{}", result);

        // TODO: using language from args.
        let langs = [Lang::Swift];

        let des_files = Scanner::scan(&args.destination, &langs).num_of_files();
        let source_files = Scanner::scan(&args.source, &langs).num_of_files();

        println!("Total rate: {:.2}%", total_rate / des_files as f32 * 100.0);
        println!(
            "Total rate of self: {:.2}%",
            self_rate / source_files as f32 * 100.0
        );
    }
}

pub fn read_lines<P, F>(path: P, mut f: F) -> io::Result<()>
where
    P: AsRef<Path>,
    F: FnMut(&String),
{
    let file = File::open(path)?;
    let lines = BufReader::new(file).lines();

    for line in lines {
        match line {
            Ok(str) => f(&str),
            Err(_) => break,
        }
    }
    Ok(())
}

pub fn count_lines<P>(path: P, ignore_blank: bool) -> usize
where
    P: AsRef<Path>,
{
    let mut lines = 0usize;
    let result = read_lines(path, |line| {
        if *line != "" || !ignore_blank {
            lines += 1
        }
    });

    match result {
        Ok(_) => return lines,
        Err(_) => return 0,
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn count_lines() {
        assert!(super::count_lines("./Cargo.toml", true) == 8);
        assert!(super::count_lines("./Cargo.toml", false) == 10);
    }
}
