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
        match select {
            ast::SelectList::Explicit { fields } => {
                let mut first = true;
                for field in fields {
                    if !first {
                        write!(self.output, ", ")?;
                    }
                    self.print_val_expr(&field.value)?;
                    if let Some(alias) = &field.alias {
                        write!(self.output, " as {}", alias)?;
                    }
                    first = false;
                }
            },
            ast::SelectList::Wildcard => write!(self.output, "*")?
        };
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
        todo!()
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
            select: ast::SelectList::Wildcard,
            from: ast::TableReference::TableName { name: String::from("users") },
            where_: None,
            order_by: None,
        };

        let mut printer = Printer::default();
        assert!(printer.print_query(&query).is_ok());
        assert_eq!(String::from("SELECT *\nFROM users\n"), String::from(printer));
    }
}