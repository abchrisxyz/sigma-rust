//! Operators in ErgoTree

use eval::costs::Costs;
use num::CheckedAdd;
use num::CheckedDiv;
use num::CheckedMul;
use num::CheckedSub;
use num::Num;

use crate::eval;
use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::constant::TryExtractFrom;
use super::constant::TryExtractInto;
use super::expr::Expr;
use super::value::Value;

extern crate derive_more;
use derive_more::From;

#[cfg(feature = "arbitrary")]
use proptest_derive::Arbitrary;

/// Operations for numerical types
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub enum ArithOp {
    /// Addition
    Plus,
    /// Subtraction
    Minus,
    /// Multiplication
    Multiply,
    /// Division
    Divide,
    /// Max of two values
    Max,
    /// Min of two values
    Min,
}

impl From<ArithOp> for OpCode {
    fn from(op: ArithOp) -> Self {
        match op {
            ArithOp::Plus => OpCode::PLUS,
            ArithOp::Minus => OpCode::MINUS,
            ArithOp::Multiply => OpCode::MULTIPLY,
            ArithOp::Divide => OpCode::DIVISION,
            ArithOp::Max => OpCode::MAX,
            ArithOp::Min => OpCode::MIN,
        }
    }
}

/// Relational operations
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub enum RelationOp {
    /// Equality
    Eq,
    /// Non-equality
    NEq,
    /// Greater of equal
    GE,
    /// Greater than..
    GT,
    /// Less or equal
    LE,
    /// Less then
    LT,
    /// Logical AND
    And,
    /// Logical OR
    Or,
}

impl From<RelationOp> for OpCode {
    fn from(op: RelationOp) -> Self {
        match op {
            RelationOp::Eq => OpCode::EQ,
            RelationOp::NEq => OpCode::NEQ,
            RelationOp::GE => OpCode::GE,
            RelationOp::GT => OpCode::GT,
            RelationOp::LE => OpCode::LE,
            RelationOp::LT => OpCode::LT,
            RelationOp::And => OpCode::BIN_AND,
            RelationOp::Or => OpCode::BIN_OR,
        }
    }
}

/// Binary operations
#[derive(PartialEq, Eq, Debug, Clone, Copy, From)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub enum BinOpKind {
    /// Arithmetic operations
    Arith(ArithOp),
    /// Relation operations (equality, comparison, etc.)
    Relation(RelationOp),
}

impl From<BinOpKind> for OpCode {
    fn from(op: BinOpKind) -> Self {
        match op {
            BinOpKind::Arith(o) => o.into(),
            BinOpKind::Relation(o) => o.into(),
        }
    }
}

/// Binary operation
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BinOp {
    /// Operation kind
    pub kind: BinOpKind,
    /// Left operand
    pub left: Box<Expr>,
    /// Right operand
    pub right: Box<Expr>,
}

impl BinOp {
    /// Op code (serialization)
    pub fn op_code(&self) -> OpCode {
        self.kind.into()
    }

    /// Type
    pub fn tpe(&self) -> SType {
        match self.kind {
            BinOpKind::Relation(_) => SType::SBoolean,
            BinOpKind::Arith(_) => self.left.tpe(),
        }
    }
}

fn eval_plus<T>(lv_raw: T, rv: Value) -> Result<Value, EvalError>
where
    T: Num + CheckedAdd + TryExtractFrom<Value> + Into<Value> + std::fmt::Display,
{
    let rv_raw = rv.try_extract_into::<T>()?;
    Ok((lv_raw.checked_add(&rv_raw).ok_or_else(|| {
        EvalError::ArithmeticException(format!(
            "({0}) + ({1}) resulted in overflow",
            lv_raw, rv_raw
        ))
    })?)
    .into())
}

fn eval_minus<T>(lv_raw: T, rv: Value) -> Result<Value, EvalError>
where
    T: Num + CheckedSub + TryExtractFrom<Value> + Into<Value> + std::fmt::Display,
{
    let rv_raw = rv.try_extract_into::<T>()?;
    Ok((lv_raw.checked_sub(&rv_raw).ok_or_else(|| {
        EvalError::ArithmeticException(format!(
            "({0}) - ({1}) resulted in overflow",
            lv_raw, rv_raw
        ))
    })?)
    .into())
}

fn eval_mul<T>(lv_raw: T, rv: Value) -> Result<Value, EvalError>
where
    T: Num + CheckedMul + TryExtractFrom<Value> + Into<Value> + std::fmt::Display,
{
    let rv_raw = rv.try_extract_into::<T>()?;
    Ok((lv_raw.checked_mul(&rv_raw).ok_or_else(|| {
        EvalError::ArithmeticException(format!(
            "({0}) * ({1}) resulted in overflow",
            lv_raw, rv_raw
        ))
    })?)
    .into())
}

fn eval_div<T>(lv_raw: T, rv: Value) -> Result<Value, EvalError>
where
    T: Num + CheckedDiv + TryExtractFrom<Value> + Into<Value> + std::fmt::Display,
{
    let rv_raw = rv.try_extract_into::<T>()?;
    Ok((lv_raw.checked_div(&rv_raw).ok_or_else(|| {
        EvalError::ArithmeticException(format!(
            "({0}) / ({1}) resulted in exception",
            lv_raw, rv_raw
        ))
    })?)
    .into())
}

fn eval_ge(lv: Value, rv: Value) -> Result<Value, EvalError> {
    match lv {
        Value::Byte(lv_raw) => Ok((lv_raw >= rv.try_extract_into::<i8>()?).into()),
        Value::Short(lv_raw) => Ok((lv_raw >= rv.try_extract_into::<i16>()?).into()),
        Value::Int(lv_raw) => Ok((lv_raw >= rv.try_extract_into::<i32>()?).into()),
        Value::Long(lv_raw) => Ok((lv_raw >= rv.try_extract_into::<i64>()?).into()),
        Value::BigInt => todo!(),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected BinOp::left to be numeric value, got {0:?}",
            lv
        ))),
    }
}

fn eval_gt(lv: Value, rv: Value) -> Result<Value, EvalError> {
    match lv {
        Value::Byte(lv_raw) => Ok((lv_raw > rv.try_extract_into::<i8>()?).into()),
        Value::Short(lv_raw) => Ok((lv_raw > rv.try_extract_into::<i16>()?).into()),
        Value::Int(lv_raw) => Ok((lv_raw > rv.try_extract_into::<i32>()?).into()),
        Value::Long(lv_raw) => Ok((lv_raw > rv.try_extract_into::<i64>()?).into()),
        Value::BigInt => todo!(),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected BinOp::left to be numeric value, got {0:?}",
            lv
        ))),
    }
}

fn eval_lt(lv: Value, rv: Value) -> Result<Value, EvalError> {
    match lv {
        Value::Byte(lv_raw) => Ok((lv_raw < rv.try_extract_into::<i8>()?).into()),
        Value::Short(lv_raw) => Ok((lv_raw < rv.try_extract_into::<i16>()?).into()),
        Value::Int(lv_raw) => Ok((lv_raw < rv.try_extract_into::<i32>()?).into()),
        Value::Long(lv_raw) => Ok((lv_raw < rv.try_extract_into::<i64>()?).into()),
        Value::BigInt => todo!(),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected BinOp::left to be numeric value, got {0:?}",
            lv
        ))),
    }
}

fn eval_le(lv: Value, rv: Value) -> Result<Value, EvalError> {
    match lv {
        Value::Byte(lv_raw) => Ok((lv_raw <= rv.try_extract_into::<i8>()?).into()),
        Value::Short(lv_raw) => Ok((lv_raw <= rv.try_extract_into::<i16>()?).into()),
        Value::Int(lv_raw) => Ok((lv_raw <= rv.try_extract_into::<i32>()?).into()),
        Value::Long(lv_raw) => Ok((lv_raw <= rv.try_extract_into::<i64>()?).into()),
        Value::BigInt => todo!(),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected BinOp::left to be numeric value, got {0:?}",
            lv
        ))),
    }
}

fn eval_max<T>(lv_raw: T, rv: Value) -> Result<Value, EvalError>
where
    T: Num + Ord + TryExtractFrom<Value> + Into<Value>,
{
    let rv_raw = rv.try_extract_into::<T>()?;
    Ok((lv_raw.max(rv_raw)).into())
}

fn eval_min<T>(lv_raw: T, rv: Value) -> Result<Value, EvalError>
where
    T: Num + Ord + TryExtractFrom<Value> + Into<Value>,
{
    let rv_raw = rv.try_extract_into::<T>()?;
    Ok((lv_raw.min(rv_raw)).into())
}

impl Evaluable for BinOp {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        ctx.cost_accum.add(Costs::DEFAULT.eq_const_size)?;
        let lv = self.left.eval(env, ctx)?;
        // using closure to keep right value from evaluation (for lazy AND, OR)
        let mut rv = || self.right.eval(env, ctx);
        match self.kind {
            BinOpKind::Relation(op) => match op {
                RelationOp::Eq => Ok(Value::Boolean(lv == rv()?)),
                RelationOp::NEq => Ok(Value::Boolean(lv != rv()?)),
                RelationOp::GT => eval_gt(lv, rv()?),
                RelationOp::LT => eval_lt(lv, rv()?),
                RelationOp::GE => eval_ge(lv, rv()?),
                RelationOp::LE => eval_le(lv, rv()?),
                RelationOp::And => Ok(Value::Boolean(if lv.try_extract_into::<bool>()? {
                    rv()?.try_extract_into::<bool>()?
                } else {
                    false
                })),
                RelationOp::Or => Ok(Value::Boolean(if !lv.try_extract_into::<bool>()? {
                    rv()?.try_extract_into::<bool>()?
                } else {
                    true
                })),
            },
            BinOpKind::Arith(op) => match op {
                ArithOp::Plus => match lv {
                    Value::Byte(lv_raw) => eval_plus(lv_raw, rv()?),
                    Value::Short(lv_raw) => eval_plus(lv_raw, rv()?),
                    Value::Int(lv_raw) => eval_plus(lv_raw, rv()?),
                    Value::Long(lv_raw) => eval_plus(lv_raw, rv()?),
                    Value::BigInt => todo!(),
                    _ => Err(EvalError::UnexpectedValue(format!(
                        "expected BinOp::left to be numeric value, got {0:?}",
                        lv
                    ))),
                },
                ArithOp::Minus => match lv {
                    Value::Byte(lv_raw) => eval_minus(lv_raw, rv()?),
                    Value::Short(lv_raw) => eval_minus(lv_raw, rv()?),
                    Value::Int(lv_raw) => eval_minus(lv_raw, rv()?),
                    Value::Long(lv_raw) => eval_minus(lv_raw, rv()?),
                    Value::BigInt => todo!(),
                    _ => Err(EvalError::UnexpectedValue(format!(
                        "expected BinOp::left to be numeric value, got {0:?}",
                        lv
                    ))),
                },
                ArithOp::Multiply => match lv {
                    Value::Byte(lv_raw) => eval_mul(lv_raw, rv()?),
                    Value::Short(lv_raw) => eval_mul(lv_raw, rv()?),
                    Value::Int(lv_raw) => eval_mul(lv_raw, rv()?),
                    Value::Long(lv_raw) => eval_mul(lv_raw, rv()?),
                    Value::BigInt => todo!(),
                    _ => Err(EvalError::UnexpectedValue(format!(
                        "expected BinOp::left to be numeric value, got {0:?}",
                        lv
                    ))),
                },
                ArithOp::Divide => match lv {
                    Value::Byte(lv_raw) => eval_div(lv_raw, rv()?),
                    Value::Short(lv_raw) => eval_div(lv_raw, rv()?),
                    Value::Int(lv_raw) => eval_div(lv_raw, rv()?),
                    Value::Long(lv_raw) => eval_div(lv_raw, rv()?),
                    Value::BigInt => todo!(),
                    _ => Err(EvalError::UnexpectedValue(format!(
                        "expected BinOp::left to be numeric value, got {0:?}",
                        lv
                    ))),
                },
                ArithOp::Max => match lv {
                    Value::Byte(lv_raw) => eval_max(lv_raw, rv()?),
                    Value::Short(lv_raw) => eval_max(lv_raw, rv()?),
                    Value::Int(lv_raw) => eval_max(lv_raw, rv()?),
                    Value::Long(lv_raw) => eval_max(lv_raw, rv()?),
                    Value::BigInt => todo!(),
                    _ => Err(EvalError::UnexpectedValue(format!(
                        "expected BinOp::left to be numeric value, got {0:?}",
                        lv
                    ))),
                },
                ArithOp::Min => match lv {
                    Value::Byte(lv_raw) => eval_min(lv_raw, rv()?),
                    Value::Short(lv_raw) => eval_min(lv_raw, rv()?),
                    Value::Int(lv_raw) => eval_min(lv_raw, rv()?),
                    Value::Long(lv_raw) => eval_min(lv_raw, rv()?),
                    Value::BigInt => todo!(),
                    _ => Err(EvalError::UnexpectedValue(format!(
                        "expected BinOp::left to be numeric value, got {0:?}",
                        lv
                    ))),
                },
            },
        }
    }
}

#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for BinOp {
        type Parameters = ArbExprParams;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            match args.tpe {
                SType::SBoolean => (
                    any::<RelationOp>().prop_map_into(),
                    any_with::<Expr>(ArbExprParams {
                        tpe: SType::SAny,
                        depth: args.depth,
                    }),
                    any_with::<Expr>(ArbExprParams {
                        tpe: SType::SAny,
                        depth: args.depth,
                    }),
                )
                    .prop_map(|(kind, left, right)| BinOp {
                        kind,
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                    .boxed(),

                SType::SInt => (
                    any::<ArithOp>().prop_map_into(),
                    any_with::<Expr>(ArbExprParams {
                        tpe: SType::SInt,
                        depth: args.depth,
                    }),
                    any_with::<Expr>(ArbExprParams {
                        tpe: SType::SInt,
                        depth: args.depth,
                    }),
                )
                    .prop_map(|(kind, left, right)| BinOp {
                        kind,
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                    .boxed(),

                _ => (
                    any::<BinOpKind>(),
                    any_with::<Expr>(ArbExprParams {
                        tpe: SType::SAny,
                        depth: args.depth,
                    }),
                    any_with::<Expr>(ArbExprParams {
                        tpe: SType::SAny,
                        depth: args.depth,
                    }),
                )
                    .prop_map(|(kind, left, right)| BinOp {
                        kind,
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                    .boxed(),
                // SType::SByte => {}
                // SType::SShort => {}
                // SType::SInt => {}
                // SType::SLong => {}
                // SType::SBigInt => {}
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::eval::tests::try_eval_out;
    use crate::mir::constant::Constant;
    use crate::mir::constant::TryExtractFrom;
    use crate::test_util::force_any_val;

    use super::*;

    fn check_eq_neq(left: Constant, right: Constant) -> bool {
        let eq_op: Expr = BinOp {
            kind: BinOpKind::Relation(RelationOp::Eq),
            left: Box::new(left.clone().into()),
            right: Box::new(right.clone().into()),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        let neq_op: Expr = BinOp {
            kind: BinOpKind::Relation(RelationOp::NEq),
            left: Box::new(left.into()),
            right: Box::new(right.into()),
        }
        .into();
        let ctx1 = Rc::new(force_any_val::<Context>());
        eval_out::<bool>(&eq_op, ctx) && !eval_out::<bool>(&neq_op, ctx1)
    }

    #[test]
    fn num_eq() {
        assert!(check_eq_neq(1i64.into(), 1i64.into()));
    }

    #[test]
    fn num_neq() {
        assert!(!check_eq_neq(2i64.into(), 1i64.into()));
    }

    #[test]
    fn option_eq() {
        assert!(check_eq_neq(Some(1i64).into(), Some(1i64).into()));
        let none: Option<i64> = None;
        assert!(check_eq_neq(none.into(), none.into()));
        // Option<Vec<i8>>
        assert!(check_eq_neq(
            Some(vec![1i8, 2i8]).into(),
            Some(vec![1i8, 2i8]).into()
        ));
        // Vec<Option<i64>>
        assert!(check_eq_neq(
            vec![Some(1i64), Some(1i64)].into(),
            vec![Some(1i64), Some(1i64)].into()
        ));
    }

    #[test]
    fn option_neq() {
        assert!(!check_eq_neq(Some(2i64).into(), Some(1i64).into()));
        let none: Option<i64> = None;
        assert!(!check_eq_neq(none.into(), Some(1i64).into()));
        // Option<Vec<i8>>
        assert!(!check_eq_neq(
            Some(vec![1i8, 2i8]).into(),
            Some(vec![2i8, 2i8]).into()
        ));
        // Vec<Option<i64>>
        assert!(!check_eq_neq(
            vec![Some(1i64), Some(1i64)].into(),
            vec![Some(2i64), Some(1i64)].into()
        ));
    }

    #[test]
    fn tuple_eq() {
        assert!(check_eq_neq((1i64, true).into(), (1i64, true).into()));
    }

    #[test]
    fn bin_or_eval_laziness() {
        let e: Expr = BinOp {
            kind: BinOpKind::Relation(RelationOp::Or),
            left: Box::new(Expr::Const(true.into())),
            // something that should blow-up the evaluation
            right: Box::new(
                BinOp {
                    kind: ArithOp::Divide.into(),
                    left: Box::new(Expr::Const(1i32.into())),
                    right: Box::new(Expr::Const(0i32.into())),
                }
                .into(),
            ),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(eval_out::<bool>(&e, ctx), true);
    }

    #[test]
    fn bin_and_eval_laziness() {
        let e: Expr = BinOp {
            kind: BinOpKind::Relation(RelationOp::And),
            left: Box::new(Expr::Const(false.into())),
            // something that should blow-up the evaluation
            right: Box::new(
                BinOp {
                    kind: ArithOp::Divide.into(),
                    left: Box::new(Expr::Const(1i32.into())),
                    right: Box::new(Expr::Const(0i32.into())),
                }
                .into(),
            ),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(eval_out::<bool>(&e, ctx), false);
    }

    fn eval_num_op<T: TryExtractFrom<Value>>(
        op: ArithOp,
        left: Constant,
        right: Constant,
    ) -> Result<T, EvalError> {
        let expr: Expr = BinOp {
            kind: BinOpKind::Arith(op),
            left: Box::new(left.into()),
            right: Box::new(right.into()),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        try_eval_out::<T>(&expr, ctx)
    }

    fn eval_relation_op(op: RelationOp, left: Constant, right: Constant) -> bool {
        let expr: Expr = BinOp {
            kind: BinOpKind::Relation(op),
            left: Box::new(left.into()),
            right: Box::new(right.into()),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        eval_out::<bool>(&expr, ctx)
    }

    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(32))]

        #[test]
        fn test_eq(v in any::<Constant>()) {
            prop_assert![check_eq_neq(v.clone(), v)];
        }

        #[test]
        fn test_num_slong(l in any::<i64>(), r in any::<i64>()) {
            prop_assert_eq!(eval_num_op(ArithOp::Plus, l.into(), r.into()).ok(), l.checked_add(r));
            prop_assert_eq!(eval_num_op(ArithOp::Minus, l.into(), r.into()).ok(), l.checked_sub(r));
            prop_assert_eq!(eval_num_op(ArithOp::Multiply, l.into(), r.into()).ok(), l.checked_mul(r));
            prop_assert_eq!(eval_num_op(ArithOp::Divide, l.into(), r.into()).ok(), l.checked_div(r));
            prop_assert_eq!(eval_num_op::<i64>(ArithOp::Max, l.into(), r.into()).unwrap(), l.max(r));
            prop_assert_eq!(eval_num_op::<i64>(ArithOp::Min, l.into(), r.into()).unwrap(), l.min(r));

            prop_assert_eq!(eval_relation_op(RelationOp::GT, l.into(), r.into()), l > r);
            prop_assert_eq!(eval_relation_op(RelationOp::LT, l.into(), r.into()), l < r);
            prop_assert_eq!(eval_relation_op(RelationOp::GE, l.into(), r.into()), l >= r);
            prop_assert_eq!(eval_relation_op(RelationOp::LE, l.into(), r.into()), l <= r);
        }

        #[test]
        fn test_num_sint(l in any::<i32>(), r in any::<i32>()) {
            prop_assert_eq!(eval_num_op(ArithOp::Plus, l.into(), r.into()).ok(), l.checked_add(r));
            prop_assert_eq!(eval_num_op(ArithOp::Minus, l.into(), r.into()).ok(), l.checked_sub(r));
            prop_assert_eq!(eval_num_op(ArithOp::Multiply, l.into(), r.into()).ok(), l.checked_mul(r));
            prop_assert_eq!(eval_num_op(ArithOp::Divide, l.into(), r.into()).ok(), l.checked_div(r));
            prop_assert_eq!(eval_num_op::<i32>(ArithOp::Max, l.into(), r.into()).unwrap(), l.max(r));
            prop_assert_eq!(eval_num_op::<i32>(ArithOp::Min, l.into(), r.into()).unwrap(), l.min(r));

            prop_assert_eq!(eval_relation_op(RelationOp::GT, l.into(), r.into()), l > r);
            prop_assert_eq!(eval_relation_op(RelationOp::LT, l.into(), r.into()), l < r);
            prop_assert_eq!(eval_relation_op(RelationOp::GE, l.into(), r.into()), l >= r);
            prop_assert_eq!(eval_relation_op(RelationOp::LE, l.into(), r.into()), l <= r);
        }

        #[test]
        fn test_num_sshort(l in any::<i16>(), r in any::<i16>()) {
            prop_assert_eq!(eval_num_op(ArithOp::Plus, l.into(), r.into()).ok(), l.checked_add(r));
            prop_assert_eq!(eval_num_op(ArithOp::Minus, l.into(), r.into()).ok(), l.checked_sub(r));
            prop_assert_eq!(eval_num_op(ArithOp::Multiply, l.into(), r.into()).ok(), l.checked_mul(r));
            prop_assert_eq!(eval_num_op(ArithOp::Divide, l.into(), r.into()).ok(), l.checked_div(r));
            prop_assert_eq!(eval_num_op::<i16>(ArithOp::Max, l.into(), r.into()).unwrap(), l.max(r));
            prop_assert_eq!(eval_num_op::<i16>(ArithOp::Min, l.into(), r.into()).unwrap(), l.min(r));

            prop_assert_eq!(eval_relation_op(RelationOp::GT, l.into(), r.into()), l > r);
            prop_assert_eq!(eval_relation_op(RelationOp::LT, l.into(), r.into()), l < r);
            prop_assert_eq!(eval_relation_op(RelationOp::GE, l.into(), r.into()), l >= r);
            prop_assert_eq!(eval_relation_op(RelationOp::LE, l.into(), r.into()), l <= r);
        }

        #[test]
        fn test_num_sbyte(l in any::<i8>(), r in any::<i8>()) {
            prop_assert_eq!(eval_num_op(ArithOp::Plus, l.into(), r.into()).ok(), l.checked_add(r));
            prop_assert_eq!(eval_num_op(ArithOp::Minus, l.into(), r.into()).ok(), l.checked_sub(r));
            prop_assert_eq!(eval_num_op(ArithOp::Multiply, l.into(), r.into()).ok(), l.checked_mul(r));
            prop_assert_eq!(eval_num_op(ArithOp::Divide, l.into(), r.into()).ok(), l.checked_div(r));
            prop_assert_eq!(eval_num_op::<i8>(ArithOp::Max, l.into(), r.into()).unwrap(), l.max(r));
            prop_assert_eq!(eval_num_op::<i8>(ArithOp::Min, l.into(), r.into()).unwrap(), l.min(r));

            prop_assert_eq!(eval_relation_op(RelationOp::GT, l.into(), r.into()), l > r);
            prop_assert_eq!(eval_relation_op(RelationOp::LT, l.into(), r.into()), l < r);
            prop_assert_eq!(eval_relation_op(RelationOp::GE, l.into(), r.into()), l >= r);
            prop_assert_eq!(eval_relation_op(RelationOp::LE, l.into(), r.into()), l <= r);
        }

        #[test]
        fn test_and_or(l in any::<bool>(), r in any::<bool>()) {
            prop_assert_eq!(eval_relation_op(RelationOp::And, l.into(), r.into()), l && r);
            prop_assert_eq!(eval_relation_op(RelationOp::Or, l.into(), r.into()), l || r);
        }

    }
}