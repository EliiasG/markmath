mod unit_lib;

mod language;
mod markdown;

use crate::language::expression::EvaluationContext;
use crate::language::format::FormattableLibraryProvider;
use crate::language::latex_impl::LatexFormatter;
use crate::unit_lib::{CLIUnitLib, UnitCollection};
use std::path::Path;
use std::{fs, io};

pub enum CompileMode {
    Resolving,
    NonResolving,
    Live,
}

pub fn run(compile_mode: CompileMode, input: &Path, output: &Path) -> io::Result<()> {
    let input = fs::read_to_string(&input)?;
    let mut unit_lib = CLIUnitLib::new(UnitCollection::new());
    let mut eval_ctx = EvaluationContext::new();
    let lib = FormattableLibraryProvider::new(LatexFormatter { precision: 5 });
    let res = markdown::parse_markdown(&input, &mut eval_ctx, &mut unit_lib, &lib);
    fs::write(output, res)?;
    Ok(())
}
