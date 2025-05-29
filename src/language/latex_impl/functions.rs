use crate::language::format::BasicFunction;
use crate::language::format::FormattableFunction;
use crate::language::latex_impl::LatexFormatter;

pub fn functions() -> Vec<Box<dyn FormattableFunction<LatexFormatter>>> {
    vec![
        Box::new(Pi),
        Box::new(E),
        Box::new(Parenthesize),
        Box::new(Floor),
        Box::new(Ceil),
        Box::new(Abs),
        Box::new(Sqrt),
        Box::new(NRoot),
        Box::new(Log10),
        Box::new(Log),
        Box::new(Sin),
        Box::new(Cos),
        Box::new(Tan),
        Box::new(Atan),
        Box::new(Asin),
        Box::new(Acos),
        Box::new(Modulo),
        Box::new(Precision),
        Box::new(Display),
    ]
}

macro_rules! impl_basic_function {
    ($type:ty, $name:expr, $arg_count:expr, $fmt:expr, |$args:ident| $eval:block) => {
        impl BasicFunction<LatexFormatter> for $type {
            const NAME: &'static str = $name;
            const ARG_COUNT: usize = $arg_count;
            const FMT: &'static str = $fmt;

            fn eval(&self, $args: &[f64]) -> Result<f64, String> $eval
        }
    };
}

struct Pi;
impl_basic_function!(Pi, "pi", 0, "\\pi", |_args| { Ok(std::f64::consts::PI) });

struct E;
impl_basic_function!(E, "e", 0, "e", |_args| { Ok(std::f64::consts::E) });

struct Parenthesize;
impl_basic_function!(Parenthesize, "par", 1, "( $0 )", |args| { Ok(args[0]) });

struct Floor;
impl_basic_function!(Floor, "floor", 1, "\\lfloor $0 \\rfloor", |args| { Ok(args[0].floor()) });

struct Ceil;
impl_basic_function!(Ceil, "ceil", 1, "\\lceil $0 \\rceil", |args| { Ok(args[0].ceil()) });

struct Abs;
impl_basic_function!(Abs, "abs", 1, "|$0|", |args| { Ok(args[0].abs()) });

struct Sqrt;
impl_basic_function!(Sqrt, "sqrt", 1, "\\sqrt{$0}", |args| {
    if args[0] < 0.0 {
        Err("sqrt of negative number".into())
    } else {
        Ok(args[0].sqrt())
    }
});

struct NRoot;
impl_basic_function!(NRoot, "nroot", 2, "\\sqrt[$1]{$0}", |args| {
    if args[1] == 0.0 {
        Err("root with exponent 0".into())
    } else {
        Ok(args[0].powf(1.0 / args[1]))
    }
});

struct Log10;
impl_basic_function!(Log10, "log10", 1, "\\log_{10}{$0}", |args| {
    if args[0] <= 0.0 {
        Err("log10 of non-positive number".into())
    } else {
        Ok(args[0].log10())
    }
});

struct Log;
impl_basic_function!(Log, "log", 2, "\\log_{$1}{$0}", |args| {
    if args[0] <= 0.0 || args[1] <= 0.0 {
        Err("log of non-positive number".into())
    } else {
        Ok(args[0].log(args[1]))
    }
});

struct Sin;
impl_basic_function!(Sin, "sin", 1, "\\sin{$0}", |args| { Ok(args[0].to_radians().sin()) });

struct Cos;
impl_basic_function!(Cos, "cos", 1, "\\cos{$0}", |args| { Ok(args[0].to_radians().cos()) });

struct Tan;
impl_basic_function!(Tan, "tan", 1, "\\tan{$0}", |args| { Ok(args[0].to_radians().tan()) });

struct Atan;
impl_basic_function!(Atan, "atan", 1, "\\tan^{-1}{$0}", |args| { Ok(args[0].atan().to_degrees()) });

struct Asin;
impl_basic_function!(Asin, "asin", 1, "\\sin^{-1}{$0}", |args| {
    if args[0].abs() > 1.0 {
        Err("asin domain error".into())
    } else {
        Ok(args[0].asin().to_degrees())
    }
});

struct Acos;
impl_basic_function!(Acos, "acos", 1, "\\cos^{-1}{$0}", |args| {
    if args[0].abs() > 1.0 {
        Err("acos domain error".into())
    } else {
        Ok(args[0].acos().to_degrees())
    }
});

struct Modulo;
impl_basic_function!(Modulo, "mod", 2, "$0\\bmod$1", |args| {
    if args[1] == 0.0 {
        Err("mod by zero".into())
    } else {
        Ok(args[0] % args[1])
    }
});

struct Precision;
impl_basic_function!(Precision, "p", 2, "$0", |args| {
    Ok((args[0] / args[1]).round() * args[1])
});

struct Display;
impl_basic_function!(Display, "disp", 2, "$1", |args| {
   Ok(args[0])
});

