mod assets;
mod compiler;
mod module_graph;
mod transform;
pub use assets::Assets;
pub use compiler::SwcCompiler;
pub use module_graph::{Module, ModuleGraph, ResolveModuleOptions};
pub use transform::{compile, optimize};
