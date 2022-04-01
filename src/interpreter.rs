use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use crate::callable::*;
use crate::environment::*;
use crate::error::*;
use crate::expr::*;
use crate::lox_class::*;
use crate::lox_function::*;
use crate::native_functions::*;
use crate::object::*;
use crate::stmt::*;
use crate::token::*;
use crate::token_type::*;

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: RefCell<Rc<RefCell<Environment>>>,
    locals: RefCell<HashMap<Rc<Expr>, usize>>,
}

impl StmtVisitor<()> for Interpreter {
    fn visit_class_stmt(&self, _: Rc<Stmt>, stmt: &ClassStmt) -> Result<(), LoxResult> {
        let superclass = if let Some(superclass_expr) = &stmt.superclass {
            let superclass = self.evaluate(superclass_expr.clone())?;

            if let Object::Class(c) = superclass {
                Some(c)
            } else if let Expr::Variable(v) = superclass_expr.deref() {
                return Err(LoxResult::runtime_error(
                    &v.name,
                    "Superclass must be a class.",
                ));
            } else {
                panic!("could not extract variable expr");
            }
        } else {
            None
        };

        self.environment
            .borrow()
            .borrow_mut()
            .define(&stmt.name.as_string(), Object::Nil);

        let enclosing = if let Some(ref s) = superclass {
            let mut e = Environment::new_with_enclosing(self.environment.borrow().clone());
            e.define("super", Object::Class(s.clone()));
            Some(self.environment.replace(Rc::new(RefCell::new(e))))
        } else {
            None
        };

        let mut methods = HashMap::new();
        for method in stmt.methods.deref() {
            if let Stmt::Function(func) = method.deref() {
                let is_init = func.name.as_string() == "init";
                let function = Object::Func(Rc::new(LoxFunction::new(
                    func,
                    &self.environment.borrow(),
                    is_init,
                )));
                methods.insert(func.name.as_string(), function);
            } else {
                panic!("non-function method in class");
            };
        }

        let klass = Object::Class(Rc::new(LoxClass::new(
            &stmt.name.as_string(),
            superclass,
            methods,
        )));

        if let Some(previous) = enclosing {
            self.environment.replace(previous);
        }

        self.environment
            .borrow()
            .borrow_mut()
            .assign(&stmt.name, klass)?;

        Ok(())
    }

    fn visit_return_stmt(&self, _: Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), LoxResult> {
        if let Some(value) = stmt.value.clone() {
            Err(LoxResult::return_value(self.evaluate(value)?))
        } else {
            Err(LoxResult::return_value(Object::Nil))
        }
    }

    fn visit_function_stmt(&self, _: Rc<Stmt>, stmt: &FunctionStmt) -> Result<(), LoxResult> {
        let function = LoxFunction::new(stmt, self.environment.borrow().deref(), false);
        self.environment
            .borrow()
            .borrow_mut()
            .define(&stmt.name.as_string(), Object::Func(Rc::new(function)));
        Ok(())
    }

    fn visit_break_stmt(&self, _: Rc<Stmt>, _stmt: &BreakStmt) -> Result<(), LoxResult> {
        Err(LoxResult::Break)
    }

    fn visit_while_stmt(&self, _: Rc<Stmt>, stmt: &WhileStmt) -> Result<(), LoxResult> {
        while self.is_truthy(&self.evaluate(stmt.condition.clone())?) {
            match self.execute(stmt.body.clone()) {
                Err(LoxResult::Break) => break,
                Err(e) => return Err(e),
                Ok(_) => {}
            }
        }

        Ok(())
    }

    fn visit_if_stmt(&self, _: Rc<Stmt>, stmt: &IfStmt) -> Result<(), LoxResult> {
        if self.is_truthy(&self.evaluate(stmt.condition.clone())?) {
            self.execute(stmt.then_branch.clone())
        } else if let Some(else_branch) = stmt.else_branch.clone() {
            self.execute(else_branch)
        } else {
            Ok(())
        }
    }

    fn visit_block_stmt(&self, _: Rc<Stmt>, stmt: &BlockStmt) -> Result<(), LoxResult> {
        let e = Environment::new_with_enclosing(self.environment.borrow().clone());
        self.execute_block(&stmt.statements, e)
    }

    fn visit_expression_stmt(&self, _: Rc<Stmt>, stmt: &ExpressionStmt) -> Result<(), LoxResult> {
        self.evaluate(stmt.expression.clone())?;
        Ok(())
    }

    fn visit_print_stmt(&self, _: Rc<Stmt>, stmt: &PrintStmt) -> Result<(), LoxResult> {
        let value = self.evaluate(stmt.expression.clone())?;
        println!("{value}");
        Ok(())
    }

    fn visit_var_stmt(&self, _: Rc<Stmt>, stmt: &VarStmt) -> Result<(), LoxResult> {
        let value = if let Some(initializer) = stmt.initializer.clone() {
            self.evaluate(initializer)?
        } else {
            Object::Nil
        };

        self.environment
            .borrow()
            .borrow_mut()
            .define(&stmt.name.as_string(), value);
        Ok(())
    }
}

impl ExprVisitor<Object> for Interpreter {
    fn visit_super_expr(&self, wrapper: Rc<Expr>, expr: &SuperExpr) -> Result<Object, LoxResult> {
        let distance = *self.locals.borrow().get(&wrapper).unwrap();
        let superclass = if let Some(sc) = self
            .environment
            .borrow()
            .borrow()
            .get_at(distance, "super")
            .ok()
        {
            if let Object::Class(superclass) = sc {
                superclass
            } else {
                panic!("Unable to extract superclass");
            }
        } else {
            panic!("Unable to extract superclass");
        };

        let object = self
            .environment
            .borrow()
            .borrow()
            .get_at(distance - 1, "this")
            .ok()
            .unwrap();

        if let Some(method) = superclass.find_method(&expr.method.as_string()) {
            if let Object::Func(func) = method {
                Ok(func.bind(&object))
            } else {
                panic!("method was not a function");
            }
        } else {
            Err(LoxResult::runtime_error(
                &expr.method,
                &format!("Undefined property '{}'.", expr.method.as_string()),
            ))
        }
    }

    fn visit_this_expr(&self, wrapper: Rc<Expr>, expr: &ThisExpr) -> Result<Object, LoxResult> {
        self.look_up_variable(&expr.keyword, wrapper)
    }

    fn visit_set_expr(&self, _: Rc<Expr>, expr: &SetExpr) -> Result<Object, LoxResult> {
        let object = self.evaluate(expr.object.clone())?;
        if let Object::Instance(inst) = object {
            let value = self.evaluate(expr.value.clone())?;
            inst.set(&expr.name, value.clone());
            Ok(value)
        } else {
            Err(LoxResult::runtime_error(
                &expr.name,
                "Only instances have fields",
            ))
        }
    }

    fn visit_get_expr(&self, _: Rc<Expr>, expr: &GetExpr) -> Result<Object, LoxResult> {
        let object = self.evaluate(expr.object.clone())?;
        if let Object::Instance(inst) = object {
            Ok(inst.get(&expr.name, &inst)?)
        } else {
            Err(LoxResult::runtime_error(
                &expr.name,
                "Only instances have properties",
            ))
        }
    }

    fn visit_call_expr(&self, _: Rc<Expr>, expr: &CallExpr) -> Result<Object, LoxResult> {
        let callee = self.evaluate(expr.callee.clone())?;

        let mut arguments = Vec::new();
        for argument in expr.arguments.clone() {
            arguments.push(self.evaluate(argument)?);
        }

        let (callfunc, klass): (Option<Rc<dyn LoxCallable>>, Option<Rc<LoxClass>>) = match callee {
            Object::Func(f) => (Some(f), None),
            Object::Native(n) => (Some(n.func.clone()), None),
            Object::Class(c) => {
                let klass = Rc::clone(&c);
                (Some(c), Some(klass))
            }
            _ => (None, None),
        };

        if let Some(callfunc) = callfunc {
            if arguments.len() != callfunc.arity() {
                return Err(LoxResult::runtime_error(
                    &expr.paren,
                    &format!(
                        "Expected {} arguments but got {}.",
                        callfunc.arity(),
                        arguments.len()
                    ),
                ));
            }
            callfunc.call(self, arguments, klass)
        } else {
            Err(LoxResult::runtime_error(
                &expr.paren,
                "Can only call functions and classes",
            ))
        }
    }

    fn visit_logical_expr(&self, _: Rc<Expr>, expr: &LogicalExpr) -> Result<Object, LoxResult> {
        let left = self.evaluate(expr.left.clone())?;

        if expr.operator.is(TokenType::Or) {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else if !self.is_truthy(&left) {
            return Ok(left);
        }

        self.evaluate(expr.right.clone())
    }

    fn visit_assign_expr(&self, wrapper: Rc<Expr>, expr: &AssignExpr) -> Result<Object, LoxResult> {
        let value = self.evaluate(expr.value.clone())?;
        if let Some(distance) = self.locals.borrow().get(&wrapper) {
            self.environment.borrow().borrow_mut().assign_at(
                *distance,
                &expr.name,
                value.clone(),
            )?;
        } else {
            self.globals
                .borrow_mut()
                .assign(&expr.name, value.clone())?;
        }

        Ok(value)
    }

    fn visit_literal_expr(&self, _: Rc<Expr>, expr: &LiteralExpr) -> Result<Object, LoxResult> {
        Ok(expr.value.clone().unwrap())
    }

    fn visit_grouping_expr(&self, _: Rc<Expr>, expr: &GroupingExpr) -> Result<Object, LoxResult> {
        self.evaluate(expr.expression.clone())
    }

    fn visit_binary_expr(&self, _: Rc<Expr>, expr: &BinaryExpr) -> Result<Object, LoxResult> {
        let left = self.evaluate(expr.left.clone())?;
        let right = self.evaluate(expr.right.clone())?;
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

    fn visit_unary_expr(&self, _: Rc<Expr>, expr: &UnaryExpr) -> Result<Object, LoxResult> {
        let right = self.evaluate(expr.right.clone())?;

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

    fn visit_variable_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &VariableExpr,
    ) -> Result<Object, LoxResult> {
        self.look_up_variable(&expr.name, wrapper)
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Rc::new(RefCell::new(Environment::new()));

        globals.borrow_mut().define(
            "clock",
            Object::Native(Rc::new(LoxNative {
                func: Rc::new(NativeClock {}),
            })),
        );

        Interpreter {
            globals: Rc::clone(&globals),
            environment: RefCell::new(Rc::clone(&globals)),
            locals: RefCell::new(HashMap::new()),
        }
    }

    fn evaluate(&self, expr: Rc<Expr>) -> Result<Object, LoxResult> {
        expr.accept(expr.clone(), self)
    }

    fn execute(&self, stmt: Rc<Stmt>) -> Result<(), LoxResult> {
        stmt.accept(stmt.clone(), self)
    }

    pub fn execute_block(
        &self,
        statements: &Rc<Vec<Rc<Stmt>>>,
        environment: Environment,
    ) -> Result<(), LoxResult> {
        let previous = self.environment.replace(Rc::new(RefCell::new(environment)));

        let result = statements
            .iter()
            .try_for_each(|statement| self.execute(statement.clone()));

        self.environment.replace(previous);

        result
    }

    // Anything that is not Nil or False is true
    fn is_truthy(&self, object: &Object) -> bool {
        !matches!(object, Object::Nil | Object::Bool(false))
    }

    pub fn interpret(&self, statements: &[Rc<Stmt>]) -> bool {
        let mut success = true;
        for statement in statements {
            if self.execute(statement.clone()).is_err() {
                success = false;
                break;
            }
        }

        success
    }

    pub fn print_environment(&self) {
        println!("{:?}", self.environment);
    }

    pub fn resolve(&self, expr: Rc<Expr>, depth: usize) {
        self.locals.borrow_mut().insert(expr, depth);
    }

    fn look_up_variable(&self, name: &Token, expr: Rc<Expr>) -> Result<Object, LoxResult> {
        if let Some(distance) = self.locals.borrow().get(&expr) {
            self.environment
                .borrow()
                .borrow()
                .get_at(*distance, &name.as_string())
        } else {
            self.globals.borrow().get(name)
        }
    }
}
