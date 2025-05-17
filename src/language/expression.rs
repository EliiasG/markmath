use crate::language::parse::TokenTree;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

pub trait LibraryProvider {
    type LibraryError: Debug;

    fn function_exists(&self, name: &str, param_c: usize) -> bool;
    fn operator_exists(&self, symbol: &str) -> bool;

    fn eval_function(&self, name: &str, params: &[f64]) -> Result<f64, Self::LibraryError>;

    fn eval_operator(&self, symbol: &str, left: f64, right: f64)
    -> Result<f64, Self::LibraryError>;

    fn operator_associative(&self, symbol: &str) -> bool;

    fn operator_precedence(&self, symbol: &str) -> u32;
}

#[derive(Clone)]
pub enum Unit {
    Defined(DefinedUnit),
    Literal(String),
    None,
}

#[derive(Clone)]
pub enum DefinedUnit {
    Defined(String),
    Implicit {
        operator: String,
        /// Nice to save here, as it makes it easier to resolve the unit later
        associative: bool,
        left: Box<DefinedUnit>,
        right: Box<DefinedUnit>,
    },
}

pub struct EvaluationContext {
    map: HashMap<String, (f64, Unit)>,
}
impl EvaluationContext {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<(f64, Unit)> {
        self.map.get(name).cloned()
    }

    pub fn store_variable(&mut self, name: &str, value: (f64, Unit)) {
        self.map.insert(name.to_string(), value);
    }
}

pub enum ExpressionError {
    InvalidFunction { name: String, param_c: usize },
    InvalidNumber(String),
    InvalidOperator(String),
}

pub enum EvaluationError<LibraryError: Debug> {
    LibraryError(LibraryError),
    MissingVariable { name: String },
}

impl<LibraryError: Debug> Debug for EvaluationError<LibraryError> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            EvaluationError::LibraryError(err) => err.fmt(f),
            EvaluationError::MissingVariable { name } => write!(f, "Variable '{}' not found", name),
        }
    }
}

impl<LibraryError: Debug> From<LibraryError> for EvaluationError<LibraryError> {
    fn from(value: LibraryError) -> Self {
        Self::LibraryError(value)
    }
}

impl Debug for ExpressionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionError::InvalidFunction { name, param_c } => write!(
                f,
                "Invalid function: '{}', with {} parameter(s)",
                name, param_c
            ),
            ExpressionError::InvalidOperator(op) => write!(f, "Invalid operator: '{}'", op),
            ExpressionError::InvalidNumber(num) => write!(f, "Invalid number: '{}'", num),
        }
    }
}

pub enum Expression {
    VariableAssign {
        name: String,
        child: Box<Expression>,
    },
    Operator {
        operator: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    FunctionCall {
        function: String,
        args: Vec<Expression>,
    },
    DefinedUnit {
        name: Option<String>,
        child: Box<Expression>,
    },
    LiteralUnit {
        name: String,
        child: Box<Expression>,
    },
    VariableRef(String),
    NumberLiteral(f64),
    Negate(Box<Expression>),
}

impl Expression {
    pub fn new(
        token_tree: TokenTree,
        provider: &impl LibraryProvider,
    ) -> Result<Expression, ExpressionError> {
        match token_tree {
            TokenTree::VariableAssign { name, child } => Ok(Self::VariableAssign {
                name,
                child: Box::new(Self::new(*child, provider)?),
            }),
            TokenTree::OperatorSequence {
                operators,
                children,
            } => {
                let children = children
                    .into_iter()
                    .map(|tt| Self::new(tt, provider))
                    .collect::<Result<_, _>>()?;
                Ok(transform_operators(provider, operators, children))
            }
            TokenTree::DefinedUnit { name, child } => Ok(Self::DefinedUnit {
                name: if name == "None" { None } else { Some(name) },
                child: Box::new(Self::new(*child, provider)?),
            }),
            TokenTree::LiteralUnit { name, child } => Ok(Self::LiteralUnit {
                name,
                child: Box::new(Self::new(*child, provider)?),
            }),
            TokenTree::FunctionCall { name, args } => {
                if provider.function_exists(&name, args.len()) {
                    Ok(Self::FunctionCall {
                        function: name,
                        args: args
                            .into_iter()
                            .map(|tt| Expression::new(tt, provider))
                            .collect::<Result<_, _>>()?,
                    })
                } else {
                    Err(ExpressionError::InvalidFunction {
                        name,
                        param_c: args.len(),
                    })
                }
            }

            TokenTree::VariableRef(name) => Ok(Self::VariableRef(name)),
            TokenTree::NumberLiteral(val) => {
                if let Ok(v) = val.parse() {
                    Ok(Self::NumberLiteral(v))
                } else {
                    Err(ExpressionError::InvalidNumber(val))
                }
            }
            TokenTree::Negate(child) => Ok(Self::Negate(Box::new(Self::new(*child, provider)?))),
        }
    }

    pub fn eval<LP: LibraryProvider>(
        &self,
        provider: &LP,
        context: &mut EvaluationContext,
    ) -> Result<(f64, Unit), EvaluationError<LP::LibraryError>> {
        match &self {
            Expression::VariableAssign { name, child } => {
                let res = child.eval(provider, context)?;
                context.store_variable(name, res.clone());
                Ok(res)
            }
            Expression::Operator {
                operator,
                left,
                right,
            } => {
                let (l_v, l_u) = left.eval(provider, context)?;
                let (r_v, r_u) = right.eval(provider, context)?;
                let res_v = provider.eval_operator(operator, l_v, r_v)?;
                let res_u = match l_u {
                    Unit::Defined(l_d) => match r_u {
                        Unit::Defined(r_d) => Unit::Defined(DefinedUnit::Implicit {
                            operator: operator.clone(),
                            associative: provider.operator_associative(&operator),
                            left: Box::new(l_d),
                            right: Box::new(r_d),
                        }),
                        Unit::Literal(_) | Unit::None => Unit::Defined(l_d),
                    },
                    Unit::Literal(l_s) => match r_u {
                        Unit::Defined(r_s) => Unit::Defined(r_s),
                        Unit::Literal(_) => Unit::None,
                        Unit::None => Unit::Literal(l_s),
                    },
                    Unit::None => r_u,
                };
                Ok((res_v, res_u))
            }
            Expression::FunctionCall { function, args } => {
                let r = provider.eval_function(
                    function,
                    &args
                        .into_iter()
                        .map(|arg| arg.eval(provider, context).map(|(v, _)| v))
                        .collect::<Result<Vec<_>, _>>()?,
                )?;
                Ok((r, Unit::None))
            }
            Expression::VariableRef(name) => {
                if let Some(r) = context.get_variable(name) {
                    Ok(r)
                } else {
                    Err(EvaluationError::MissingVariable { name: name.clone() })
                }
            }
            Expression::DefinedUnit { name, child } => {
                let (r, _) = child.eval(provider, context)?;
                Ok((
                    r,
                    name.as_ref()
                        .map_or(Unit::None, |n| Unit::Defined(DefinedUnit::Defined(n.clone())))
                ))
            }
            Expression::LiteralUnit { name, child } => {
                let (r, _) = child.eval(provider, context)?;
                Ok((r, Unit::Literal(name.clone())))
            }
            Expression::NumberLiteral(num) => Ok((*num, Unit::None)),
            Expression::Negate(expr) => {
                let (r, u) = expr.eval(provider, context)?;
                Ok((-r, u))
            }
        }
    }
}

/// Must have independent tree for transforming operators, to not get mixed up with already transformed operators
enum TransformNode {
    Op {
        left: Box<Self>,
        right: Box<Self>,
        op: String,
    },
    Expr(Expression),
}

impl TransformNode {
    fn transform(self, provider: &impl LibraryProvider) -> Self {
        let Self::Op { left, right, op } = self else {
            return self;
        };
        let left = left.transform(provider);
        let Self::Op {
            left: l_left,
            right: l_right,
            op: l_op,
        } = left
        else {
            return Self::Op {
                left: Box::new(left),
                right,
                op,
            };
        };
        if provider.operator_precedence(&op) > provider.operator_precedence(&l_op) {
            Self::Op {
                left: l_left,
                right: Box::new(Self::Op {
                    left: l_right,
                    right,
                    op,
                }),
                op: l_op,
            }
        } else {
            Self::Op {
                left: Box::new(Self::Op {
                    left: l_left,
                    right: l_right,
                    op: l_op,
                }),
                right,
                op,
            }
        }
    }

    fn compile(self) -> Expression {
        match self {
            TransformNode::Op { left, right, op } => Expression::Operator {
                operator: op,
                left: Box::new(left.compile()),
                right: Box::new(right.compile()),
            },
            TransformNode::Expr(e) => e,
        }
    }
}

fn transform_operators(
    provider: &impl LibraryProvider,
    operators: Vec<String>,
    expressions: Vec<Expression>,
) -> Expression {
    let mut exp = expressions.into_iter();
    let mut op = operators.into_iter();
    let mut l = TransformNode::Op {
        left: Box::new(TransformNode::Expr(
            exp.next()
                .expect("expected at least 2 expressions in opseq"),
        )),
        right: Box::new(TransformNode::Expr(
            exp.next()
                .expect("expected at least 2 expressions in opseq"),
        )),
        op: op.next().expect("expected at least 1 operator in opseq"),
    };
    while let (Some(e), Some(op)) = (exp.next(), op.next()) {
        l = TransformNode::Op {
            left: Box::new(l),
            right: Box::new(TransformNode::Expr(e)),
            op,
        };
    }
    l = l.transform(provider);
    l.compile()
}
