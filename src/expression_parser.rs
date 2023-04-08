use crate::{lexer, intermediate};
use intermediate::*;
use crate::intermediate::AnalyzationError::ErrType;
use crate::intermediate::dictionary::*;
use crate::lexer::tokenizer::Tokens;
use crate::tree_walker::tree_walker::Node;


/*fn expr_into_tree(node: &Node, errors: &mut Vec<ErrType>) -> ExprNode {
    
}*/


/*pub struct ExprNode {
    left: Value,
    right: Value,
    operator: Operator,
}
pub struct Value {
    unary: Vec<Tokens>,
    value: ValueType,


}
pub enum Operator {
    
}
pub enum ValueType {
    String(String),
    Number(Tokens),
    Variable(Variable),
}
pub struct Variable {
    refs: usize,
    /// atm only keyword new, so bool would be sufficient, but who knows what will be in the future updates
    modificatior: String,
    /// for longer variables
    /// example: danda[5].touch_grass(9)
    ///          ~~~~~ <- this is considered a root
    root: String,
    /// for longer variables
    /// example: danda[5].touch_grass(9)
    /// if danda is root, then rest is tail
    tail: Vec<TailNodes>
}
pub enum TailNodes {
    Nested(String),
    Index(ExprNode),
    Call(todo!()),
    Cast(ShallowType),
}*/