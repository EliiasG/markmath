use crate::expression::LibraryProvider;
use std::collections::HashMap;

pub enum FormattableExpression {
    Function {
        name: String,
        args: Box<Vec<FormattableExpression>>,
    },
    Operator {
        operator: String,
        left: Box<FormattableExpression>,
        right: Box<FormattableExpression>,
    },
    Negate(Box<FormattableExpression>),
    Parenthesis(Box<FormattableExpression>),
    Variable(String),
    Number {
        value: f64,
        unit: Option<String>,
    },
}

pub trait LanguageFormatter: Sized {
    fn parenthesise(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &FormattableExpression,
        out: &mut String,
    );

    fn negate(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &FormattableExpression,
        out: &mut String,
    );

    fn write_number(&self, number: f64, unit: Option<&str>, out: &mut String);

    fn write_variable(&self, variable: &str, out: &mut String);

    fn build_operators(&self) -> Vec<Box<dyn FormattableOperator<Self>>>;

    fn build_functions(&self) -> Vec<Box<dyn FormattableFunction<Self>>>;
}

pub trait FormattableOperator<Formatter: LanguageFormatter> {
    fn precedence(&self) -> u32;

    fn is_associative(&self) -> bool;

    fn symbol(&self) -> &str;

    fn eval(&self, left: f64, right: f64) -> Result<f64, String>;

    fn write(
        &self,
        lib: &FormattableLibraryProvider<Formatter>,
        out: &mut String,
        left: &FormattableExpression,
        right: &FormattableExpression,
    );
}

pub trait FormattableFunction<Formatter: LanguageFormatter> {
    fn name(&self) -> &str;

    fn supports_arg_count(&self, argc: usize) -> bool;

    fn eval(&self, args: &[f64]) -> Result<f64, String>;

    fn write(
        &self,
        lib: &FormattableLibraryProvider<Formatter>,
        out: &mut String,
        args: &[FormattableExpression],
    );
}

pub struct FormattableLibraryProvider<Formatter: LanguageFormatter> {
    functions: HashMap<String, Box<dyn FormattableFunction<Formatter>>>,
    operators: HashMap<String, Box<dyn FormattableOperator<Formatter>>>,
    formatter: Formatter,
}

impl<F: LanguageFormatter> FormattableLibraryProvider<F> {
    pub fn new(formatter: F) -> Self {
        let mut funcs = HashMap::new();
        let mut ops = HashMap::new();
        for f in formatter.build_functions() {
            if let Some(v) = funcs.insert(f.name().to_string(), f) {
                panic!("Duplicate function: {}", v.name());
            }
        }
        for o in formatter.build_operators() {
            if let Some(v) = ops.insert(o.symbol().to_string(), o) {
                panic!("Duplicate operator: {}", v.symbol());
            }
        }
        Self {
            functions: funcs,
            operators: ops,
            formatter,
        }
    }

    pub fn write_expression(&self, exp: &FormattableExpression, out: &mut String) {
        match exp {
            FormattableExpression::Operator {
                operator,
                left,
                right,
            } => self
                .operators
                .get(operator)
                .expect("operator not found")
                .write(self, out, left, right),
            FormattableExpression::Function { name, args } => self
                .functions
                .get(name)
                .expect("function not found")
                .write(self, out, args),

            FormattableExpression::Negate(child) => self.formatter.negate(self, child, out),
            FormattableExpression::Parenthesis(child) => {
                self.formatter.parenthesise(self, child, out)
            }
            FormattableExpression::Variable(v) => self.formatter.write_variable(v, out),
            FormattableExpression::Number { value, unit } => {
                self.formatter
                    .write_number(*value, unit.as_ref().map(|s| s.as_str()), out)
            }
        }
    }

    fn fmt_expression(&self, args: &[&FormattableExpression], fmt: &str, out: &mut String) {
        let mut exp = false;
        let mut num = String::new();
        let push = |num: &mut String, out: &mut String| {
            let n: usize = num.parse().unwrap();
            num.clear();
            self.write_expression(args[n], out);
        };
        for c in fmt.chars() {
            if exp {
                if c.is_numeric() {
                    num.push(c);
                } else {
                    push(&mut num, out);
                    out.push(c);
                    exp = false;
                }
            } else {
                if c == '$' {
                    exp = true;
                } else {
                    out.push(c);
                }
            }
        }
        push(&mut num, out);
    }
}

impl<F: LanguageFormatter> LibraryProvider for FormattableLibraryProvider<F> {
    type LibraryError = String;

    fn function_exists(&self, name: &str, param_c: usize) -> bool {
        self.functions
            .get(name)
            .map_or(false, |f| f.supports_arg_count(param_c))
    }

    fn operator_exists(&self, symbol: &str) -> bool {
        self.operators.contains_key(symbol)
    }

    fn eval_function(&self, name: &str, params: &[f64]) -> Result<f64, Self::LibraryError> {
        self.functions
            .get(name)
            .expect("should call function_exists before evaluating function")
            .as_ref()
            .eval(params)
    }

    fn eval_operator(
        &self,
        symbol: &str,
        left: f64,
        right: f64,
    ) -> Result<f64, Self::LibraryError> {
        self.operators
            .get(symbol)
            .expect("should call operator_exists before evaluating operator")
            .eval(left, right)
    }

    fn operator_associative(&self, symbol: &str) -> bool {
        self.operators
            .get(symbol)
            .expect("should call operator_exists before acessing operator")
            .is_associative()
    }

    fn operator_precedence(&self, symbol: &str) -> u32 {
        self.operators
            .get(symbol)
            .expect("should call operator_exists before acessing operator")
            .precedence()
    }
}

pub trait BasicOperator<Formatter: LanguageFormatter> {
    const PRECEDENCE: u32;
    const ASSOCIATIVE: bool;
    const SYMBOL: &'static str;
    /// Will be used for formatting, \$0 will be replaced by the left arg and \$1 will be replaced by the right arg  
    /// \$\$ becomes \$
    const FMT: &'static str;

    fn eval(&self, left: f64, right: f64) -> Result<f64, String>;
}

impl<F: LanguageFormatter, T: BasicOperator<F>> FormattableOperator<F> for T {
    fn precedence(&self) -> u32 {
        T::PRECEDENCE
    }

    fn is_associative(&self) -> bool {
        T::ASSOCIATIVE
    }

    fn symbol(&self) -> &str {
        T::SYMBOL
    }

    fn eval(&self, left: f64, right: f64) -> Result<f64, String> {
        self.eval(left, right)
    }

    fn write(
        &self,
        lib: &FormattableLibraryProvider<F>,
        out: &mut String,
        left: &FormattableExpression,
        right: &FormattableExpression,
    ) {
        lib.fmt_expression(&[left, right], T::FMT, out);
    }
}

pub trait BasicFunction<Formatter: LanguageFormatter> {
    const NAME: &'static str;
    const ARG_COUNT: usize;

    /// \$n will become param n where n is a number  
    /// \$\$ becomes \$
    const FMT: &'static str;

    fn eval(&self, args: &[f64]) -> Result<f64, String>;
}

impl<F: LanguageFormatter, T: BasicFunction<F>> FormattableFunction<F> for T {
    fn name(&self) -> &str {
        T::NAME
    }

    fn supports_arg_count(&self, argc: usize) -> bool {
        argc == T::ARG_COUNT
    }

    fn eval(&self, args: &[f64]) -> Result<f64, String> {
        self.eval(args)
    }

    fn write(
        &self,
        lib: &FormattableLibraryProvider<F>,
        out: &mut String,
        args: &[FormattableExpression],
    ) {
        let refs: Vec<_> = args.iter().collect();
        lib.fmt_expression(&refs, T::FMT, out);
    }
}
