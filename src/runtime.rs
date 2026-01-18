use std::collections::HashMap;
use crate::ast::*;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Array(Vec<Value>),
}

pub struct Runtime {
    globals: HashMap<String, Value>,
    locals: Vec<HashMap<String, Value>>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            globals: HashMap::new(),
            locals: Vec::new(),
        }
    }

    pub fn set_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    pub fn get_var(&self, name: &str) -> Option<Value> {
        // Check locals from innermost to outermost
        for scope in self.locals.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        // Check globals
        self.globals.get(name).cloned()
    }

    pub fn set_var(&mut self, name: String, value: Value) {
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name, value);
        } else {
            self.globals.insert(name, value);
        }
    }

    pub fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.locals.pop();
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Array(exprs) => {
                let mut values = Vec::new();
                for e in exprs {
                    values.push(self.eval_expr(e)?);
                }
                Ok(Value::Array(values))
            }
            Expr::Var(name) => {
                self.get_var(name).ok_or_else(|| format!("Undefined variable: {}", name))
            }
            Expr::DataRef(name) => {
                self.get_var(name).ok_or_else(|| format!("Undefined data field: {}", name))
            }
            Expr::Call { name, args } => {
                Err(format!("Function calls not yet supported in evaluation: {}", name))
            }
            Expr::Index { target, index } => {
                let target_val = self.eval_expr(target)?;
                let index_val = self.eval_expr(index)?;

                match (&target_val, &index_val) {
                    (Value::Array(arr), Value::Int(idx)) => {
                        let i = *idx as usize;
                        arr.get(i).cloned().ok_or_else(|| format!("Array index out of bounds: {}", i))
                    }
                    _ => Err("Array indexing requires array and int".to_string()),
                }
            }
        }
    }

    pub fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Option<Value>, String> {
        match stmt {
            Stmt::Let { name, ty: _, value } => {
                let val = self.eval_expr(value)?;
                self.set_var(name.clone(), val);
                Ok(None)
            }
            Stmt::ExprStmt(expr) => {
                self.eval_expr(expr)?;
                Ok(None)
            }
            Stmt::Return(expr) => {
                let val = self.eval_expr(expr)?;
                Ok(Some(val))
            }
        }
    }

    pub fn eval_function(&mut self, func: &Function, args: Vec<Value>) -> Result<Value, String> {
        // Check argument count
        if func.params.len() != args.len() {
            return Err(format!(
                "Function {} expects {} arguments, got {}",
                func.name,
                func.params.len(),
                args.len()
            ));
        }

        self.push_scope();

        // Bind parameters
        for (param, arg) in func.params.iter().zip(args.iter()) {
            self.set_var(param.name.clone(), arg.clone());
        }

        // Execute body
        let mut result = Value::Int(0);
        for stmt in &func.body {
            if let Some(val) = self.eval_stmt(stmt)? {
                result = val;
                break;
            }
        }

        // Check for return expression
        if let Some(expr) = &func.return_expr {
            result = self.eval_expr(expr)?;
        }

        self.pop_scope();
        Ok(result)
    }
}
