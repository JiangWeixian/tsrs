use std::path::Path;
use std::sync::Arc;

use crate::plugins::{DemoVisitor, ImportExportVisitor};
use crate::utils::ImportSpecifier;
use swc_core::common::{chain, comments::Comments, Mark, SourceMap};
use swc_core::ecma::{
    transforms::base::pass::noop,
    visit::{as_folder, Fold},
};

#[allow(clippy::too_many_arguments)]
pub fn transform<'a>(
    _resource_path: &'a Path,
    _comments: Option<&'a dyn Comments>,
    _top_level_mark: Mark,
    _unresolved_mark: Mark,
    _cm: Arc<SourceMap>,
    collected_imports: &'a mut Vec<ImportSpecifier>,
) -> impl Fold + 'a {
    let export_import_visitor = ImportExportVisitor::new(collected_imports);
    let ch = chain!(
        as_folder(DemoVisitor::default()),
        as_folder(export_import_visitor),
        noop()
    );
    ch
}
