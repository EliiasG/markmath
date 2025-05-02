use std::cmp::PartialEq;
use std::fmt::{Debug, Display, Formatter};

pub enum TokenTree {
    VariableAssign {
        name: String,
        child: Box<TokenTree>,
    },
    OperatorSequence {
        operators: Vec<String>,
        children: Vec<TokenTree>,
    },
    DefinedUnit {
        name: String,
        child: Box<TokenTree>,
    },
    LiteralUnit {
        name: String,
        child: Box<TokenTree>,
    },
    FunctionCall {
        name: String,
        args: Vec<TokenTree>,
    },
    VariableRef {
        name: String,
        /// True if the variable should be rendered as an expression
        exp: bool,
    },
    NumberLiteral(String),
    Negate(Box<TokenTree>),
}

impl Display for TokenTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let r = match &self {
            TokenTree::VariableAssign { name, child } => {
                format!("{} = {}", name, child)
            }
            TokenTree::OperatorSequence {
                operators,
                children,
            } => {
                let mut r = "(".to_string() + &children[0].to_string();
                for (i, o) in operators.iter().enumerate() {
                    r += " ";
                    r += o;
                    r += " ";
                    r += &children[i + 1].to_string();
                }
                r + ")"
            }
            TokenTree::DefinedUnit { name, child } => {
                format!("{} {}", child.to_string(), name)
            }
            TokenTree::LiteralUnit { name, child } => {
                format!("{} \"{}\"", child.to_string(), name)
            }
            TokenTree::FunctionCall { name, args } => {
                let mut r = name.clone() + "(";
                for arg in args {
                    r += &arg.to_string();
                    r += ", ";
                }
                r.pop();
                r.pop();
                r + ")"
            }
            TokenTree::VariableRef { name, exp } => {
                if *exp {
                    format!("!{}", name)
                } else {
                    name.clone()
                }
            }
            TokenTree::NumberLiteral(n) => n.clone(),
            TokenTree::Negate(child) => {
                format!("-{}", child)
            }
        };
        write!(f, "{}", r)
    }
}

pub struct TokenizationError(String);

impl Debug for TokenizationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "tokenizer error: {}", self.0)
    }
}

pub fn tokenize(source: &str) -> Result<TokenTree, TokenizationError> {
    let source_tokens = tokenize_source(source)?;
    let (tree, i) = gen_tree(&source_tokens, 0)?;
    if i == source_tokens.len()-1 {
        Ok(tree)
    } else {
        Err(TokenizationError("unexpected ) or ,".to_string()))
    }
}
#[derive(Eq, PartialEq)]
/// The most basic type of token, directly encodes source code.  
enum SourceToken {
    /// Any sequence of numeric chars.  
    Number(String),
    /// Any sequence of non-alphanumeric, non-whitespace chars.
    Operator(String),
    /// Any sequence of alphabetic chars.  
    Name(String),
    /// Any sequence of chars surrounded by "".  
    String(String),
    /// A parentheses, true means closing.
    Parentheses(bool),
}

fn gen_tree(expr: &[SourceToken], start: usize) -> Result<(TokenTree, usize), TokenizationError> {
    let is_end = |i: usize| {
        i >= expr.len()
            || expr[i] == SourceToken::Parentheses(true)
            || expr[i] == SourceToken::Operator(','.to_string())
    };
    let mut tokens = Vec::new();
    let mut operators = Vec::new();
    let mut expect_expr = true;
    let mut neg = false;
    let mut i = start;
    while !is_end(i) {
        if expect_expr {
            let n = neg;
            if expr[i] == SourceToken::Operator('-'.to_string()) {
                if neg {
                    return Err(TokenizationError(
                        "Double negation is not allowed".to_string(),
                    ));
                }
                neg = true;
                i += 1;
                continue;
            } else {
                neg = false;
            }
            let r = handle_expr(expr, &mut i)?;
            if n {
                tokens.push(TokenTree::Negate(Box::new(r)));
            } else {
                tokens.push(r);
            }
            expect_expr = false;
        } else {
            match &expr[i] {
                SourceToken::Operator(o) => {
                    operators.push(o.clone());
                    expect_expr = true;
                }
                SourceToken::Name(name) => {
                    let t = tokens
                        .pop()
                        .expect("program err: first iteration should always generate valid token");
                    tokens.push(TokenTree::DefinedUnit {
                        name: name.clone(),
                        child: Box::new(t),
                    });
                }
                SourceToken::String(name) => {
                    let t = tokens
                        .pop()
                        .expect("program err: first iteration should always generate valid token");
                    tokens.push(TokenTree::LiteralUnit {
                        name: name.clone(),
                        child: Box::new(t),
                    });
                }
                SourceToken::Number(n) => {
                    return Err(TokenizationError(format!(
                        "Expected unit or operator, got number {}",
                        n
                    )));
                }
                SourceToken::Parentheses(_) => {
                    return Err(TokenizationError(
                        "Expected unit or operator, got (".to_string(),
                    ));
                }
            }
        }

        i += 1;
    }
    i -= 1;
    if tokens.is_empty() {
        Err(TokenizationError("Expected expression".to_string()))
    } else if operators.len() != tokens.len() - 1 {
        Err(TokenizationError(
            "Expected expression after operator".to_string(),
        ))
    } else if operators.is_empty() {
        Ok((tokens.into_iter().next().unwrap(), i))
    } else {
        Ok((
            TokenTree::OperatorSequence {
                operators,
                children: tokens,
            },
            i,
        ))
    }
}

/// to handle expressions for gen_tree
fn handle_expr(expr: &[SourceToken], i: &mut usize) -> Result<TokenTree, TokenizationError> {
    match &expr[*i] {
        SourceToken::Number(num) => Ok(TokenTree::NumberLiteral(num.clone())),
        SourceToken::Operator(op) => {
            if op == "!" {
                // handle VarRef exp=true
                *i += 1;
                if let Some(SourceToken::Name(name)) = expr.get(*i) {
                    Ok(TokenTree::VariableRef {
                        name: name.clone(),
                        exp: true,
                    })
                } else {
                    Err(TokenizationError(
                        "expected variable name after '!'".to_string(),
                    ))
                }
            } else {
                Err(TokenizationError(format!(
                    "Expected expression, got operator '{}'",
                    op
                )))
            }
        }
        SourceToken::Name(name) => {
            if expr.get(*i + 1) == Some(&SourceToken::Operator("=".to_string())) {
                // handle VarAssign
                let (child, ii) = gen_tree(expr, *i + 2)?;
                *i = ii;
                Ok(TokenTree::VariableAssign {
                    name: name.clone(),
                    child: Box::new(child),
                })
            } else if let Some(SourceToken::Parentheses(false)) = expr.get(*i + 1) {
                // handle FuncCall
                let mut args = Vec::new();
                *i+=2;
                loop {
                    let (arg, ii) = gen_tree(expr, *i)?;
                    args.push(arg);
                    *i = ii+1;
                    if expr.get(*i) != Some(&SourceToken::Operator(','.to_string())) {
                        break;
                    }
                    *i += 1;
                }
                if expr.get(*i) != Some(&SourceToken::Parentheses(true)) {
                    return Err(TokenizationError(format!(
                        "Expected ) after function call '{}'",
                        name
                    )));
                }
                Ok(TokenTree::FunctionCall {
                    name: name.clone(),
                    args,
                })
            } else {
                // handle VarRef exp=false
                Ok(TokenTree::VariableRef {
                    name: name.clone(),
                    exp: false,
                })
            }
        }
        SourceToken::String(s) => Err(TokenizationError(format!(
            "Expected token, got string \"{}\"",
            s
        ))),
        SourceToken::Parentheses(v) => {
            // handle (
            // if closing then it will be caught by is_end
            assert!(!v);
            let (token, ii) = gen_tree(expr, *i + 1)?;
            *i = ii+1;
            if expr.get(*i) != Some(&SourceToken::Parentheses(true)) {
                Err(TokenizationError("Expected ) after (".to_string()))
            } else {
                Ok(token)
            }
        }
    }
}

fn tokenize_source(expr: &str) -> Result<Vec<SourceToken>, TokenizationError> {
    let mut tokens: Vec<SourceToken> = Vec::new();
    let mut current = None;
    // Takes the token as argument, to not perm borrow
    let mut push_token = |token: &mut Option<SourceToken>| {
        if let Some(token) = token.take() {
            tokens.push(token);
        }
    };
    for c in expr.chars() {
        // currently in string, overrides all
        if let Some(SourceToken::String(s)) = &mut current {
            match c {
                '"' => push_token(&mut current),
                _ => s.push(c),
            }
        } else if c.is_whitespace() {
            push_token(&mut current);
        } else if c == '"' {
            push_token(&mut current);
            current = Some(SourceToken::String(String::new()));
        } else if c.is_numeric() || c == '.' {
            if let Some(SourceToken::Number(num)) = &mut current {
                num.push(c);
            } else {
                push_token(&mut current);
                current = Some(SourceToken::Number(c.to_string()));
            }
        } else if c.is_alphabetic() {
            if let Some(SourceToken::Name(name)) = &mut current {
                name.push(c);
            } else {
                push_token(&mut current);
                current = Some(SourceToken::Name(c.to_string()));
            }
        } else if c == '(' || c == ')' {
            push_token(&mut current);
            current = Some(SourceToken::Parentheses(c == ')'));
        } else {
            if let Some(SourceToken::Operator(op)) = &mut current {
                op.push(c);
            } else {
                push_token(&mut current);
                current = Some(SourceToken::Operator(c.to_string()));
            }
        }
    }
    if let Some(SourceToken::String(_)) = &current {
        return Err(TokenizationError("Expected end of string".to_string()))
    }
    push_token(&mut current);
    Ok(tokens)
}
