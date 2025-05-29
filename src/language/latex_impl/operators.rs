use crate::language::format::{BasicOperator, FormattableOperator};
use crate::language::latex_impl::LatexFormatter;

pub fn operators() -> Vec<Box<dyn FormattableOperator<LatexFormatter>>> {
    vec![
        Box::new(Add),
        Box::new(Sub),
        Box::new(Mul),
        Box::new(Div),
        Box::new(DivSymbol),
        Box::new(Pow),
    ]
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

struct Sub;

impl BasicOperator<LatexFormatter> for Sub {
    const PRECEDENCE: u32 = 0;
    const ASSOCIATIVE: bool = false;
    const SHOULD_PARENTHESIZE_LEFT: bool = true;
    const SHOULD_PARENTHESIZE_RIGHT: bool = true;
    const SYMBOL: &'static str = "-";
    const FMT: &'static str = "$0 - $1";

    fn eval(&self, left: f64, right: f64) -> Result<f64, String> {
        Ok(left - right)
    }
}

struct Mul;

impl BasicOperator<LatexFormatter> for Mul {
    const PRECEDENCE: u32 = 1;
    const ASSOCIATIVE: bool = true;
    const SHOULD_PARENTHESIZE_LEFT: bool = true;
    const SHOULD_PARENTHESIZE_RIGHT: bool = true;
    const SYMBOL: &'static str = "*";
    const FMT: &'static str = "$0 \\cdot $1";

    fn eval(&self, left: f64, right: f64) -> Result<f64, String> {
        Ok(left * right)
    }
}

struct Div;

impl BasicOperator<LatexFormatter> for Div {
    const PRECEDENCE: u32 = 1;
    const ASSOCIATIVE: bool = false;
    const SHOULD_PARENTHESIZE_LEFT: bool = false;
    const SHOULD_PARENTHESIZE_RIGHT: bool = false;
    const SYMBOL: &'static str = "/";
    const FMT: &'static str = "\\dfrac{$0}{$1}";

    fn eval(&self, left: f64, right: f64) -> Result<f64, String> {
        div(left, right)
    }
}

struct DivSymbol;

impl BasicOperator<LatexFormatter> for DivSymbol {
    const PRECEDENCE: u32 = 1;
    const ASSOCIATIVE: bool = false;
    const SHOULD_PARENTHESIZE_LEFT: bool = true;
    const SHOULD_PARENTHESIZE_RIGHT: bool = true;
    const SYMBOL: &'static str = "//";
    const FMT: &'static str = "$0\\div $1";

    fn eval(&self, left: f64, right: f64) -> Result<f64, String> {
        div(left, right)
    }
}

fn div(left: f64, right: f64) -> Result<f64, String> {
    if right == 0. {
        Err("division by zero".to_string())
    } else {
        Ok(left / right)
    }
}

struct Pow;

impl BasicOperator<LatexFormatter> for Pow {
    const PRECEDENCE: u32 = 2;
    const ASSOCIATIVE: bool = false;
    const SHOULD_PARENTHESIZE_LEFT: bool = true;
    const SHOULD_PARENTHESIZE_RIGHT: bool = false;
    const SYMBOL: &'static str = "**";
    const FMT: &'static str = "$0^{$1}";

    fn eval(&self, left: f64, right: f64) -> Result<f64, String> {
        Ok(left.powf(right))
    }
}
