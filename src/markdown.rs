use crate::language::expression::{
    EvaluationContext, Expression, LibraryProvider,
};
use crate::language::format::{
    FormattableLibraryProvider, LanguageFormatter, UnitLibrary, ValueMode,
};
use crate::language::parse;
use std::mem;

pub fn parse_markdown<F: LanguageFormatter>(
    source: &str,
    eval_ctx: &mut EvaluationContext,
    unit_lib: &mut impl UnitLibrary,
    lib: &mut FormattableLibraryProvider<F>,
) -> String {
    let mut blocks = get_blocks(source).into_iter();
    let mut res = String::new();
    loop {
        let Some(block) = blocks.next() else {
            break;
        };
        res.push_str(&block);
        let Some(block) = blocks.next() else {
            break;
        };
        res.push_str(&handle_code_block(&block, eval_ctx, unit_lib, lib));
    }
    res
}

fn get_blocks(source: &str) -> Vec<String> {
    let mut itr = source.chars().peekable();
    let mut cur = String::new();
    let mut blocks = Vec::new();
    while let Some(c) = itr.next() {
        if c == '^' {
            if itr.peek() == Some(&'^') {
                cur.push('^');
                itr.next();
            } else {
                blocks.push(mem::take(&mut cur));
            }
        } else {
            cur.push(c);
        }
    }
    blocks.push(cur);
    blocks
}

fn handle_code_block<F: LanguageFormatter>(
    block: &str,
    eval_ctx: &mut EvaluationContext,
    unit_lib: &mut impl UnitLibrary,
    lib: &mut FormattableLibraryProvider<F>,
) -> String {
    let mut render_vars = false;
    let mut render_units = false;
    let mut i = 0;
    for (j, c) in block.char_indices() {
        if c.is_whitespace() {
            i = j;
            break;
        }
        if c == 'u' {
            render_units = false;
        } else if c == 'v' {
            render_vars = true;
        } else {
            return format_err(&format!("Invalid, flag: {c}"));
        }
    }
    let val_mode = match (render_vars, render_units) {
        (false, false) => ValueMode::NumbersNoUnit,
        (false, true) => ValueMode::NumbersWithUnit,
        (true, false) => ValueMode::NamedNoUnit,
        (true, true) => ValueMode::NamedLiteralUnit,
    };
    let mut lines: Vec<_> = block[i..].lines().collect();
    let mut exps = Vec::new();
    let mut err = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let exp = match exp(line, lib) {
            Ok(r) => r,
            Err(e) => {
                err = Some((i, e));
                break;
            }
        };
        exps.push(exp);
    }
    if let Some((i, e)) = err {
        return if lines.len() == 1 {
            format_err(&format!("Error: {e}"))
        } else {
            format_err(&format!("Error on line {i}: {e}"))
        };
    }
    let res = if lines.len() == 1 {
        lib.make_single_calculation(eval_ctx, unit_lib, &exps[0], val_mode)
    } else {
        lib.make_multi_calculation(eval_ctx, unit_lib, &exps, render_units)
    };
    res.unwrap_or_else(|e| format_err(&format!("{e:?}")))
}

fn exp(source: &str, lib: &impl LibraryProvider) -> Result<Expression, String> {
    let tokens = match parse::tokenize(source) {
        Ok(r) => r,
        Err(e) => return Err(format!("{e:?}")),
    };
    match Expression::new(tokens, lib) {
        Ok(r) => Ok(r),
        Err(e) => Err(format!("{e:?}")),
    }
}

fn format_err(error: &str) -> String {
    todo!("format error")
}
