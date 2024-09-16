#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ImportType {
  #[doc = "
    A normal static using any syntax variations
        import .. from 'module'
    "]
  Static = 1,
  #[doc = "
    A dynamic import expression `import(specifier)` or `import(specifier, opts)`
    "]
  Dynamic = 2,
  #[doc = "
    An import.meta expression
    "]
  ImportMeta = 3,
  #[doc = "
    A source phase import 
        import source x from 'module'
    "]
  StaticSourcePhase = 4,
  #[doc = "
    A dynamic source phase import
        import.source('module')
    "]
  DynamicSourcePhase = 5,
}

#[derive(Debug, Clone)]
pub struct ImportSpecifier {
  #[doc = " Source name "]
  pub n: Option<String>,
  #[doc = " Type of import statement "]
  pub t: ImportType,
}

#[derive(Debug, Clone)]
pub struct ExportSpecifier {
  #[doc = " Export name "]
  pub n: String,
  #[doc = " Export origin name "]
  pub ln: Option<String>,
}
