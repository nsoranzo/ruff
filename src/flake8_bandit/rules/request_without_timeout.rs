use rustc_hash::{FxHashMap, FxHashSet};
use rustpython_ast::{Expr, ExprKind, Keyword};
use rustpython_parser::ast::Constant;

use crate::ast::helpers::{collect_call_paths, dealias_call_path, match_call_path, SimpleCallArgs};
use crate::ast::types::{Range, Resolver};
use crate::checkers::ast::Checker;
use crate::registry::Diagnostic;
use crate::violations;

const HTTP_VERBS: [&str; 7] = ["get", "options", "head", "post", "put", "patch", "delete"];

/// S113
pub fn request_without_timeout<'a, 'b, T>(
    func: &'b Expr,
    args: &'b [Expr],
    keywords: &'b [Keyword],
    resolver: &'a T,
) -> Option<Diagnostic>
where
    'b: 'a,
    T: Resolver<'a, 'b>,
{
    if let Some(call_path) = resolver.resolve_call_path(func) {
        for func_name in &HTTP_VERBS {
            if call_path == ["requests", func_name] {
                let call_args = SimpleCallArgs::new(args, keywords);
                if let Some(timeout_arg) = call_args.get_argument("timeout", None) {
                    if let Some(timeout) = match &timeout_arg.node {
                        ExprKind::Constant {
                            value: value @ Constant::None,
                            ..
                        } => Some(value.to_string()),
                        _ => None,
                    } {
                        return Some(Diagnostic::new(
                            violations::RequestWithoutTimeout(Some(timeout)),
                            Range::from_located(timeout_arg),
                        ));
                    }
                } else {
                    return Some(Diagnostic::new(
                        violations::RequestWithoutTimeout(None),
                        Range::from_located(func),
                    ));
                }
            }
        }
    }
    None
}
