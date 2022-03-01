use crate::error::*;
use crate::expr::*;
use crate::object::*;
use crate::token_type::*;

pub struct Interpreter {}

impl ExprVisitor<Object> for Interpreter {
    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<Object, LoxError> {
        Ok(expr.value.clone().unwrap())
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<Object, LoxError> {
        Ok(self.evaluate(&expr.expression)?)
    }

    fn visit_binary_expr(&self, _expr: &BinaryExpr) -> Result<Object, LoxError> {
        Ok(Object::Nil)
    }

    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<Object, LoxError> {
        let right = self.evaluate(&expr.right)?;

        match expr.operator.token_type() {
            TokenType::Minus => match right {
                Object::Num(n) => return Ok(Object::Num(-n)),
                _ => return Ok(Object::Nil),
            },
            TokenType::Bang => Ok(Object::Bool(!self.is_truthy(&right))),
            _ => Err(LoxError::error(0, "Unreachable accordin to Nystrom")),
        }
    }
}

impl Interpreter {
    fn evaluate(&self, expr: &Expr) -> Result<Object, LoxError> {
        expr.accept(self)
    }

    // Anything that is not Nil or False is true
    fn is_truthy(&self, object: &Object) -> bool {
        !matches!(object, Object::Nil | Object::Bool(false))
    }
}
