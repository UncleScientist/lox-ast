use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use crate::error::*;
use crate::expr::*;
use crate::interpreter::*;
use crate::stmt::*;
use crate::token::*;

struct Resolver {
    interpreter: Interpreter,
    scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
}

impl StmtVisitor<()> for Resolver {
    fn visit_return_stmt(&self, _: Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), LoxResult> {
        if let Some(value) = stmt.value.clone() {
            self.resolve_expr(value)?;
        }
        Ok(())
    }
    fn visit_function_stmt(&self, _: Rc<Stmt>, stmt: &FunctionStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        self.define(&stmt.name);

        self.resolve_function(stmt);

        Ok(())
    }
    fn visit_break_stmt(&self, _: Rc<Stmt>, _stmt: &BreakStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_while_stmt(&self, _: Rc<Stmt>, stmt: &WhileStmt) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.condition.clone())?;
        self.resolve_stmt(stmt.body.clone())?;
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

impl ExprVisitor<()> for Resolver {
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
            && !self
                .scopes
                .borrow()
                .last()
                .unwrap()
                .borrow()
                .get(&expr.name.as_string())
                .unwrap()
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

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: RefCell::new(Vec::new()),
        }
    }

    fn resolve(&self, statements: &Rc<Vec<Rc<Stmt>>>) -> Result<(), LoxResult> {
        for statement in statements.deref() {
            self.resolve_stmt(statement.clone())?;
        }
        Ok(())
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
        if !self.scopes.borrow().is_empty() {
            self.scopes
                .borrow()
                .last()
                .unwrap()
                .borrow_mut()
                .insert(name.as_string(), false);
        }
    }

    fn define(&self, name: &Token) {
        if !self.scopes.borrow().is_empty() {
            self.scopes
                .borrow()
                .last()
                .unwrap()
                .borrow_mut()
                .insert(name.as_string(), true);
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

    fn resolve_function(&self, function: &FunctionStmt) -> Result<(), LoxResult> {
        self.begin_scope();

        for param in function.params.iter() {
            self.declare(param);
            self.define(param);
        }

        self.resolve(&function.body)?;

        self.end_scope();
        Ok(())
    }
}
