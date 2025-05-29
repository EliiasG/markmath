mod library_provider;

pub use library_provider::*;

use crate::language::expression::{
    DefinedUnit, EvaluationContext, EvaluationError, Expression, LibraryProvider, Unit,
};

/// A sort of middleman between an [Expression] and a [String].  
/// The Unit is generic because it can be either [Unit](crate::language::expression::Unit) or [Option<String>].   
/// The former case is defined as an [UnresolvedFormattableExpression], and units are still not resolved.   
pub enum FormattableExpression<Unit> {
    Function {
        name: String,
        args: Box<Vec<FormattableExpression<Unit>>>,
    },
    Operator {
        operator: String,
        left: Box<FormattableExpression<Unit>>,
        right: Box<FormattableExpression<Unit>>,
    },
    Negate(Box<FormattableExpression<Unit>>),
    Parenthesis(Box<FormattableExpression<Unit>>),
    Variable(String),
    Number {
        value: f64,
        unit: Unit,
    },
}

impl<U> FormattableExpression<U> {
    pub fn map_unit<O>(self, mut f: impl FnMut(U) -> O) -> FormattableExpression<O> {
        self.map_unit_impl(&mut f)
    }

    /// to make public api better
    fn map_unit_impl<O>(self, f: &mut impl FnMut(U) -> O) -> FormattableExpression<O> {
        match self {
            Self::Function { name, args } => FormattableExpression::<O>::Function {
                name,
                args: Box::new(args.into_iter().map(|e| e.map_unit_impl(f)).collect()),
            },
            Self::Operator {
                operator,
                left,
                right,
            } => FormattableExpression::<O>::Operator {
                operator,
                left: Box::new(left.map_unit_impl(f)),
                right: Box::new(right.map_unit_impl(f)),
            },
            Self::Negate(child) => {
                FormattableExpression::<O>::Negate(Box::new(child.map_unit_impl(f)))
            }
            Self::Parenthesis(child) => {
                FormattableExpression::<O>::Parenthesis(Box::new(child.map_unit_impl(f)))
            }
            Self::Variable(name) => FormattableExpression::<O>::Variable(name),
            Self::Number { value, unit } => FormattableExpression::<O>::Number {
                value,
                unit: f(unit),
            },
        }
    }
}

/// A [FormattableExpression] That needs resolving
pub type UnresolvedFormattableExpression = FormattableExpression<Unit>;
/// A [FormattableExpression] where units are resolved.  
pub type ResolvedFormattableExpression = FormattableExpression<Option<String>>;

pub trait LanguageFormatter: Sized {
    fn parenthesise(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &ResolvedFormattableExpression,
        out: &mut String,
    );

    fn negate(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &ResolvedFormattableExpression,
        out: &mut String,
    );

    fn write_number(&self, number: f64, unit: Option<&str>, out: &mut String);

    fn write_variable(&self, variable: &str, out: &mut String);

    fn format_single(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &ResolvedFormattableExpression,
        result: Option<&ResolvedFormattableExpression>,
    ) -> String;

    fn format_multi(
        &self,
        lib: &FormattableLibraryProvider<Self>,
        expr: &[(ResolvedFormattableExpression, ResolvedFormattableExpression)],
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
        left: &ResolvedFormattableExpression,
        right: &ResolvedFormattableExpression,
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
        args: &[ResolvedFormattableExpression],
    );
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
        left: &ResolvedFormattableExpression,
        right: &ResolvedFormattableExpression,
    ) {
        lib.fmt_expression(T::FMT, &[left, right], out);
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
        args: &[ResolvedFormattableExpression],
    ) {
        let refs: Vec<_> = args.iter().collect();
        lib.fmt_expression(T::FMT, &refs, out);
    }
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

/// Responsible for evaluating unit names and applying operators to units.  
pub trait UnitLibrary: Sized {
    /// Called when generating [FormattableExpression]s.
    /// This can be used to make a list of what units should be resolved before resolution.  
    fn cache_defined_unit(&mut self, unit: &DefinedUnit);

    /// Called after calls of [cache_defined_unit](Self::cache_defined_unit), after this is called, units should be available for [get_defined_unit](Self::get_defined_unit)
    fn resolve_units(&mut self) {}

    /// Called during formatting to get unit names.
    /// It should be expected that [cache_defined_unit](Self::cache_defined_unit) has been called for the given unit.
    fn get_defined_unit(&self, unit: &DefinedUnit) -> String;
}

pub struct CalculationsBuilder<'a, Formatter: LanguageFormatter, Lib: UnitLibrary> {
    lib: &'a FormattableLibraryProvider<Formatter>,
    eval_ctx: &'a mut EvaluationContext,
    unit_lib: &'a mut Lib,
    calculations: Calculations,
}

impl<'a, F: LanguageFormatter, L: UnitLibrary> CalculationsBuilder<'a, F, L> {
    pub fn add_single_calculation(
        &mut self,
        exp: &Expression,
        value_mode: ValueMode,
    ) -> Result<
        usize,
        EvaluationError<<FormattableLibraryProvider<F> as LibraryProvider>::LibraryError>,
    > {
        let mut result = None;
        if let ValueMode::NumbersWithUnit | ValueMode::NumbersNoUnit = value_mode {
            // important that eval happens before generating fexp
            let (value, unit) = exp.eval(self.lib, self.eval_ctx)?;
            if let Unit::Defined(d) = &unit {
                self.unit_lib.cache_defined_unit(d);
            }
            result = Some(UnresolvedFormattableExpression::Number { value, unit });
        }
        // okay to generate without evaluating if variable values are not needed
        let expr = self.lib.generate_formattable_expression(
            self.eval_ctx,
            self.unit_lib,
            exp,
            value_mode,
            false,
        );
        self.calculations
            .0
            .push(Calculation::Single { expr, result });
        Ok(self.calculations.0.len() - 1)
    }

    pub fn add_multi_calculation(
        &mut self,
        exps: &[Expression],
        display_units: bool,
    ) -> Result<
        usize,
        EvaluationError<<FormattableLibraryProvider<F> as LibraryProvider>::LibraryError>,
    > {
        let val_mode = if display_units {
            ValueMode::NumbersWithUnit
        } else {
            ValueMode::NumbersNoUnit
        };
        #[rustfmt::skip]
        let fexps = exps
            .iter()
            .map(|exp| {
                let (value, unit) = exp.eval(self.lib, self.eval_ctx)?;
                if let Unit::Defined(d) = &unit {
                    self.unit_lib.cache_defined_unit(d);
                }
                Ok((
                    self.lib.generate_formattable_expression(self.eval_ctx, self.unit_lib, exp, val_mode, false),
                    FormattableExpression::Number { value, unit },
                ))
            })
            .collect::<Result<Vec<_>, EvaluationError<_>>>()?;
        self.calculations.0.push(Calculation::Multi(fexps));
        Ok(self.calculations.0.len() - 1)
    }

    pub fn finish(self) -> Calculations {
        self.calculations
    }
}

pub struct Calculations(Vec<Calculation>);

enum Calculation {
    Single {
        expr: UnresolvedFormattableExpression,
        result: Option<UnresolvedFormattableExpression>,
    },
    Multi(
        Vec<(
            UnresolvedFormattableExpression,
            UnresolvedFormattableExpression,
        )>,
    ),
}
