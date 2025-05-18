use super::*;
use crate::language::expression::{
    DefinedUnit, EvaluationContext, Expression, LibraryProvider, Unit,
};
use std::collections::HashMap;

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

    pub fn make_calculations<'a, Lib: UnitLibrary>(&'a self, eval_ctx: &'a mut EvaluationContext, unit_lib: &'a mut Lib) -> CalculationsBuilder<'a, F, Lib> {
        CalculationsBuilder {
            lib: self,
            eval_ctx,
            unit_lib,
            calculations: Calculations(Vec::new()),
        }
    }
    
    pub fn format_calculations(
        &self,
        unit_lib: &impl UnitLibrary,
        calculations: Calculations,
    ) -> Vec<String> {
        calculations
            .0
            .into_iter()
            .map(|c| match c {
                Calculation::Single { expr, result } => {
                    let expr = self.resolve_formattable_expression(unit_lib, expr);
                    let result = result.map(|r| self.resolve_formattable_expression(unit_lib, r));
                    self.formatter.format_single(self, &expr, result.as_ref())
                }
                Calculation::Multi(v) => {
                    let res: Vec<_> = v
                        .into_iter()
                        .map(|(c, r)| {
                            (
                                self.resolve_formattable_expression(unit_lib, c),
                                self.resolve_formattable_expression(unit_lib, r),
                            )
                        })
                        .collect();
                    self.formatter.format_multi(self, &res)
                }
            })
            .collect()
    }

    pub fn generate_formattable_expression(
        &self,
        eval_ctx: &EvaluationContext,
        unit_lib: &mut impl UnitLibrary,
        exp: &Expression,
        value_mode: ValueMode,
        parenthesise: bool,
    ) -> UnresolvedFormattableExpression {
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
                let unit = name
                    .as_ref()
                    .map(|name| {
                        let d = DefinedUnit::Defined(name.clone());
                        unit_lib.cache_defined_unit(&d);
                        Unit::Defined(d)
                    })
                    .unwrap_or(Unit::None);
                self.handle_unit(eval_ctx, unit_lib, value_mode, unit, &child)
            }
            Expression::LiteralUnit { name, child } => self.handle_unit(
                eval_ctx,
                unit_lib,
                value_mode,
                Unit::Literal(name.clone()),
                child,
            ),
            Expression::VariableRef(name) => match value_mode {
                ValueMode::NumbersNoUnit | ValueMode::NumbersWithUnit => {
                    let (value, unit) = eval_ctx
                        .get_variable(name)
                        .expect("variable not found, call eval and get Ok before formatting");
                    if value_mode == ValueMode::NumbersWithUnit {
                        if let Unit::Defined(d) = &unit {
                            unit_lib.cache_defined_unit(d);
                        }
                        FormattableExpression::Number { value, unit }
                    } else {
                        FormattableExpression::Number {
                            value,
                            unit: Unit::None,
                        }
                    }
                }
                ValueMode::NamedLiteralUnit | ValueMode::NamedNoUnit => {
                    FormattableExpression::Variable(name.to_string())
                }
            },
            Expression::NumberLiteral(v) => FormattableExpression::Number {
                value: *v,
                unit: Unit::None,
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

    pub fn resolve_formattable_expression(
        &self,
        unit_lib: &impl UnitLibrary,
        unresolved: UnresolvedFormattableExpression,
    ) -> ResolvedFormattableExpression {
        unresolved.map_unit(&mut |unit| match unit {
            Unit::Defined(d) => Some(unit_lib.get_defined_unit(&d).to_string()),
            Unit::Literal(l) => Some(l),
            Unit::None => None,
        })
    }

    pub fn write_expression(&self, exp: &ResolvedFormattableExpression, out: &mut String) {
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
        unit: Unit,
        child: &Box<Expression>,
    ) -> UnresolvedFormattableExpression {
        let value = if let ValueMode::NamedNoUnit | ValueMode::NumbersNoUnit = value_mode {
            return self
                .generate_formattable_expression(eval_ctx, unit_lib, child, value_mode, false);
        } else if let Expression::NumberLiteral(v) = child.as_ref() {
            *v
        } else if let (Expression::VariableRef(var_name), ValueMode::NumbersWithUnit) =
            (child.as_ref(), value_mode)
        {
            eval_ctx
                .get_variable(var_name)
                .expect("variable not found, call eval and get Ok before formatting")
                .0
        } else {
            return self
                .generate_formattable_expression(eval_ctx, unit_lib, child, value_mode, false);
        };
        if let Unit::Defined(d) = &unit {
            unit_lib.cache_defined_unit(&d);
        }
        FormattableExpression::Number { value, unit }
    }

    /// Appends fmt to out, where $n becomes the formatted result of args\[n\].  
    pub fn fmt_expression(
        &self,
        fmt: &str,
        args: &[&ResolvedFormattableExpression],
        out: &mut String,
    ) {
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
