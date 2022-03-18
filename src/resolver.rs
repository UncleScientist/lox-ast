use std::cell::RefCell;
use std::collections::HashMap;

use crate::error::*;
use crate::expr::*;
use crate::interpreter::*;
use crate::stmt::*;
use crate::token::*;

struct Resolver {
    interpreter: Interpreter,
    scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: RefCell::new(Vec::new()),
        }
    }
}

impl StmtVisitor<()> for Resolver {
    fn visit_return_stmt(&self, _: &Stmt, _stmt: &ReturnStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_function_stmt(&self, _: &Stmt, _stmt: &FunctionStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_break_stmt(&self, _: &Stmt, _stmt: &BreakStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_while_stmt(&self, _: &Stmt, _stmt: &WhileStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_if_stmt(&self, _: &Stmt, _stmt: &IfStmt) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_block_stmt(&self, _: &Stmt, stmt: &BlockStmt) -> Result<(), LoxResult> {
        self.begin_scope();
        self.resolve(&stmt.statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_expression_stmt(&self, _: &Stmt, _stmt: &ExpressionStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_print_stmt(&self, _: &Stmt, _stmt: &PrintStmt) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_var_stmt(&self, _: &Stmt, stmt: &VarStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        if let Some(init) = &stmt.initializer {
            self.resolve_expr(&init)?;
        }
        self.define(&stmt.name);
        Ok(())
    }
}

impl ExprVisitor<()> for Resolver {
    fn visit_call_expr(&self, _: &Expr, _expr: &CallExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_logical_expr(&self, _: &Expr, _expr: &LogicalExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_assign_expr(&self, _: &Expr, _expr: &AssignExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_literal_expr(&self, _: &Expr, _expr: &LiteralExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_grouping_expr(&self, _: &Expr, _expr: &GroupingExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_binary_expr(&self, _: &Expr, _expr: &BinaryExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_unary_expr(&self, _: &Expr, _expr: &UnaryExpr) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_variable_expr(&self, wrapper: &Expr, expr: &VariableExpr) -> Result<(), LoxResult> {
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
    fn resolve(&self, statements: &[Stmt]) -> Result<(), LoxResult> {
        for statement in statements {
            self.resolve_stmt(statement)?;
        }
        Ok(())
    }

    fn resolve_stmt(&self, stmt: &Stmt) -> Result<(), LoxResult> {
        stmt.accept(self)
    }

    fn resolve_expr(&self, expr: &Expr) -> Result<(), LoxResult> {
        expr.accept(self)
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

    fn resolve_local(&self, expr: &Expr, name: &Token) {
        for (scope, map) in self.scopes.borrow().iter().rev().enumerate() {
            if map.borrow().contains_key(&name.as_string()) {
                self.interpreter.resolve(expr, scope);
                return;
            }
        }
    }
}
