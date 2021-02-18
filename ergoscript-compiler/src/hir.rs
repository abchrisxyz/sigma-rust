//! High-level Intermediate Representation
//! Refered as frontend representation in sigmastate

mod rewrite;

use ergo_lib::types::stype::SType;
pub use rewrite::rewrite;

use super::ast;
use crate::syntax::SyntaxKind;
use text_size::TextRange;

extern crate derive_more;
use derive_more::From;

pub fn lower(ast: ast::Root) -> Result<Expr, HirLoweringError> {
    // TODO: return error if more than one expr is found
    let first_expr = ast.children().next().unwrap();
    Expr::lower(&first_expr)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: TextRange,
    pub tpe: Option<SType>,
}

// TODO: refine: span, expected, found?
#[derive(Debug, PartialEq)]
pub struct HirLoweringError(pub String);

impl Expr {
    pub fn lower(expr: &ast::Expr) -> Result<Expr, HirLoweringError> {
        match expr {
            ast::Expr::BinaryExpr(ast) => Ok(Expr {
                kind: Binary::lower(ast)?.into(),
                span: ast.span(),
                tpe: None,
            }),
            ast::Expr::Ident(ast) => ast
                .name()
                .map(|node| Expr {
                    kind: ExprKind::Ident(node.text().to_string()),
                    span: ast.span(),
                    tpe: None,
                })
                .ok_or_else(|| HirLoweringError("".to_string())),
            _ => todo!("{0:?}", expr),
        }
    }

    pub fn debug_tree(&self) -> String {
        let tree = format!("{:#?}", self);
        tree
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Spanned<T: Clone> {
    pub node: T,
    pub span: TextRange,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Binary {
    pub op: Spanned<BinaryOp>,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

impl Binary {
    fn lower(ast: &ast::BinaryExpr) -> Result<Binary, HirLoweringError> {
        // TODO: unwraps -> errors
        let op = match ast.op().unwrap().kind() {
            SyntaxKind::Plus => BinaryOp::Plus,
            SyntaxKind::Minus => BinaryOp::Minus,
            SyntaxKind::Star => BinaryOp::Multiply,
            SyntaxKind::Slash => BinaryOp::Divide,
            _ => unreachable!(),
        };

        let lhs = Expr::lower(&ast.lhs().unwrap());
        let rhs = Expr::lower(&ast.rhs().unwrap());

        Ok(Binary {
            op: Spanned {
                node: op,
                span: ast.op().unwrap().text_range(),
            },
            lhs: Box::new(lhs?),
            rhs: Box::new(rhs?),
        })
    }
}

#[derive(Debug, PartialEq, From, Clone)]
pub enum ExprKind {
    Ident(String),
    Binary(Binary),
    GlobalVars(GlobalVars),
    // ...
    // Block
    // ValNode
    // Select
    // ApplyTypes
    // MethodCallLike
    // Lambda
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Plus,
    Minus,
    Multiply,
    Divide,
}

// #[derive(Debug, PartialEq, Clone)]
// pub enum UnaryOp {
//     Neg,
// }

#[derive(Debug, PartialEq, Clone)]
pub enum GlobalVars {
    Height,
}

impl GlobalVars {
    pub fn tpe(&self) -> SType {
        match self {
            GlobalVars::Height => SType::SInt,
        }
    }
}
