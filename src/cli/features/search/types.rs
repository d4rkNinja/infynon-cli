#[derive(Debug, Clone)]
pub(super) struct SearchHit {
    pub(super) eco: String,
    pub(super) pkg: String,
    pub(super) ver: String,
    pub(super) desc: String,
    pub(super) signal: String,
    pub(super) score: i32,
}
