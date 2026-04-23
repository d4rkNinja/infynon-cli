pub struct NpmAdapter;
impl super::adapter::RegistryAdapter for NpmAdapter {
    fn resolve_dependencies(&self) -> Vec<String> {
        vec![]
    }
    fn install(&self) {}
}
