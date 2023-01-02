use rustpython_ast::{Expr, ExprKind, Stmt, StmtKind};

use crate::ast::types::Range;
use crate::autofix::Fix;
use crate::checkers::ast::Checker;
use crate::checks::{Check, CheckKind};

fn get_items(expr: &Expr) -> Vec<String> {
    match &expr.node {
        ExprKind::Name { id, .. } => vec![id.to_string()],
        ExprKind::Tuple { elts, .. } => elts.iter().map(|e| get_items(e)).flatten().collect(),
        _ => vec![],
    }
}

pub fn rewrite_yield_from(
    checker: &mut Checker,
    stmt: &Stmt,
    target: &Expr,
    body: &Vec<Stmt>,
    iter: &Expr,
    orelse: &Vec<Stmt>,
) {
    println!("{:?}", target);
    // If there is an else statement we should not refactor
    if !orelse.is_empty() {
        return;
    }
    // Don't run if there is logic besides the yield
    if body.len() > 1 {
        return;
    }
    let first_statement = match body.get(0) {
        None => return,
        Some(item) => item,
    };
    if let StmtKind::Expr { value } = &first_statement.node {
        if let ExprKind::Yield { value: sub_value } = &value.node {
            // let for_items: Vec<String> =
            let clean_value = match sub_value {
                None => return,
                Some(item) => item,
            };
            let yield_items = get_items(clean_value);
            let target_items = get_items(target);
            if yield_items == target_items {
                let mut check = Check::new(CheckKind::RewriteYieldFrom, Range::from_located(stmt));
                let contents = checker
                    .locator
                    .slice_source_code_range(&Range::from_located(iter));
                let final_contents = format!("yield from {}", contents);
                // FOR REVIEWER: The stmt does not include comments made after the last
                // code in the for loop, which causes our version to still be "correct",
                // but to different from pyupgrade. See tests that causes difference here:
                // https://github.com/asottile/pyupgrade/blob/main/tests/features/yield_from_test.py#L52-L68
                if checker.patch(check.kind.code()) {
                    check.amend(Fix::replacement(
                        final_contents,
                        stmt.location,
                        stmt.end_location.unwrap(),
                    ));
                }
                checker.add_check(check);
            }
        }
    }
}
