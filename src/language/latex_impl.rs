mod functions;
mod operators;

use crate::language::format::{
    FormattableFunction, FormattableLibraryProvider, FormattableOperator, LanguageFormatter,
    ResolvedFormattableExpression,
};

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
        let num = format!("{:.*}", self.precision, number);
        let num = num.trim_end_matches('0').trim_end_matches('.');
        let unit = unit
            .map(|u| format!("\\small\\text{{ {u}}}\\normalsize"))
            .unwrap_or(String::new());
        out.push_str(&format!("{num}{unit}"))
    }

    fn write_variable(&self, variable: &str, out: &mut String) {
        let parts: Vec<_> = variable.split('_').collect();
        let mut r = parts.join("_{");
        r.push_str(&"}".repeat(parts.len()-1));
        out.push_str(&format!("\\mathit{{{}}}", &r));
    }

    fn format_single(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &ResolvedFormattableExpression,
        result: Option<&ResolvedFormattableExpression>,
    ) -> String {
        let mut res = String::new();
        if let Some(result) = result {
            lib.fmt_expression("$$$0 = $1$$", &[expr, result], &mut res);
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
        out.push_str("\\end{align*} $$");
        out
    }

    fn build_operators(&self) -> Vec<Box<dyn FormattableOperator<Self>>> {
        operators::operators()
    }

    fn build_functions(&self) -> Vec<Box<dyn FormattableFunction<Self>>> {
        functions::functions()
    }
}
