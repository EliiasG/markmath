mod unit_lib;

mod language;
mod markdown;

use crate::language::expression::EvaluationContext;
use crate::language::format::FormattableLibraryProvider;
use crate::language::latex_impl::LatexFormatter;
use crate::unit_lib::{CLIUnitLib, UnitCollection};
use std::path::Path;
use std::{fs, io, thread};
use std::io::ErrorKind;
use std::process::Command;
use std::time::Duration;

const UNIT_PATH: &str = "units.txt";

#[derive(Debug, PartialEq, Eq)]
pub enum CompileMode {
    Resolving,
    NonResolving,
    Live,
}

pub fn run(compile_mode: CompileMode, input: &Path, output: &Path) -> io::Result<()> {
    let unit_collection = match fs::read_to_string(UNIT_PATH) {
        Ok(s) => {
            match s.parse() {
                Ok(c) => c,
                Err(e) => {
                    println!("Error parsing units: {}", e);
                    // return Ok cause no io err
                    return Ok(());
                }
            }
        }
        Err(_) => {
            println!("no unit collection, creating empty");
            UnitCollection::new()
        }
    };
    let html = match output.extension().map(|s| s.to_str()).flatten() {
        Some("html") => true,
        Some("md") => false,
        _ => return Err(io::Error::new(ErrorKind::Unsupported, "invalid output extension")),
    };
    let out = output.with_extension("md");
    let mut unit_lib = CLIUnitLib::new(unit_collection, compile_mode == CompileMode::Resolving);
    let lib = FormattableLibraryProvider::new(LatexFormatter { precision: 5 });
    loop {
        let mut eval_ctx = EvaluationContext::new();
        let input = fs::read_to_string(&input)?;
        let res = markdown::parse_markdown(&input, &mut eval_ctx, &mut unit_lib, &lib);
        fs::write(&out, res)?;
        match Command::new("pandoc").arg(&out).arg("-o").arg(&output).args(["--katex", "-s"]).status() {
            Ok(s) => {
                if !s.success() {
                    panic!("pandoc fail with code {}", s.code().unwrap());
                }
            }
            Err(e) => {
                panic!("pandoc executor exited with error: {}", e);
            }
        }
        if compile_mode != CompileMode::Live {
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }
    
    fs::write(UNIT_PATH, unit_lib.finish().to_string())?;
    Ok(())
}
