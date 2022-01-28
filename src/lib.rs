use std::sync::Arc;

mod object;

pub use object::{Oid, Object};

pub struct Db<O: Object + ?Sized> {
    mapping: Vec<u64>,
    ops: Vec<ObjectOp<O>>
}

impl<O: Object + ?Sized> Db<O> {
    pub fn new() -> Self {
        Self {
            mapping: Vec::new(),
            ops: Vec::new()
        }
    }
    pub fn get(&self, oid: Oid, max_offset: u64) -> Option<Arc<O>> {
        let idx: usize = oid.0.try_into().unwrap();
        let mut offset = *self.mapping.get(idx)?;
        let mut changes = vec![];
        let mut obj = loop {
            match &self.ops[offset as usize] {
                ObjectOp::Put( _ , obj ) => break obj.clone(),
                ObjectOp::Patch { prev_offset, change, .. } => {
                    if max_offset >= offset {
                        changes.push(change);
                    }
                    offset = *prev_offset;
                }
            }
        };
        if offset > max_offset { return None; }
        if changes.len() > 0 { 
            Arc::make_mut(&mut obj).apply(changes.as_slice());
        }
        Some(obj)
    }
    pub fn patch(&mut self, oid: Oid, change: O::Change) {
        let idx: usize = oid.0.try_into().unwrap();
        let prev_offset = self.mapping[idx];
        let offset = self.ops.len();
        self.ops.push(ObjectOp::Patch {
            oid,
            prev_offset,
            change
        });
        self.mapping[idx] = offset as u64;
    }
    pub fn replace(&mut self, oid: Oid, obj: Arc<O>) {
        let idx: usize = oid.0.try_into().unwrap();
        let offset = self.ops.len();
        self.ops.push(ObjectOp::Put(oid, obj));
        self.mapping[idx] = offset as u64;
    }
    pub fn insert(&mut self, obj: Arc<O>) -> Oid {
        let idx = self.mapping.len();
        let oid = Oid(idx as u64);
        let offset = self.ops.len() as u64;
        self.ops.push(ObjectOp::Put(oid, obj));
        self.mapping.push(offset);
        oid
    }
}

pub enum ObjectOp<O: Object + ?Sized> {
    Put(Oid, Arc<O>),
    Patch {
        oid: Oid,
        prev_offset: u64,
        change: O::Change
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{Object, Db};

    #[test]
    fn test() {
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct L(Vec<&'static str>);

        impl Object for L {
            type Change = &'static str;

            fn apply(&mut self, changes: &[&&'static str]) {
                for s in changes.iter().rev() {
                    self.0.push(**s);
                }
            }
        }

        let mut db = Db::new();
        
        let o1 = db.insert(Arc::new(L(vec![])));
        let o2 = db.insert(Arc::new(L(vec!["d"])));

        db.patch(o1, "a");
        db.patch(o2, "e");
        db.patch(o1, "b");
        db.patch(o1, "c");

        assert_eq!(db.get(o1, 5), Some(Arc::new(L(vec!["a", "b", "c"]))));
        assert_eq!(db.get(o1, 1000), Some(Arc::new(L(vec!["a", "b", "c"]))));
        assert_eq!(db.get(o2, 5), Some(Arc::new(L(vec!["d", "e"]))));
        assert_eq!(db.get(o1, 3), Some(Arc::new(L(vec!["a"]))));

        db.replace(o1, Arc::new(L(vec!["1", "2", "3"])));
        db.patch(o1, "4");

        assert_eq!(db.get(o1, 5), None);
        assert_eq!(db.get(o1, 6), Some(Arc::new(L(vec!["1", "2", "3"]))));
        assert_eq!(db.get(o1, 1000), Some(Arc::new(L(vec!["1", "2", "3", "4"]))));
    }
}