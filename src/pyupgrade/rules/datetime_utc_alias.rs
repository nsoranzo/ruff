use rustpython_ast::Expr;

use crate::ast::helpers::compose_call_path;
use crate::ast::types::Range;
use crate::checkers::ast::Checker;
use crate::fix::Fix;
use crate::registry::{Diagnostic, RuleCode};
use crate::violations;

/// UP017
pub fn datetime_utc_alias(checker: &mut Checker, expr: &Expr) {
    if checker.resolve_call_path(expr).map_or(false, |call_path| {
        call_path == ["datetime", "timezone", "utc"]
    }) {
        let mut diagnostic =
            Diagnostic::new(violations::DatetimeTimezoneUTC, Range::from_located(expr));
        if checker.patch(&RuleCode::UP017) {
            diagnostic.amend(Fix::replacement(
                compose_call_path(expr)
                    .unwrap()
                    .replace("timezone.utc", "UTC"),
                expr.location,
                expr.end_location.unwrap(),
            ));
        }
        checker.diagnostics.push(diagnostic);
    }
}
