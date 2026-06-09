use crate::language::expression::{EvaluationContext, Expression, LibraryProvider};
use crate::language::parse::{TokenTree, tokenize};
use std::f32::consts::E;

/// Real one would be [FormattableLibraryProvider](crate::language::format::FormattableLibraryProvider)

struct MockLibraryProvider;
impl LibraryProvider for MockLibraryProvider {
    type LibraryError = String;

    fn function_exists(&self, name: &str, param_c: usize) -> bool {
        name == "sum"
    }

    fn operator_exists(&self, symbol: &str) -> bool {
        matches!(symbol, "+" | "-" | "*" | "/" | "^" | "?")
    }

    fn eval_function(&self, name: &str, params: &[f64]) -> Result<f64, Self::LibraryError> {
        Ok(params.iter().sum())
    }

    fn eval_operator(
        &self,
        symbol: &str,
        left: f64,
        right: f64,
    ) -> Result<f64, Self::LibraryError> {
        match symbol {
            "+" => Ok(left + right),
            "-" => Ok(left - right),
            "*" => Ok(left * right),
            "/" => {
                if right == 0. {
                    Err("div 0".into())
                } else {
                    Ok(left / right)
                }
            }
            "^" => Ok(left.powf(right)),
            "?" => Ok(42.),
            _ => panic!("illegal operator"),
        }
    }

    fn operator_associative(&self, symbol: &str) -> bool {
        symbol != "/" && symbol != "^" && symbol != "-"
    }

    fn operator_precedence(&self, symbol: &str) -> u32 {
        match symbol {
            "+" | "-" => 0,
            "*" | "/" => 1,
            "^" => 2,
            "?" => 3,
            _ => panic!("Unknown operator {}", symbol),
        }
    }
}

fn assert_expr(inp: TokenTree, out: Expression) {
    // unwrap asserts no err
    let exp = Expression::new(inp, &MockLibraryProvider).unwrap();
    assert_eq!(exp, out);
}

#[test]
fn numbers() {
    assert_expr(
        TokenTree::NumberLiteral("5".into()),
        Expression::NumberLiteral(5.),
    );
    assert_expr(
        TokenTree::NumberLiteral("5.".into()),
        Expression::NumberLiteral(5.),
    );
    // careful with float precision
    assert_expr(
        TokenTree::NumberLiteral("0.25".into()),
        Expression::NumberLiteral(0.25),
    );
    assert_expr(
        TokenTree::NumberLiteral(".25".into()),
        Expression::NumberLiteral(0.25),
    );
}

#[test]
fn units() {
    assert_expr(
        TokenTree::DefinedUnit {
            name: "Meter".into(),
            child: Box::new(TokenTree::NumberLiteral("5".into())),
        },
        Expression::DefinedUnit {
            name: Some("Meter".into()),
            child: Box::new(Expression::NumberLiteral(5.)),
        },
    );
    assert_expr(
        TokenTree::DefinedUnit {
            name: "None".into(),
            child: Box::new(TokenTree::NumberLiteral("5".into())),
        },
        Expression::DefinedUnit {
            name: None,
            child: Box::new(Expression::NumberLiteral(5.)),
        },
    );
    assert_expr(
        TokenTree::LiteralUnit {
            name: "m".into(),
            child: Box::new(TokenTree::NumberLiteral("5".into())),
        },
        Expression::LiteralUnit {
            name: "m".into(),
            child: Box::new(Expression::NumberLiteral(5.)),
        },
    );
}

#[test]
fn variable_ref() {
    assert_expr(
        TokenTree::VariableRef("my_fun_variable".into()),
        Expression::VariableRef("my_fun_variable".into()),
    );
}

#[test]
fn functions() {
    let invalid_func = TokenTree::FunctionCall {
        name: "invalid_name".to_string(),
        args: vec![TokenTree::NumberLiteral("7".into())],
    };
    // func name does not exist in lib provider
    assert!(Expression::new(invalid_func, &MockLibraryProvider).is_err());
    assert_expr(
        TokenTree::FunctionCall {
            name: "sum".to_string(),
            args: vec![],
        },
        Expression::FunctionCall {
            function: "sum".to_string(),
            args: vec![],
        },
    );
    assert_expr(
        TokenTree::FunctionCall {
            name: "sum".to_string(),
            args: vec![
                TokenTree::NumberLiteral("1".into()),
                TokenTree::NumberLiteral("2".into()),
            ],
        },
        Expression::FunctionCall {
            function: "sum".to_string(),
            args: vec![Expression::NumberLiteral(1.), Expression::NumberLiteral(2.)],
        },
    );
}

#[test]
fn negate() {
    assert_expr(
        TokenTree::Negate(Box::new(TokenTree::VariableRef("my_var".into()))),
        Expression::Negate(Box::new(Expression::VariableRef("my_var".into()))),
    );
}

#[test]
fn simple_operator() {
    let invalid_op = TokenTree::OperatorSequence {
        operators: vec!["=".into()],
        children: vec![
            TokenTree::NumberLiteral("1".into()),
            TokenTree::NumberLiteral("2".into()),
        ],
    };
    // operator does not exist in lib provider
    assert!(Expression::new(invalid_op, &MockLibraryProvider).is_err());

    assert_expr(
        TokenTree::OperatorSequence {
            operators: vec!["+".into()],
            children: vec![
                TokenTree::NumberLiteral("1".into()),
                TokenTree::NumberLiteral("2".into()),
            ],
        },
        Expression::Operator {
            operator: "+".into(),
            left: Box::new(Expression::NumberLiteral(1.)),
            right: Box::new(Expression::NumberLiteral(2.)),
        },
    );
    assert_expr(
        TokenTree::OperatorSequence {
            operators: vec!["*".into()],
            children: vec![
                TokenTree::NumberLiteral("1".into()),
                TokenTree::VariableRef("my_var".into()),
            ],
        },
        Expression::Operator {
            operator: "*".into(),
            left: Box::new(Expression::NumberLiteral(1.)),
            right: Box::new(Expression::VariableRef("my_var".into())),
        },
    );
}

#[test]
fn operator_sequence() {
    assert_expr(
        TokenTree::OperatorSequence {
            operators: vec!["+".into(), "+".into()],
            children: vec![
                TokenTree::NumberLiteral("7".into()),
                TokenTree::NumberLiteral("1".into()),
                TokenTree::NumberLiteral("3".into()),
            ],
        },
        Expression::Operator {
            operator: "+".into(),
            left: Box::new(Expression::Operator {
                operator: "+".into(),
                left: Box::new(Expression::NumberLiteral(7.)),
                right: Box::new(Expression::NumberLiteral(1.)),
            }),
            right: Box::new(Expression::NumberLiteral(3.)),
        },
    );

    assert_expr(
        TokenTree::OperatorSequence {
            operators: vec!["+".into(), "*".into()],
            children: vec![
                TokenTree::NumberLiteral("7".into()),
                TokenTree::NumberLiteral("1".into()),
                TokenTree::NumberLiteral("3".into()),
            ],
        },
        Expression::Operator {
            operator: "+".into(),
            left: Box::new(Expression::NumberLiteral(7.)),
            right: Box::new(Expression::Operator {
                operator: "*".into(),
                left: Box::new(Expression::NumberLiteral(1.)),
                right: Box::new(Expression::NumberLiteral(3.)),
            }),
        },
    );

    assert_expr(
        TokenTree::OperatorSequence {
            operators: vec![
                "*".into(),
                "+".into(),
                "/".into(),
                "*".into(),
                "^".into(),
                "+".into(),
            ],
            children: vec![
                TokenTree::NumberLiteral("7".into()),
                TokenTree::NumberLiteral("1".into()),
                TokenTree::NumberLiteral("3".into()),
                TokenTree::NumberLiteral("2".into()),
                TokenTree::NumberLiteral("5".into()),
                TokenTree::NumberLiteral("9".into()),
                TokenTree::NumberLiteral("42".into()),
            ],
        },
        Expression::Operator {
            operator: "+".into(),
            left: Box::new(Expression::Operator {
                operator: "+".into(),
                left: Box::new(Expression::Operator {
                    operator: "*".into(),
                    left: Box::new(Expression::NumberLiteral(7.)),
                    right: Box::new(Expression::NumberLiteral(1.)),
                }),
                right: Box::new(Expression::Operator {
                    operator: "*".into(),
                    left: Box::new(Expression::Operator {
                        operator: "/".into(),
                        left: Box::new(Expression::NumberLiteral(3.)),
                        right: Box::new(Expression::NumberLiteral(2.)),
                    }),
                    right: Box::new(Expression::Operator {
                        operator: "^".into(),
                        left: Box::new(Expression::NumberLiteral(5.)),
                        right: Box::new(Expression::NumberLiteral(9.)),
                    }),
                }),
            }),
            right: Box::new(Expression::NumberLiteral(42.)),
        },
    )
}

#[test]
fn eval() {
    let mut ctxt = EvaluationContext::new();
    let expr = Expression::Operator {
        operator: "+".into(),
        left: Box::new(Expression::FunctionCall {
            function: "sum".into(),
            args: vec![
                Expression::NumberLiteral(3.25),
                Expression::Negate(Box::new(Expression::Operator {
                    operator: "*".into(),
                    left: Box::new(Expression::NumberLiteral(7.)),
                    right: Box::new(Expression::NumberLiteral(3.)),
                })),
                Expression::Operator {
                    operator: "+".into(),
                    left: Box::new(Expression::Operator {
                        operator: "*".into(),
                        left: Box::new(Expression::Operator {
                            operator: "^".into(),
                            left: Box::new(Expression::NumberLiteral(2.)),
                            right: Box::new(Expression::NumberLiteral(3.)),
                        }),
                        right: Box::new(Expression::NumberLiteral(2.)),
                    }),
                    right: Box::new(Expression::NumberLiteral(1.75)),
                },
            ],
        }),
        right: Box::new(Expression::NumberLiteral(42.)),
    };
    // unwrap is part of test as it should not be err
    assert_eq!(expr.eval(&MockLibraryProvider, &mut ctxt).unwrap().0, 42.)
}
