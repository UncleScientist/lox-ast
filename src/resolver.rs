use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use crate::error::*;
use crate::expr::*;
use crate::interpreter::*;
use crate::stmt::*;
use crate::token::*;

pub struct Resolver<'a> {
    interpreter: &'a Interpreter,
    scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
    had_error: RefCell<bool>,
    current_function: RefCell<FunctionType>,
    current_class: RefCell<ClassType>,
    in_while: RefCell<bool>,
}

#[derive(PartialEq)]
enum FunctionType {
    None,
    Function,
    Method,
}

#[derive(PartialEq)]
enum ClassType {
    None,
    Class,
}

impl<'a> StmtVisitor<()> for Resolver<'a> {
    fn visit_class_stmt(&self, _: Rc<Stmt>, stmt: &ClassStmt) -> Result<(), LoxResult> {
        let enclosing_class = self.current_class.replace(ClassType::Class);

        self.declare(&stmt.name);
        self.define(&stmt.name);

        self.begin_scope();
        self.scopes
            .borrow()
            .last()
            .unwrap()
            .borrow_mut()
            .insert("this".to_string(), true);

        for method in stmt.methods.deref() {
            let declaration = FunctionType::Method;

            if let Stmt::Function(method) = method.deref() {
                self.resolve_function(method, declaration)?;
            } else {
                return Err(LoxResult::runtime_error(
                    &stmt.name,
                    "Class method did not resolve into a function statement",
                ));
            }
        }

        self.end_scope();
        self.current_class.replace(enclosing_class);

        Ok(())
    }

    fn visit_return_stmt(&self, _: Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), LoxResult> {
        if *self.current_function.borrow() == FunctionType::None {
            self.error(&stmt.keyword, "Can't return from top-level code.");
        }

        if let Some(value) = stmt.value.clone() {
            self.resolve_expr(value)?;
        }
        Ok(())
    }

    fn visit_function_stmt(&self, _: Rc<Stmt>, stmt: &FunctionStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        self.define(&stmt.name);

        self.resolve_function(stmt, FunctionType::Function)?;

        Ok(())
    }

    fn visit_break_stmt(&self, _: Rc<Stmt>, stmt: &BreakStmt) -> Result<(), LoxResult> {
        if !*self.in_while.borrow() {
            self.error(&stmt.token, "Break statement outside of a while/for loop.");
        }

        Ok(())
    }

    fn visit_while_stmt(&self, _: Rc<Stmt>, stmt: &WhileStmt) -> Result<(), LoxResult> {
        let previous_nesting = self.in_while.replace(true);
        self.resolve_expr(stmt.condition.clone())?;
        self.resolve_stmt(stmt.body.clone())?;

        self.in_while.replace(previous_nesting);
        Ok(())
    }

    fn visit_if_stmt(&self, _: Rc<Stmt>, stmt: &IfStmt) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.condition.clone())?;
        self.resolve_stmt(stmt.then_branch.clone())?;
        if let Some(else_branch) = stmt.else_branch.clone() {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn visit_block_stmt(&self, _: Rc<Stmt>, stmt: &BlockStmt) -> Result<(), LoxResult> {
        self.begin_scope();
        self.resolve(&stmt.statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_expression_stmt(&self, _: Rc<Stmt>, stmt: &ExpressionStmt) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.expression.clone())?;
        Ok(())
    }
    fn visit_print_stmt(&self, _: Rc<Stmt>, stmt: &PrintStmt) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.expression.clone())?;
        Ok(())
    }

    fn visit_var_stmt(&self, _: Rc<Stmt>, stmt: &VarStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        if let Some(init) = stmt.initializer.clone() {
            self.resolve_expr(init)?;
        }
        self.define(&stmt.name);
        Ok(())
    }
}

impl<'a> ExprVisitor<()> for Resolver<'a> {
    fn visit_this_expr(&self, wrapper: Rc<Expr>, expr: &ThisExpr) -> Result<(), LoxResult> {
        if *self.current_class.borrow() == ClassType::None {
            self.error(&expr.keyword, "Can't use 'this' outside of a class.");
            return Ok(());
        }

        self.resolve_local(wrapper, &expr.keyword);
        Ok(())
    }

    fn visit_set_expr(&self, _: Rc<Expr>, expr: &SetExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.value.clone())?;
        self.resolve_expr(expr.object.clone())?;
        Ok(())
    }

    fn visit_get_expr(&self, _: Rc<Expr>, expr: &GetExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.object.clone())?;
        Ok(())
    }

    fn visit_call_expr(&self, _: Rc<Expr>, expr: &CallExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.callee.clone())?;

        for argument in expr.arguments.iter() {
            self.resolve_expr(argument.clone())?;
        }

        Ok(())
    }

    fn visit_logical_expr(&self, _: Rc<Expr>, expr: &LogicalExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.left.clone())?;
        self.resolve_expr(expr.right.clone())?;
        Ok(())
    }

    fn visit_assign_expr(&self, wrapper: Rc<Expr>, expr: &AssignExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.value.clone())?;
        self.resolve_local(wrapper, &expr.name);
        Ok(())
    }

    fn visit_literal_expr(&self, _: Rc<Expr>, _expr: &LiteralExpr) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_grouping_expr(&self, _: Rc<Expr>, expr: &GroupingExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.expression.clone())?;
        Ok(())
    }

    fn visit_binary_expr(&self, _: Rc<Expr>, expr: &BinaryExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.left.clone())?;
        self.resolve_expr(expr.right.clone())?;
        Ok(())
    }

    fn visit_unary_expr(&self, _: Rc<Expr>, expr: &UnaryExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.right.clone())?;
        Ok(())
    }

    fn visit_variable_expr(&self, wrapper: Rc<Expr>, expr: &VariableExpr) -> Result<(), LoxResult> {
        if !self.scopes.borrow().is_empty()
            && self
                .scopes
                .borrow()
                .last()
                .unwrap()
                .borrow()
                .get(&expr.name.as_string())
                == Some(&false)
        {
            Err(LoxResult::runtime_error(
                &expr.name,
                "Can't read local variable in its own initizlier.",
            ))
        } else {
            self.resolve_local(wrapper, &expr.name);
            Ok(())
        }
    }
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a Interpreter) -> Self {
        Self {
            interpreter,
            scopes: RefCell::new(Vec::new()),
            had_error: RefCell::new(false),
            current_function: RefCell::new(FunctionType::None),
            current_class: RefCell::new(ClassType::None),
            in_while: RefCell::new(false),
        }
    }

    pub fn resolve(&self, statements: &Rc<Vec<Rc<Stmt>>>) -> Result<(), LoxResult> {
        for statement in statements.deref() {
            self.resolve_stmt(statement.clone())?;
        }
        Ok(())
    }

    pub fn success(&self) -> bool {
        !*self.had_error.borrow()
    }

    fn resolve_stmt(&self, stmt: Rc<Stmt>) -> Result<(), LoxResult> {
        stmt.accept(stmt.clone(), self)
    }

    fn resolve_expr(&self, expr: Rc<Expr>) -> Result<(), LoxResult> {
        expr.accept(expr.clone(), self)
    }

    fn begin_scope(&self) {
        self.scopes.borrow_mut().push(RefCell::new(HashMap::new()));
    }

    fn end_scope(&self) {
        self.scopes.borrow_mut().pop();
    }

    fn declare(&self, name: &Token) {
        if let Some(scope) = self.scopes.borrow().last() {
            if scope.borrow().contains_key(&name.as_string()) {
                self.error(name, "Already a variable with this name in this scope.");
            }

            scope.borrow_mut().insert(name.as_string(), false);
        }
    }

    fn define(&self, name: &Token) {
        if let Some(scope) = self.scopes.borrow().last() {
            scope.borrow_mut().insert(name.as_string(), true);
        }
    }

    fn resolve_local(&self, expr: Rc<Expr>, name: &Token) {
        for (scope, map) in self.scopes.borrow().iter().rev().enumerate() {
            if map.borrow().contains_key(&name.as_string()) {
                self.interpreter.resolve(expr, scope);
                return;
            }
        }
    }

    fn resolve_function(
        &self,
        function: &FunctionStmt,
        ftype: FunctionType,
    ) -> Result<(), LoxResult> {
        let enclosing_function = self.current_function.replace(ftype);

        self.begin_scope();

        for param in function.params.iter() {
            self.declare(param);
            self.define(param);
        }

        self.resolve(&function.body)?;

        self.end_scope();
        self.current_function.replace(enclosing_function);

        Ok(())
    }

    fn error(&self, token: &Token, message: &str) {
        self.had_error.replace(true);
        LoxResult::runtime_error(token, message);
    }
}
