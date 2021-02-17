use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::constant::TryExtractInto;
use super::expr::Expr;
use super::value::Value;

/// If (lazy)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct If {
    condition: Box<Expr>,
    true_branch: Box<Expr>,
    false_branch: Box<Expr>,
}

impl If {
    pub const OP_CODE: OpCode = OpCode::IF;

    pub fn tpe(&self) -> SType {
        self.true_branch.tpe()
    }

    pub fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl Evaluable for If {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let condition_v = self.condition.eval(env, ctx)?;
        if condition_v.try_extract_into::<bool>()? {
            self.true_branch.eval(env, ctx)
        } else {
            self.false_branch.eval(env, ctx)
        }
    }
}

impl SigmaSerializable for If {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.condition.sigma_serialize(w)?;
        self.true_branch.sigma_serialize(w)?;
        self.false_branch.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let condition = Expr::sigma_parse(r)?.into();
        let true_branch = Expr::sigma_parse(r)?.into();
        let false_branch = Expr::sigma_parse(r)?.into();
        Ok(Self {
            condition,
            true_branch,
            false_branch,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::ast::bin_op::ArithOp;
    use crate::ast::bin_op::BinOp;
    use crate::ast::expr::tests::ArbExprParams;
    use crate::ast::expr::Expr;
    use crate::eval::tests::eval_out_wo_ctx;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    use proptest::prelude::*;

    impl Arbitrary for If {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SBoolean,
                    depth: 2,
                }),
                any::<Expr>(),
            )
                .prop_map(|(condition, true_branch)| Self {
                    condition: condition.into(),
                    true_branch: true_branch.clone().into(),
                    false_branch: true_branch.into(),
                })
                .boxed()
        }
    }

    #[test]
    fn eval() {
        let expr: Expr = If {
            condition: Expr::Const(true.into()).into(),
            true_branch: Expr::Const(1i64.into()).into(),
            false_branch: Expr::Const(2i64.into()).into(),
        }
        .into();
        let res = eval_out_wo_ctx::<i64>(&expr);
        assert_eq!(res, 1);
    }

    #[test]
    fn eval_laziness_true_branch() {
        let expr: Expr = If {
            condition: Expr::Const(true.into()).into(),
            true_branch: Expr::Const(1i64.into()).into(),
            false_branch: Box::new(
                BinOp {
                    kind: ArithOp::Divide.into(),
                    left: Box::new(Expr::Const(1i64.into())),
                    right: Box::new(Expr::Const(0i64.into())),
                }
                .into(),
            ),
        }
        .into();
        let res = eval_out_wo_ctx::<i64>(&expr);
        assert_eq!(res, 1);
    }

    #[test]
    fn eval_laziness_false_branch() {
        let expr: Expr = If {
            condition: Expr::Const(false.into()).into(),
            true_branch: Box::new(
                BinOp {
                    kind: ArithOp::Divide.into(),
                    left: Box::new(Expr::Const(1i64.into())),
                    right: Box::new(Expr::Const(0i64.into())),
                }
                .into(),
            ),
            false_branch: Expr::Const(1i64.into()).into(),
        }
        .into();
        let res = eval_out_wo_ctx::<i64>(&expr);
        assert_eq!(res, 1);
    }

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<If>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}