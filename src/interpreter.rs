use std::cell::RefCell;
use std::rc::Rc;

use crate::environment::*;
use crate::error::*;
use crate::expr::*;
use crate::object::*;
use crate::stmt::*;
use crate::token_type::*;

pub struct Interpreter {
    environment: RefCell<Rc<RefCell<Environment>>>,
    nest: RefCell<usize>,
}

impl StmtVisitor<()> for Interpreter {
    fn visit_break_stmt(&self, stmt: &BreakStmt) -> Result<(), LoxResult> {
        if *self.nest.borrow() == 0 {
            Err(LoxResult::runtime_error(
                &stmt.token,
                "break outside of while/for loop",
            ))
        } else {
            Err(LoxResult::Break)
        }
    }

    fn visit_while_stmt(&self, stmt: &WhileStmt) -> Result<(), LoxResult> {
        *self.nest.borrow_mut() += 1;
        while self.is_truthy(&self.evaluate(&stmt.condition)?) {
            match self.execute(&stmt.body) {
                Err(LoxResult::Break) => break,
                Err(e) => return Err(e),
                Ok(_) => {}
            }
        }
        *self.nest.borrow_mut() -= 1;

        Ok(())
    }

    fn visit_if_stmt(&self, stmt: &IfStmt) -> Result<(), LoxResult> {
        if self.is_truthy(&self.evaluate(&stmt.condition)?) {
            self.execute(&stmt.then_branch)
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch)
        } else {
            Ok(())
        }
    }

    fn visit_block_stmt(&self, stmt: &BlockStmt) -> Result<(), LoxResult> {
        let e = Environment::new_with_enclosing(self.environment.borrow().clone());
        self.execute_block(&stmt.statements, e)
    }

    fn visit_expression_stmt(&self, stmt: &ExpressionStmt) -> Result<(), LoxResult> {
        self.evaluate(&stmt.expression)?;
        Ok(())
    }

    fn visit_print_stmt(&self, stmt: &PrintStmt) -> Result<(), LoxResult> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{value}");
        Ok(())
    }

    fn visit_var_stmt(&self, stmt: &VarStmt) -> Result<(), LoxResult> {
        let value = if let Some(initializer) = &stmt.initializer {
            self.evaluate(initializer)?
        } else {
            Object::Nil
        };

        self.environment
            .borrow()
            .borrow_mut()
            .define(stmt.name.as_string(), value);
        Ok(())
    }
}

impl ExprVisitor<Object> for Interpreter {
    fn visit_call_expr(&self, expr: &CallExpr) -> Result<Object, LoxResult> {
        let callee = self.evaluate(&expr.callee)?;

        let mut arguments = Vec::new();
        for argument in &expr.arguments {
            arguments.push(self.evaluate(argument)?);
        }

        if let Object::Func(function) = callee {
            if arguments.len() != function.func.arity() {
                return Err(LoxResult::runtime_error(
                    &expr.paren,
                    &format!(
                        "Expected {} arguments but got {}.",
                        function.func.arity(),
                        arguments.len()
                    ),
                ));
            }
            function.func.call(self, arguments)
        } else {
            Err(LoxResult::runtime_error(
                &expr.paren,
                "Can only call functions and classes",
            ))
        }
    }

    fn visit_logical_expr(&self, expr: &LogicalExpr) -> Result<Object, LoxResult> {
        let left = self.evaluate(&expr.left)?;

        if expr.operator.is(TokenType::Or) {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else if !self.is_truthy(&left) {
            return Ok(left);
        }

        self.evaluate(&expr.right)
    }

    fn visit_assign_expr(&self, expr: &AssignExpr) -> Result<Object, LoxResult> {
        let value = self.evaluate(&expr.value)?;
        self.environment
            .borrow()
            .borrow_mut()
            .assign(&expr.name, value.clone())?;
        Ok(value)
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<Object, LoxResult> {
        Ok(expr.value.clone().unwrap())
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<Object, LoxResult> {
        self.evaluate(&expr.expression)
    }

    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<Object, LoxResult> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;
        let op = expr.operator.token_type();

        let result = match (left, right) {
            (Object::Num(left), Object::Num(right)) => match op {
                TokenType::Minus => Object::Num(left - right),
                TokenType::Slash => Object::Num(left / right),
                TokenType::Star => Object::Num(left * right),
                TokenType::Plus => Object::Num(left + right),
                TokenType::Greater => Object::Bool(left > right),
                TokenType::GreaterEqual => Object::Bool(left >= right),
                TokenType::Less => Object::Bool(left < right),
                TokenType::LessEqual => Object::Bool(left <= right),
                TokenType::BangEqual => Object::Bool(left != right),
                TokenType::Equals => Object::Bool(left == right),
                _ => {
                    todo!("need to work on your code dude");
                }
            },
            (Object::Num(left), Object::Str(right)) => match op {
                TokenType::Plus => Object::Str(format!("{left}{right}")),
                _ => Object::ArithmeticError,
            },
            (Object::Str(left), Object::Num(right)) => match op {
                TokenType::Plus => Object::Str(format!("{left}{right}")),
                _ => Object::ArithmeticError,
            },
            (Object::Str(left), Object::Str(right)) => match op {
                TokenType::Plus => Object::Str(format!("{left}{right}")),
                TokenType::BangEqual => Object::Bool(left != right),
                TokenType::Equals => Object::Bool(left == right),
                _ => Object::ArithmeticError,
            },
            (Object::Bool(left), Object::Bool(right)) => match op {
                TokenType::BangEqual => Object::Bool(left != right),
                TokenType::Equals => Object::Bool(left == right),
                _ => Object::ArithmeticError,
            },
            (Object::Nil, Object::Nil) => match op {
                TokenType::BangEqual => Object::Bool(false),
                TokenType::Equals => Object::Bool(true),
                _ => Object::ArithmeticError,
            },
            (Object::Nil, _) => match op {
                TokenType::Equals => Object::Bool(false),
                TokenType::BangEqual => Object::Bool(true),
                _ => Object::ArithmeticError,
            },
            _ => Object::ArithmeticError,
        };

        if result == Object::ArithmeticError {
            Err(LoxResult::runtime_error(
                &expr.operator,
                "Illegal expression",
            ))
        } else {
            Ok(result)
        }
    }

    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<Object, LoxResult> {
        let right = self.evaluate(&expr.right)?;

        match expr.operator.token_type() {
            TokenType::Minus => match right {
                Object::Num(n) => Ok(Object::Num(-n)),
                _ => Ok(Object::Nil),
            },
            TokenType::Bang => Ok(Object::Bool(!self.is_truthy(&right))),
            _ => Err(LoxResult::error(
                expr.operator.line,
                "Unreachable according to Nystrom",
            )),
        }
    }

    fn visit_variable_expr(&self, expr: &VariableExpr) -> Result<Object, LoxResult> {
        self.environment.borrow().borrow().get(&expr.name)
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: RefCell::new(Rc::new(RefCell::new(Environment::new()))),
            nest: RefCell::new(0),
        }
    }

    fn evaluate(&self, expr: &Expr) -> Result<Object, LoxResult> {
        expr.accept(self)
    }

    fn execute(&self, stmt: &Stmt) -> Result<(), LoxResult> {
        stmt.accept(self)
    }

    fn execute_block(
        &self,
        statements: &[Stmt],
        environment: Environment,
    ) -> Result<(), LoxResult> {
        let previous = self.environment.replace(Rc::new(RefCell::new(environment)));

        let result = statements
            .iter()
            .try_for_each(|statement| self.execute(statement));

        self.environment.replace(previous);

        result
    }

    // Anything that is not Nil or False is true
    fn is_truthy(&self, object: &Object) -> bool {
        !matches!(object, Object::Nil | Object::Bool(false))
    }

    pub fn interpret(&self, statements: &[Stmt]) -> bool {
        let mut success = true;
        *self.nest.borrow_mut() = 0;
        for statement in statements {
            if self.execute(statement).is_err() {
                success = false;
                break;
            }
        }

        success
    }

    pub fn print_environment(&self) {
        println!("{:?}", self.environment);
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
        let terp = Interpreter::new();
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
        let terp = Interpreter::new();
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
        let terp = Interpreter::new();
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
        let terp = Interpreter::new();
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
        let terp = Interpreter::new();
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
        let terp = Interpreter::new();
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
        let terp = Interpreter::new();
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
        let terp = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::Minus, "-".to_string(), None, 123),
            right: make_literal(Object::Bool(true)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_arithmetic_error_for_greater() {
        let terp = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::Greater, ">".to_string(), None, 123),
            right: make_literal(Object::Bool(true)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_err());
    }

    fn run_comparison_test(tok: &Token, cmps: Vec<bool>) {
        let nums = vec![14.0, 15.0, 16.0];
        let terp = Interpreter::new();

        for (c, nums) in cmps.iter().zip(nums) {
            let binary_expr = BinaryExpr {
                left: make_literal(Object::Num(nums)),
                operator: tok.dup(),
                right: make_literal(Object::Num(15.0)),
            };
            let result = terp.visit_binary_expr(&binary_expr);
            assert!(result.is_ok());
            assert_eq!(
                result.ok(),
                Some(Object::Bool(*c)),
                "Testing {} {} 15.0",
                nums,
                tok.as_string()
            );
        }
    }

    #[test]
    fn test_less_than() {
        run_comparison_test(
            &Token::new(TokenType::Less, "<".to_string(), None, 123),
            vec![true, false, false],
        );
    }

    #[test]
    fn test_less_than_or_equal_to() {
        run_comparison_test(
            &Token::new(TokenType::LessEqual, "<=".to_string(), None, 123),
            vec![true, true, false],
        );
    }

    #[test]
    fn test_greater_than() {
        run_comparison_test(
            &Token::new(TokenType::Greater, ">".to_string(), None, 123),
            vec![false, false, true],
        );
    }

    #[test]
    fn test_greater_than_or_equal_to() {
        run_comparison_test(
            &Token::new(TokenType::GreaterEqual, ">=".to_string(), None, 123),
            vec![false, true, true],
        );
    }

    #[test]
    fn test_equals_nums() {
        run_comparison_test(
            &Token::new(TokenType::Equals, "==".to_string(), None, 123),
            vec![false, true, false],
        );
    }

    #[test]
    fn test_not_equals_nums() {
        run_comparison_test(
            &Token::new(TokenType::BangEqual, "!=".to_string(), None, 123),
            vec![true, false, true],
        );
    }

    #[test]
    fn test_not_equals_string() {
        let terp = Interpreter::new();
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
        let terp = Interpreter::new();
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
        let terp = Interpreter::new();
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
    fn test_var_stmt_defined() {
        let terp = Interpreter::new();
        let name = Token::new(TokenType::Identifier, "foo".to_string(), None, 123);
        let var_stmt = VarStmt {
            name: name.dup(),
            initializer: Some(*make_literal(Object::Num(23.0))),
        };
        assert!(terp.visit_var_stmt(&var_stmt).is_ok());
        assert_eq!(
            terp.environment.borrow().get(&name).unwrap(),
            Object::Num(23.0)
        );
    }

    #[test]
    fn test_var_stmt_undefined() {
        let terp = Interpreter::new();
        let name = Token::new(TokenType::Identifier, "foo".to_string(), None, 123);
        let var_stmt = VarStmt {
            name: name.dup(),
            initializer: None,
        };
        assert!(terp.visit_var_stmt(&var_stmt).is_ok());
        assert_eq!(terp.environment.borrow().get(&name).unwrap(), Object::Nil);
    }

    #[test]
    fn test_variable_expr() {
        let terp = Interpreter::new();
        let name = Token::new(TokenType::Identifier, "foo".to_string(), None, 123);
        let var_stmt = VarStmt {
            name: name.dup(),
            initializer: Some(*make_literal(Object::Num(23.0))),
        };
        assert!(terp.visit_var_stmt(&var_stmt).is_ok());

        let var_expr = VariableExpr { name: name.dup() };
        assert_eq!(
            terp.visit_variable_expr(&var_expr).unwrap(),
            Object::Num(23.0)
        );
    }

    #[test]
    fn test_undefined_variable_expr() {
        let terp = Interpreter::new();
        let name = Token::new(TokenType::Identifier, "foo".to_string(), None, 123);
        let var_expr = VariableExpr { name: name.dup() };
        assert!(terp.visit_variable_expr(&var_expr).is_err());
    }
}
