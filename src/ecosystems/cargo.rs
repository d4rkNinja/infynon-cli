pub struct CargoAdapter;
impl super::adapter::RegistryAdapter for CargoAdapter {
    fn resolve_dependencies(&self) -> Vec<String> {
        vec![]
    }
    fn install(&self) {}
}
