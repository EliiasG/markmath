use crate::language::expression::{
    EvaluationContext, Expression, LibraryProvider,
};
use crate::language::format::{CalculationsBuilder, FormattableLibraryProvider, LanguageFormatter, UnitLibrary, ValueMode};
use crate::language::parse;
use std::mem;

pub fn parse_markdown<F: LanguageFormatter>(
    source: &str,
    eval_ctx: &mut EvaluationContext,
    unit_lib: &mut impl UnitLibrary,
    lib: &FormattableLibraryProvider<F>,
) -> String {
    let mut blocks = get_blocks(source).into_iter();
    let mut text_blocks = Vec::new();
    let mut code_blocks = Vec::new();
    let mut cb = lib.make_calculations(eval_ctx, unit_lib);
    loop {
        let Some(block) = blocks.next() else {
            break;
        };
        text_blocks.push(block);
        let Some(block) = blocks.next() else {
            break;
        };
        code_blocks.push(handle_code_block(&block, lib, &mut cb));
    }
    let calc = cb.finish();
    unit_lib.resolve_units();
    let mut code = lib.format_calculations(unit_lib, calc);
    let mut code_blocks = code_blocks.into_iter().map(|block| match block {
        Ok(i) => i.map(|i| mem::take(&mut code[i])).unwrap_or_else(String::new),
        Err(s) => s,
    }).collect::<Vec<_>>().into_iter();
    let mut res = String::new();
    for t in text_blocks {
        res.push_str(&t);
        if let Some(c) = code_blocks.next() {
            res.push_str(&c);
        }
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

fn handle_code_block<F: LanguageFormatter, U: UnitLibrary>(
    block: &str,
    lib: &FormattableLibraryProvider<F>,
    cb: &mut CalculationsBuilder<F, U>,
) -> Result<Option<usize>, String> {
    let mut render_vars = false;
    let mut render_units = true;
    let mut visible = true;
    let mut i = 0;
    for (j, c) in block.char_indices() {
        if c.is_whitespace() {
            i = j;
            break;
        }
        match c {
            'u' => render_units = false,
            'v' => render_vars = true,
            'i' => visible = false,
            _ => return Err(format_err(&format!("Invalid preflag: {c}"))),
        }
    }
    let val_mode = match (render_vars, render_units) {
        (false, false) => ValueMode::NumbersNoUnit,
        (false, true) => ValueMode::NumbersWithUnit,
        (true, false) => ValueMode::NamedNoUnit,
        (true, true) => ValueMode::NamedLiteralUnit,
    };
    let lines: Vec<_> = block[i..].lines().collect();
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
        return Err(if lines.len() == 1 {
            format_err(&format!("Error: {e}"))
        } else {
            format_err(&format!("Error on line {i}: {e}"))
        });
    }
    let res = if lines.len() == 1 {
        cb.add_single_calculation(&exps[0], val_mode)
    } else {
        cb.add_multi_calculation(&exps, render_units)
    };
    res.map_err(|e| format_err(&format!("{e:?}"))).map(|r| Some(r).filter(|_| visible))
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
    format!("<span style=\"color:red\">{error}</span>")
}
