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

    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<Object, LoxError> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        let result = match expr.operator.token_type() {
            TokenType::Minus => left - right,
            TokenType::Slash => left / right,
            TokenType::Star => left * right,
            TokenType::Plus => left + right,
            TokenType::Greater => Object::Bool(left > right),
            TokenType::GreaterEqual => Object::Bool(left >= right),
            TokenType::Less => Object::Bool(left < right),
            TokenType::LessEqual => Object::Bool(left <= right),
            TokenType::BangEqual => Object::Bool(left != right),
            TokenType::Equals => Object::Bool(left == right),
            _ => {
                todo!("need to work on your code dude");
            }
        };

        if result == Object::ArithmeticError {
            Err(LoxError::error(expr.operator.line, "Illegal expression"))
        } else {
            Ok(result)
        }
    }

    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<Object, LoxError> {
        let right = self.evaluate(&expr.right)?;

        match expr.operator.token_type() {
            TokenType::Minus => match right {
                Object::Num(n) => return Ok(Object::Num(-n)),
                _ => return Ok(Object::Nil),
            },
            TokenType::Bang => Ok(Object::Bool(!self.is_truthy(&right))),
            _ => Err(LoxError::error(
                expr.operator.line,
                "Unreachable according to Nystrom",
            )),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::*;

    fn make_literal(o: Object) -> Box<Expr> {
        Box::new(Expr::Literal(LiteralExpr { value: Some(o) }))
    }

    fn make_literal_string(s: &str) -> Box<Expr> {
        make_literal(Object::Str(s.to_string()))
    }

    #[test]
    fn test_unary_minus() {
        let terp = Interpreter {};
        let unary_expr = UnaryExpr {
            operator: Token::new(TokenType::Minus, "-".to_string(), None, 123),
            right: make_literal(Object::Num(123.0)),
        };
        let result = terp.visit_unary_expr(&unary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(-123.0)));
    }

    #[test]
    fn test_unary_not() {
        let terp = Interpreter {};
        let unary_expr = UnaryExpr {
            operator: Token::new(TokenType::Bang, "!".to_string(), None, 123),
            right: make_literal(Object::Bool(false)),
        };
        let result = terp.visit_unary_expr(&unary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_subtraction() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::Minus, "-".to_string(), None, 123),
            right: make_literal(Object::Num(7.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(8.0)));
    }

    #[test]
    fn test_multiplication() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::Star, "*".to_string(), None, 123),
            right: make_literal(Object::Num(7.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(105.0)));
    }

    #[test]
    fn test_division() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(21.0)),
            operator: Token::new(TokenType::Slash, "/".to_string(), None, 123),
            right: make_literal(Object::Num(7.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(3.0)));
    }

    #[test]
    fn test_addition() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(21.0)),
            operator: Token::new(TokenType::Plus, "+".to_string(), None, 123),
            right: make_literal(Object::Num(7.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(28.0)));
    }

    #[test]
    fn test_string_concatination() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal_string("hello, "),
            operator: Token::new(TokenType::Plus, "+".to_string(), None, 123),
            right: make_literal_string("world!"),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Str("hello, world!".to_string())));
    }

    #[test]
    fn test_arithmetic_error_for_subtration() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::Minus, "-".to_string(), None, 123),
            right: make_literal(Object::Bool(true)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_greater_than_equal_to() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::GreaterEqual, ">=".to_string(), None, 123),
            right: make_literal(Object::Num(15.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_greater_than_true() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::Greater, ">".to_string(), None, 123),
            right: make_literal(Object::Num(7.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_greater_than_false() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::Greater, ">".to_string(), None, 123),
            right: make_literal(Object::Num(17.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(false)));
    }

    #[test]
    fn test_equals() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::Equals, "==".to_string(), None, 123),
            right: make_literal(Object::Num(15.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_not_equals_string() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal_string("hello"),
            operator: Token::new(TokenType::Equals, "==".to_string(), None, 123),
            right: make_literal_string("hellx"),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(false)));
    }

    #[test]
    fn test_equals_string() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal_string("world"),
            operator: Token::new(TokenType::Equals, "==".to_string(), None, 123),
            right: make_literal_string("world"),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_equals_nil() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Nil),
            operator: Token::new(TokenType::Equals, "==".to_string(), None, 123),
            right: make_literal(Object::Nil),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_not_equals() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::BangEqual, "!=".to_string(), None, 123),
            right: make_literal(Object::Num(16.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }
}
