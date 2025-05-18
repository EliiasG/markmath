use crate::language::format::{BasicOperator, FormattableFunction, FormattableLibraryProvider, FormattableOperator, LanguageFormatter, ResolvedFormattableExpression};

pub struct LatexFormatter {
    pub precision: usize,
}

impl LanguageFormatter for LatexFormatter {
    fn parenthesise(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &ResolvedFormattableExpression,
        out: &mut String,
    ) {
        lib.fmt_expression("($0)", &[expr], out);
    }

    fn negate(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &ResolvedFormattableExpression,
        out: &mut String,
    ) {
        lib.fmt_expression("-$0", &[expr], out);
    }

    fn write_number(&self, number: f64, unit: Option<&str>, out: &mut String) {
        // TODO clean off the \scriptsize if no unit
        out.push_str(&format!(
            "\\textbf{{{number:.prec$}}}\\text{{\\scriptsize{{{uni}}}}}",
            prec = self.precision,
            uni = unit.unwrap_or("")
        ))
    }

    fn write_variable(&self, variable: &str, out: &mut String) {
        todo!("format vars")
    }

    fn format_single(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &ResolvedFormattableExpression,
        result: Option<&ResolvedFormattableExpression>,
    ) -> String {
        let mut res = String::new();
        if let Some(result) = result {
            lib.fmt_expression("$$$0=$1$$", &[expr, result], &mut res);
        } else {
            lib.fmt_expression("$$$0$$", &[expr], &mut res);
        }
        res
    }

    fn format_multi(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &[(ResolvedFormattableExpression, ResolvedFormattableExpression)],
    ) -> String {
        let mut out = "$$ \\begin{align*}\n ".to_string();
        for (exp, res) in expr {
            lib.fmt_expression("$0 &= $1\\\\ \\\\\n", &[exp, res], &mut out);
        }
        out.push_str("\\end{align*}\n");
        out
    }

    fn build_operators(&self) -> Vec<Box<dyn FormattableOperator<Self>>> {
        todo!()
    }

    fn build_functions(&self) -> Vec<Box<dyn FormattableFunction<Self>>> {
        todo!()
    }
}

struct Add;
impl BasicOperator<LatexFormatter> for Add {
    const PRECEDENCE: u32 = 0;
    const ASSOCIATIVE: bool = true;
    const SHOULD_PARENTHESIZE_LEFT: bool = true;
    const SHOULD_PARENTHESIZE_RIGHT: bool = true;
    const SYMBOL: &'static str = "+";
    const FMT: &'static str = "$0 + $1";

    fn eval(&self, left: f64, right: f64) -> Result<f64, String> {
        Ok(left + right)
    }
}