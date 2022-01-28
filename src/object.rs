pub trait Object: Clone {
    type Change;

    /** Apply changes (changes are in reverse order) */
    fn apply(&mut self, changes: &[&Self::Change]);
}

// An object id
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct Oid(pub u64);

impl PartialOrd for Oid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Oid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}