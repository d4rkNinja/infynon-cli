pub trait RegistryAdapter {
    fn resolve_dependencies(&self) -> Vec<String>;
    fn install(&self);
}
