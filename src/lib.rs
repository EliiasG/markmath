mod unit_lib;

mod markdown;
mod language;


use std::path::Path;

pub enum CompileMode {
    Resolving,
    NonResolving,
    Live,
}

pub fn run(compile_mode: CompileMode, input: &Path, output: &Path) {
    
}