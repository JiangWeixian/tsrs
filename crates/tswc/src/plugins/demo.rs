use swc_core::ecma::ast::*;
use swc_core::ecma::visit::VisitMut;

#[derive(Default)]
pub struct DemoVisitor {}

impl VisitMut for DemoVisitor {
    fn visit_mut_export_default_expr(&mut self, n: &mut ExportDefaultExpr) {
        println!("export_decl {:?}", n);
    }
}
