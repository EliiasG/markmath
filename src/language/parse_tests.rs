use super::parse::*;
use TokenTree::*;

#[test]
fn empty() {
    assert!(tokenize("").is_err());
}

#[test]
fn numbers() {
    assert_eq!(tokenize("1"), Ok(TokenTree::NumberLiteral("1".into())));
    assert_eq!(tokenize("1.5"), Ok(TokenTree::NumberLiteral("1.5".into())));
    assert_eq!(tokenize(".75"), Ok(TokenTree::NumberLiteral(".75".into())));
    assert_eq!(tokenize("1."), Ok(TokenTree::NumberLiteral("1.".into())));
}

// unit^2 testing
#[test]
fn units() {
    assert_eq!(
        tokenize("1 Meter"),
        Ok(DefinedUnit {
            name: "Meter".into(),
            child: Box::new(NumberLiteral("1".into()))
        })
    );
    assert_eq!(
        tokenize("1 \"m\""),
        Ok(LiteralUnit {
            name: "m".into(),
            child: Box::new(NumberLiteral("1".into()))
        })
    );
}

#[test]
fn variable_ref() {
    assert_eq!(
        tokenize("this_is_a_variable"),
        Ok(VariableRef("this_is_a_variable".into()))
    );
}

#[test]
fn negate() {
    assert_eq!(
        tokenize("-2"),
        Ok(Negate(Box::new(NumberLiteral("2".into()))))
    );
    assert_eq!(
        tokenize("-variable"),
        Ok(Negate(Box::new(VariableRef("variable".into()))))
    );
    assert!(tokenize("--5").is_err());
}

#[test]
fn parentheses() {
    assert_eq!(tokenize("2"), tokenize("(2)"));
    assert_eq!(tokenize("2"), tokenize("(((2)))"));
    assert_eq!(tokenize("2 + 3"), tokenize("(2 + 3)"));
    // operator precedence is unknow at this point. The first is one OperatorSequence, second is two
    assert_ne!(tokenize("2 + 3 * 7"), tokenize("2 + (3 * 7)"));
    assert_eq!(tokenize("-2"), tokenize("-(2)"));
    assert_eq!(tokenize("2 Meter"), tokenize("(2)Meter"));
}

#[test]
fn simple_operators() {
    assert_eq!(
        tokenize("1 + 4"),
        Ok(OperatorSequence {
            operators: vec!["+".into()],
            children: vec![
                TokenTree::NumberLiteral("1".into()),
                TokenTree::NumberLiteral("4".into()),
            ]
        })
    );
    assert_eq!(
        tokenize("a_variable // 7"),
        Ok(OperatorSequence {
            operators: vec!["//".into()],
            children: vec![
                VariableRef("a_variable".into()),
                TokenTree::NumberLiteral("7".into()),
            ]
        })
    );
}

#[test]
fn operator_sequence() {
    assert_eq!(
        tokenize("7 + 1 * 8 + my_var - 57.3"),
        Ok(OperatorSequence {
            operators: vec!["+".into(), "*".into(), "+".into(), "-".into()],
            children: vec![
                TokenTree::NumberLiteral("7".into()),
                TokenTree::NumberLiteral("1".into()),
                TokenTree::NumberLiteral("8".into()),
                VariableRef("my_var".into()),
                TokenTree::NumberLiteral("57.3".into()),
            ]
        })
    )
}

#[test]
fn functions() {
    assert_eq!(
        tokenize("simple()"),
        Ok(FunctionCall {
            name: "simple".into(),
            args: vec![]
        })
    );
    assert_eq!(
        tokenize("simple_with_arg(1)"),
        Ok(FunctionCall {
            name: "simple_with_arg".into(),
            args: vec![TokenTree::NumberLiteral("1".into())]
        })
    );
    assert_eq!(
        tokenize("two_arguments(2, var)"),
        Ok(FunctionCall {
            name: "two_arguments".into(),
            args: vec![
                TokenTree::NumberLiteral("2".into()),
                VariableRef("var".into()),
            ]
        })
    );
    assert_eq!(
        tokenize("so_many_arguments(var_1, 3, 5, 75)"),
        Ok(FunctionCall {
            name: "so_many_arguments".into(),
            args: vec![
                VariableRef("var_1".into()),
                TokenTree::NumberLiteral("3".into()),
                TokenTree::NumberLiteral("5".into()),
                TokenTree::NumberLiteral("75".into()),
            ]
        })
    );
}

#[test]
fn advanced() {
    assert_eq!(
        tokenize(
            "7 + floor(pi()) - avg(7, my_var, 45, nest(nest(nest(9, nest(9 - 4)))) * y + 8 - 7)"
        ),
        Ok(OperatorSequence {
            operators: vec!["+".into(), "-".into()],
            children: vec![
                TokenTree::NumberLiteral("7".into()),
                FunctionCall {
                    name: "floor".into(),
                    args: vec![FunctionCall {
                        name: "pi".into(),
                        args: vec![],
                    }],
                },
                FunctionCall {
                    name: "avg".into(),
                    args: vec![
                        TokenTree::NumberLiteral("7".into()),
                        VariableRef("my_var".into()),
                        TokenTree::NumberLiteral("45".into()),
                        OperatorSequence {
                            operators: vec!["*".into(), "+".into(), "-".into()],
                            children: vec![
                                FunctionCall {
                                    name: "nest".into(),
                                    args: vec![FunctionCall {
                                        name: "nest".into(),
                                        args: vec![FunctionCall {
                                            name: "nest".into(),
                                            args: vec![
                                                TokenTree::NumberLiteral("9".into()),
                                                FunctionCall {
                                                    name: "nest".into(),
                                                    args: vec![OperatorSequence {
                                                        operators: vec!["-".into()],
                                                        children: vec![
                                                            TokenTree::NumberLiteral("9".into()),
                                                            TokenTree::NumberLiteral("4".into()),
                                                        ],
                                                    }],
                                                },
                                            ],
                                        },],
                                    },],
                                },
                                VariableRef("y".into()),
                                TokenTree::NumberLiteral("8".into()),
                                TokenTree::NumberLiteral("7".into()),
                            ],
                        },
                    ],
                },
            ],
        })
    );
}
