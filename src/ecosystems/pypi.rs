pub struct PipAdapter;
impl super::adapter::RegistryAdapter for PipAdapter {
    fn resolve_dependencies(&self) -> Vec<String> {
        vec![]
    }
    fn install(&self) {}
}
