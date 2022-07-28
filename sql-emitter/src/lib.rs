#![allow(dead_code)]
#![allow(unused_variables)]

pub mod ast;

use std::fmt::{Write, Result as FResult};

pub fn emit(select_stmt: &ast::SelectStatement) -> Result<String, String> {
    let mut printer = Printer::default();
    if printer.print_query(&select_stmt).is_err() {
        Err(String::from("Failed to format SQL output"))
    } else {
        Ok(String::from(printer))
    }
}

#[derive(Default, Debug)]
pub struct Printer {
    output: String,
    indent: u32
}

impl Printer {
    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        assert!(self.indent > 0);
        self.indent -= 1;
    }

    fn end_line(&mut self) {
        self.output.push('\n');
    }

    fn start_line(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    pub fn print_query(&mut self, select_stmt: &ast::SelectStatement) -> FResult {
        self.print_select(&select_stmt.modifier, &select_stmt.select)?;
        self.print_from(&select_stmt.from)?;
        if let Some(cond) = &select_stmt.where_ {
            self.print_where(&cond)?;
        }
        if let Some(order) = &select_stmt.order_by {
            self.print_order_by(&order)?;
        }
        Ok(())
    }

    fn print_select(&mut self, modifier: &Option<ast::Modifier>, select: &ast::SelectList) -> FResult {
        self.start_line();
        match modifier {
            Some(ast::Modifier::Distinct) => write!(self.output, "SELECT DISTINCT ")?,
            Some(ast::Modifier::All) => write!(self.output, "SELECT ALL ")?,
            None => write!(self.output, "SELECT ")?
        };
        let mut first = true;
        if select.wildcard {
            write!(self.output, "* ")?;
            first = false;
        }
        for field in select.columns.iter() {
            if !first {
                write!(self.output, ", ")?;
            }
            self.print_val_expr(&field.value)?;
            if let Some(alias) = &field.alias {
                write!(self.output, " as {}", alias)?;
            }
            first = false;
        }
        self.end_line();
        Ok(())
    }

    fn print_from(&mut self, table_refs: &ast::TableReference) -> FResult {
        match table_refs {
            ast::TableReference::TableName { name } => {
                self.start_line();
                write!(self.output, "FROM {}", name)?;
                self.end_line();
            },
            ast::TableReference::InnerStatement { value } => {
                self.start_line();
                write!(self.output, "FROM (")?;
                self.end_line();

                self.indent();
                self.print_query(&*value)?;
                self.dedent();

                self.start_line();
                write!(self.output, ")")?;
                self.end_line();
            },
        }
        Ok(())
    }

    fn print_where(&mut self, cond: &ast::SearchCondition) -> FResult {
        match cond {
            ast::SearchCondition::BoolExpr { left, op, right } => todo!(),
            ast::SearchCondition::ComparisonExpr { left, op, right } => todo!(),
        }
    }

    fn print_order_by(&mut self, cond: &ast::OrderByClause) -> FResult {
        todo!()
    }

    fn print_val_expr(&mut self, expr: &ast::ValueExpression) -> FResult {
        match expr {
            ast::ValueExpression::Column { name } => write!(self.output, "{}", name),
            ast::ValueExpression::FuncCall { name, args } => {
                write!(self.output, "{}(", name)?;
                let mut first = true;
                for arg in args.iter() {
                    if !first {
                        write!(self.output, ", ")?;
                    }
                    self.print_val_expr(arg)?;
                    first = false;
                }
                write!(self.output, ")")?;
                Ok(())
            },
            ast::ValueExpression::ArithmeticExpr { left, op, right } => {
                self.print_val_expr(left)?;
                write!(self.output, " {} ", op)?;
                self.print_val_expr(right)?;
                Ok(())
            },
            ast::ValueExpression::Literal { value } => {
                match value {
                    ast::Literal::Bool(v) => write!(self.output, "{}", v),
                    ast::Literal::Integer(v) => write!(self.output, "{}", v),
                    ast::Literal::Real(v) => write!(self.output, "{}", v),
                    ast::Literal::String(v) => write!(self.output, "{}", v),
                }
            },
        }
    }
}

impl From<Printer> for String {
    fn from(p: Printer) -> Self {
        p.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let query = ast::SelectStatement {
            modifier: None,
            select: ast::SelectList {
                wildcard: true,
                columns: vec![]
            },
            from: ast::TableReference::TableName { name: String::from("users") },
            where_: None,
            order_by: None,
        };

        let mut printer = Printer::default();
        assert!(printer.print_query(&query).is_ok());
        assert_eq!(String::from("SELECT *\nFROM users\n"), String::from(printer));
    }
}