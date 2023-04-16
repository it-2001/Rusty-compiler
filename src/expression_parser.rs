use core::panic;

use crate::intermediate::dictionary::*;
use crate::intermediate::AnalyzationError::ErrType;
use crate::lexer::tokenizer::{Operators, Tokens};
use crate::tree_walker::tree_walker::Node;
use crate::{intermediate, lexer};
use intermediate::dictionary::*;
use intermediate::*;

pub fn expr_into_tree(node: &Node, errors: &mut Vec<ErrType>) -> ExprNode {
    let mut expr = ExprNode {
        left: None,
        right: None,
        operator: None,
    };
    let nodes = step_inside_arr(&node, "nodes");
    if nodes.len() == 0 {
        return expr;
    }
    if let Tokens::Text(str) = &nodes[0].name {
        if str == "anonymous_function" {
            expr.left = Some(ValueType::AnonymousFunction(get_fun_siginifier(
                &nodes[0], errors,
            )));
            return expr;
        }
    }
    let transform = transform_expr(&nodes, errors);
    println!("transformed: {:?}", transform);
    for nd in nodes {}
    expr
}

pub fn transform_expr(nodes: &Vec<Node>, errors: &mut Vec<ErrType>) -> Vec<ValueType> {
    let mut result = vec![];
    for node in nodes {
        if let Some(op) = try_get_op(&node, errors) {
            result.push(ValueType::Operator(op));
            continue;
        }
        if let Some(val) = try_get_value(&node, errors) {
            result.push(val);
            continue;
        }
    }
    result
}

pub fn try_get_value(node: &Node, errors: &mut Vec<ErrType>) -> Option<ValueType> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "value" {
            return None;
        }
    }
    let prepend = get_prepend(step_inside_val(&node, "prepend"), errors);
    if let Some(car) = try_get_variable(step_inside_val(&node, "value"), errors) {
        return Some(ValueType::Value(Variable {
            unary: prepend.2,
            refs: prepend.0,
            modificatior: prepend.1,
            root: car.0,
            tail: car.1,
        }));
    }
    if let Some(lit) = try_get_literal(step_inside_val(&node, "value"), errors, &prepend) {
        return Some(ValueType::Literal(lit));
    }
    None
}

pub fn try_get_literal(node: &Node, errors: &mut Vec<ErrType>, prepend: &(usize, Option<String>, Option<Operators>)) -> Option<Literal> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "literal" {
            return None;
        }
    }
    let this = step_inside_val(&node, "value");
    if let Tokens::Number(_, _, _) = &this.name {
        return Some(Literal {
            unary: prepend.2,
            refs: prepend.0,
            modificatior: prepend.1.clone(),
            value: Literals::Number(this.name.clone()),
        });
    }
    if let Tokens::String(str) = &this.name {
        return Some(Literal {
            unary: prepend.2,
            refs: prepend.0,
            modificatior: prepend.1.clone(),
            value: Literals::String(str.clone()),
        });
    }
    None
}

pub fn try_get_variable(
    node: &Node,
    errors: &mut Vec<ErrType>,
) -> Option<(String, Vec<TailNodes>)> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "variable" {
            return None;
        }
    }
    let ident = get_ident(&node);
    let tail = get_tail(step_inside_val(&node, "tail"), errors);
    println!("tail: {:?}", tail);
    Some((ident, tail))
}

pub fn get_args(node: &Node, errors: &mut Vec<ErrType>) -> Vec<ExprNode> {
    let mut result = vec![];
    // TODO: implement
    result
}

pub fn get_tail(node: &Node, errors: &mut Vec<ErrType>) -> Vec<TailNodes> {
    let mut tail = vec![];
    for child in step_inside_arr(&node, "nodes") {
        if let Tokens::Text(txt) = &child.name {
            if txt == "idx" {
                let expr = expr_into_tree(step_inside_val(&child, "expression"), errors);
                tail.push(TailNodes::Index(expr));
                continue;
            }
            if txt == "nested" {
                tail.push(TailNodes::Nested(get_ident(&child)));
                continue;
            }
            if txt == "function_call" {
                let generic = get_generics_expr(&child, errors);
                let args = get_args(&child, errors);
                tail.push(TailNodes::Call(FunctionCall { generic, args }));
                continue;
            }
            if txt == "cast" {
                let kind = get_nested_ident(step_inside_val(&child, "type"), errors);
                tail.push(TailNodes::Cast(kind));
                break;
            }
        }
    }
    tail
}

pub fn get_prepend(
    node: &Node,
    errors: &mut Vec<ErrType>,
) -> (usize, Option<String>, Option<Operators>) {
    let refs = count_refs(&node);
    let modificator = if let Tokens::Text(txt) = &step_inside_val(&node, "keywords").name {
        if txt == "'none" {
            None
        }else {
            Some(txt.to_string())
        }
    } else {
        None
    };
    let unary = if let Some(un) = try_step_inside_val(&step_inside_val(&node, "unary"), "op") {
        if let Tokens::Operator(op) = un.name {
            Some(op)
        } else {
            None
        }
    } else {
        None
    };
    (refs, modificator, unary)
}

pub fn try_get_op(node: &Node, errors: &mut Vec<ErrType>) -> Option<Operators> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "operator" {
            return None;
        }
    }
    if let Tokens::Operator(op) = step_inside_val(&node, "op").name {
        return Some(op);
    }
    None
}

#[derive(Debug)]
pub struct ExprNode {
    left: Option<ValueType>,
    right: Option<ValueType>,
    operator: Option<Operators>,
}
#[derive(Debug)]
pub enum ValueType {
    Literal(Literal),
    AnonymousFunction(Function),
    /// parenthesis
    Expression(Box<ExprNode>),
    /// only for inner functionality
    Operator(Operators),
    Value(Variable),
}
impl ValueType {
    pub fn fun(fun: Function) -> ValueType {
        ValueType::AnonymousFunction(fun)
    }
    pub fn value(val: Literal) -> ValueType {
        ValueType::Literal(val)
    }
}
#[derive(Debug)]
pub struct Literal {
    unary: Option<Operators>,
    refs: usize,
    /// atm only keyword new, so bool would be sufficient, but who knows what will be in the future updates
    modificatior: Option<String>,
    value: Literals,
}
#[derive(Debug)]
pub enum Literals {
    Number(Tokens),
    Array(ArrayRule),
    String(String),
}
#[derive(Debug)]
pub enum ArrayRule {
    Fill(Box<ExprNode>, usize),
    Explicit(Vec<ExprNode>),
}
#[derive(Debug)]
pub struct Variable {
    unary: Option<Operators>,
    refs: usize,
    /// atm only keyword new, so bool would be sufficient, but who knows what will be in the future updates
    modificatior: Option<String>,
    /// for longer variables
    /// example: danda[5].touch_grass(9)
    ///          ~~~~~ <- this is considered a root
    root: String,
    /// for longer variables
    /// example: danda[5].touch_grass(9)
    /// danda is root .. rest is tail
    tail: Vec<TailNodes>,
}

#[derive(Debug)]
pub struct FunctionCall {
    generic: Vec<ShallowType>,
    args: Vec<ExprNode>,
}
#[derive(Debug)]
pub enum TailNodes {
    Nested(String),
    Index(ExprNode),
    Call(FunctionCall),
    Cast(Vec<String>),
}

pub fn try_is_operator(node: &Node, errors: &mut Vec<ErrType>) -> Option<Operators> {
    if let Tokens::Operator(op) = &node.name {
        return Some(*op);
    }
    None
}
