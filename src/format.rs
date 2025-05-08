use crate::expression::{
    DefinedUnit, EvaluationContext, EvaluationError, Expression, LibraryProvider, Unit,
};
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

    fn format_single(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &FormattableExpression,
        result: Option<&FormattableExpression>,
    ) -> String;

    fn format_multi(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &[(FormattableExpression, FormattableExpression)],
    ) -> String;

    fn build_operators(&self) -> Vec<Box<dyn FormattableOperator<Self>>>;

    fn build_functions(&self) -> Vec<Box<dyn FormattableFunction<Self>>>;
}

pub trait FormattableOperator<Formatter: LanguageFormatter> {
    fn precedence(&self) -> u32;

    fn is_associative(&self) -> bool;

    /// Werther parenthesis can be added to the left (false for something like divide line or power)  
    fn should_parenthesize_left(&self) -> bool;

    /// Werther parenthesis can be to the right added (false for something like divide line)  
    fn should_parenthesize_right(&self) -> bool;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueMode {
    /// All variables get converted to numbers, and units are added to all numbers  
    NumbersWithUnit,
    /// All variables get converted to numbers, but no units on any numbers
    NumbersNoUnit,
    /// Variables get names, and number literals are with units
    NamedLiteralUnit,
    /// Variables get names, and units are never added
    NamedNoUnit,
}

pub trait UnitLibrary: Sized {
    fn resolve_defined_unit(&mut self, unit: &DefinedUnit) -> &str;

    fn unit_name(&mut self, unit: Unit) -> Option<String> {
        match unit {
            Unit::Defined(d) => Some(self.resolve_defined_unit(&d).to_string()),
            Unit::Literal(l) => Some(l),
            Unit::None => None,
        }
    }
}

pub struct FormattableLibraryProvider<F: LanguageFormatter> {
    functions: HashMap<String, Box<dyn FormattableFunction<F>>>,
    operators: HashMap<String, Box<dyn FormattableOperator<F>>>,
    formatter: F,
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

    pub fn make_single_calculation(
        &self,
        eval_ctx: &mut EvaluationContext,
        unit_lib: &mut impl UnitLibrary,
        exp: &Expression,
        value_mode: ValueMode,
    ) -> Result<String, EvaluationError<<Self as LibraryProvider>::LibraryError>> {
        let fexp = self.generate_formattable_expression(eval_ctx, unit_lib, exp, value_mode, false);
        let mut res = String::new();
        let mut res_fexp = None;
        if let ValueMode::NumbersWithUnit | ValueMode::NumbersNoUnit = value_mode {
            let (res_v, res_u) = exp.eval(self, eval_ctx)?;
            res_fexp = Some(FormattableExpression::Number {
                value: res_v,
                unit: unit_lib.unit_name(res_u),
            });
        }
        Ok(self.formatter.format_single(self, &fexp, res_fexp.as_ref()))
    }

    pub fn make_multi_calculation(
        &self,
        eval_ctx: &mut EvaluationContext,
        unit_lib: &mut impl UnitLibrary,
        exps: &[Expression],
        display_units: bool,
    ) -> Result<String, EvaluationError<<Self as LibraryProvider>::LibraryError>> {
        let val_mode = if display_units {
            ValueMode::NumbersWithUnit
        } else {
            ValueMode::NumbersNoUnit
        };
        #[rustfmt::skip]
        let fexps = exps
            .iter()
            .map(|exp| {
                let (r_v, r_u) = exp.eval(self, eval_ctx)?;
                let unit = unit_lib.unit_name(r_u).filter(|_| display_units);
                Ok((
                    self.generate_formattable_expression(eval_ctx, unit_lib, exp, val_mode, false),
                    FormattableExpression::Number { value: r_v, unit },
                ))
            })
            .collect::<Result<Vec<_>, EvaluationError<<Self as LibraryProvider>::LibraryError>>>()?;
        //    ^ rustfmt wanted the last 3 chars on a new line... ^
        Ok(self.formatter.format_multi(self, &fexps))
    }

    pub fn generate_formattable_expression(
        &self,
        eval_ctx: &EvaluationContext,
        unit_lib: &mut impl UnitLibrary,
        exp: &Expression,
        value_mode: ValueMode,
        parenthesise: bool,
    ) -> FormattableExpression {
        if parenthesise {
            return FormattableExpression::Parenthesis(Box::new(
                self.generate_formattable_expression(eval_ctx, unit_lib, exp, value_mode, false),
            ));
        }

        match exp {
            Expression::VariableAssign { child, .. } => {
                self.generate_formattable_expression(eval_ctx, unit_lib, child, value_mode, false)
            }
            Expression::Operator {
                operator,
                left,
                right,
            } => {
                let p_l = if let Expression::Operator { operator: l_op, .. } = left.as_ref() {
                    self.operator_precedence(operator) > self.operator_precedence(l_op)
                        && self.operators[operator].should_parenthesize_left()
                } else {
                    false
                };
                let p_r = if let Expression::Operator { operator: r_op, .. } = right.as_ref() {
                    self.operator_precedence(operator) > self.operator_precedence(r_op)
                        && self.operators[operator].should_parenthesize_right()
                } else {
                    false
                };

                let left =
                    self.generate_formattable_expression(eval_ctx, unit_lib, left, value_mode, p_l);
                let right = self
                    .generate_formattable_expression(eval_ctx, unit_lib, right, value_mode, p_r);

                FormattableExpression::Operator {
                    operator: operator.clone(),
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }
            Expression::FunctionCall { function, args } => {
                let fargs = args
                    .iter()
                    .map(|e| {
                        self.generate_formattable_expression(
                            eval_ctx, unit_lib, e, value_mode, false,
                        )
                    })
                    .collect();
                FormattableExpression::Function {
                    name: function.clone(),
                    args: Box::new(fargs),
                }
            }
            Expression::DefinedUnit { name, child } => {
                let name = name.as_ref().map(|n| {
                    unit_lib
                        .resolve_defined_unit(&DefinedUnit::Defined(n.clone()))
                        .to_string()
                });
                self.handle_unit(
                    eval_ctx,
                    unit_lib,
                    value_mode,
                    name.as_ref().map(String::as_str),
                    &child,
                )
            }
            Expression::LiteralUnit { name, child } => {
                self.handle_unit(eval_ctx, unit_lib, value_mode, Some(name.as_str()), child)
            }
            Expression::VariableRef(name) => match value_mode {
                ValueMode::NumbersNoUnit | ValueMode::NumbersWithUnit => {
                    let (value, unit) = eval_ctx
                        .get_variable(name)
                        .expect("variable not found, call eval and get Ok before formatting");
                    if let ValueMode::NumbersWithUnit = value_mode {
                        let unit = unit_lib.unit_name(unit);
                        FormattableExpression::Number { value, unit }
                    } else {
                        FormattableExpression::Number { value, unit: None }
                    }
                }
                ValueMode::NamedLiteralUnit | ValueMode::NamedNoUnit => {
                    FormattableExpression::Variable(name.to_string())
                }
            },
            Expression::NumberLiteral(v) => FormattableExpression::Number {
                value: *v,
                unit: None,
            },
            Expression::Negate(child) => {
                if let Expression::Operator { operator, .. } = child.as_ref() {
                    if self.operators[operator].should_parenthesize_left() {
                        return FormattableExpression::Negate(Box::new(
                            self.generate_formattable_expression(
                                eval_ctx, unit_lib, child, value_mode, true,
                            ),
                        ));
                    }
                }
                FormattableExpression::Negate(Box::new(
                    self.generate_formattable_expression(
                        eval_ctx, unit_lib, child, value_mode, false,
                    ),
                ))
            }
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

    fn handle_unit(
        &self,
        eval_ctx: &EvaluationContext,
        unit_lib: &mut impl UnitLibrary,
        value_mode: ValueMode,
        unit: Option<&str>,
        child: &Box<Expression>,
    ) -> FormattableExpression {
        if let ValueMode::NamedNoUnit | ValueMode::NumbersNoUnit = value_mode {
            self.generate_formattable_expression(eval_ctx, unit_lib, child, value_mode, false)
        } else if let Expression::NumberLiteral(v) = child.as_ref() {
            FormattableExpression::Number {
                value: *v,
                unit: unit.map(str::to_string),
            }
        } else if let (Expression::VariableRef(var_name), ValueMode::NumbersWithUnit) =
            (child.as_ref(), value_mode)
        {
            FormattableExpression::Number {
                value: eval_ctx
                    .get_variable(var_name)
                    .expect("variable not found, call eval and get Ok before formatting")
                    .0,
                unit: unit.map(str::to_string),
            }
        } else {
            self.generate_formattable_expression(eval_ctx, unit_lib, child, value_mode, false)
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
            .expect("should call operator_exists before accessing operator")
            .is_associative()
    }

    fn operator_precedence(&self, symbol: &str) -> u32 {
        self.operators
            .get(symbol)
            .expect("should call operator_exists before accessing operator")
            .precedence()
    }
}

pub trait BasicOperator<Formatter: LanguageFormatter> {
    const PRECEDENCE: u32;
    const ASSOCIATIVE: bool;

    const SHOULD_PARENTHESIZE_LEFT: bool;
    const SHOULD_PARENTHESIZE_RIGHT: bool;

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

    fn should_parenthesize_left(&self) -> bool {
        T::SHOULD_PARENTHESIZE_LEFT
    }

    fn should_parenthesize_right(&self) -> bool {
        T::SHOULD_PARENTHESIZE_RIGHT
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
