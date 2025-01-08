use crate::Id;
use std::fmt::Debug;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct UnionFind {
    parents: Vec<Id>,
}

impl UnionFind {
    pub fn make_set(&mut self) -> Id {
        let id = Id::from(self.parents.len());
        self.parents.push(id);
        id
    }

    pub fn size(&self) -> usize {
        self.parents.len()
    }

    fn parent(&self, query: Id) -> Id {
        self.parents[usize::from(query)]
    }

    fn parent_mut(&mut self, query: Id) -> &mut Id {
        &mut self.parents[usize::from(query)]
    }

    pub fn find(&self, mut current: Id) -> Id {
        while current != self.parent(current) {
            current = self.parent(current)
        }
        current
    }

    pub fn find_mut(&mut self, mut current: Id) -> Id {
        // while current != self.parent(current) {
        //     let grandparent = self.parent(self.parent(current));
        //     *self.parent_mut(current) = grandparent;
        //     current = grandparent;
        // }
        // current

        // Perform path compression after finding root
        let root = self.find(current);
        let mut node = current;
        while node != root {
            let parent = self.parent(node);
            *self.parent_mut(node) = root;
            node = parent;
        }
        root
    }

    /// Given two leader ids, unions the two eclasses making root1 the leader.
    pub fn union(&mut self, root1: Id, root2: Id) -> Id {
        *self.parent_mut(root2) = root1;
        root1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use union_find::{UnionFind as UnionFindLib, UnionBySize, QuickUnionUf};

    fn ids(us: impl IntoIterator<Item = usize>) -> Vec<Id> {
        us.into_iter().map(|u| u.into()).collect()
    }

    #[test]
    fn union_find() {
        let n = 10;
        let id = Id::from;

        let mut uf = UnionFind::default();
        for _ in 0..n {
            uf.make_set();
        }

        // test the initial condition of everyone in their own set
        assert_eq!(uf.parents, ids(0..n));

        // build up one set
        uf.union(id(0), id(1));
        uf.union(id(0), id(2));
        uf.union(id(0), id(3));

        // build up another set
        uf.union(id(6), id(7));
        uf.union(id(6), id(8));
        uf.union(id(6), id(9));

        // this should compress all paths
        for i in 0..n {
            uf.find_mut(id(i));
        }

        // indexes:         0, 1, 2, 3, 4, 5, 6, 7, 8, 9
        let expected = vec![0, 0, 0, 0, 4, 5, 6, 6, 6, 6];
        assert_eq!(uf.parents, ids(expected));
    }

    // #[test]
    // fn union_find_example() {
    //     // build a union find datastructure for 10 elements with quick unions,
    //     // merge the unions by size.
    //     let mut uf = QuickUnionUf::<UnionBySize>::new(10);

    //     // initially each element is in it's own set
    //     for i in 0..10 {
    //         assert_eq!(uf.find(i), i);
    //     }

    //     // join sets containing 0 and 1
    //     assert!(uf.union(0,1));

    //     assert_eq!(uf.find(0), 0);
    //     assert_eq!(uf.find(1), 0);
    //     for i in 2..10 {
    //         assert_eq!(uf.find(i), i);
    //     }
    // }
}
