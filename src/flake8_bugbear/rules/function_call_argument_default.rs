use rustpython_ast::{Arguments, Constant, Expr, ExprKind};

use crate::ast::helpers::{compose_call_path, to_module_and_member};
use crate::ast::types::Range;
use crate::ast::visitor;
use crate::ast::visitor::Visitor;
use crate::checkers::ast::Checker;
use crate::flake8_bugbear::rules::mutable_argument_default::is_mutable_func;
use crate::registry::{Diagnostic, DiagnosticKind};
use crate::violations;

const IMMUTABLE_FUNCS: [(&str, &str); 7] = [
    ("", "tuple"),
    ("", "frozenset"),
    ("operator", "attrgetter"),
    ("operator", "itemgetter"),
    ("operator", "methodcaller"),
    ("types", "MappingProxyType"),
    ("re", "compile"),
];

fn is_immutable_func(
    checker: &Checker,
    expr: &Expr,
    extend_immutable_calls: &[(&str, &str)],
) -> bool {
    checker.resolve_call_path(expr).map_or(false, |call_path| {
        IMMUTABLE_FUNCS
            .iter()
            .chain(extend_immutable_calls)
            .any(|(module, member)| call_path == [*module, *member])
    })
}

struct ArgumentDefaultVisitor<'a> {
    checker: &'a Checker<'a>,
    diagnostics: Vec<(DiagnosticKind, Range)>,
    extend_immutable_calls: &'a [(&'a str, &'a str)],
}

impl<'a, 'b> Visitor<'b> for ArgumentDefaultVisitor<'b>
where
    'b: 'a,
{
    fn visit_expr(&mut self, expr: &'b Expr) {
        match &expr.node {
            ExprKind::Call { func, args, .. } => {
                if !is_mutable_func(self.checker, func)
                    && !is_immutable_func(self.checker, func, self.extend_immutable_calls)
                    && !is_nan_or_infinity(func, args)
                {
                    self.diagnostics.push((
                        violations::FunctionCallArgumentDefault(compose_call_path(expr)).into(),
                        Range::from_located(expr),
                    ));
                }
                visitor::walk_expr(self, expr);
            }
            ExprKind::Lambda { .. } => {}
            _ => visitor::walk_expr(self, expr),
        }
    }
}

fn is_nan_or_infinity(expr: &Expr, args: &[Expr]) -> bool {
    let ExprKind::Name { id, .. } = &expr.node else {
        return false;
    };
    if id != "float" {
        return false;
    }
    let Some(arg) = args.first() else {
        return false;
    };
    let ExprKind::Constant {
        value: Constant::Str(value),
        ..
    } = &arg.node else {
        return false;
    };
    let lowercased = value.to_lowercase();
    matches!(
        lowercased.as_str(),
        "nan" | "+nan" | "-nan" | "inf" | "+inf" | "-inf" | "infinity" | "+infinity" | "-infinity"
    )
}

/// B008
pub fn function_call_argument_default(checker: &mut Checker, arguments: &Arguments) {
    // Map immutable calls to (module, member) format.
    let extend_immutable_cells: Vec<(&str, &str)> = checker
        .settings
        .flake8_bugbear
        .extend_immutable_calls
        .iter()
        .map(|target| to_module_and_member(target))
        .collect();
    let diagnostics = {
        let mut visitor = ArgumentDefaultVisitor {
            checker,
            diagnostics: vec![],
            extend_immutable_calls: &extend_immutable_cells,
        };
        for expr in arguments
            .defaults
            .iter()
            .chain(arguments.kw_defaults.iter())
        {
            visitor.visit_expr(expr);
        }
        visitor.diagnostics
    };
    for (check, range) in diagnostics {
        checker.diagnostics.push(Diagnostic::new(check, range));
    }
}
