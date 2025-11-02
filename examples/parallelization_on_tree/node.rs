use std::fmt::Debug;

pub struct Node<T> {
    pub idx: usize,
    pub data: T,
    pub children: Vec<Node<T>>,
}

impl<T> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("idx", &self.idx)
            .field("num_children", &self.children.len())
            .field(
                "children_idx",
                &self
                    .children
                    .iter()
                    .take(10)
                    .map(|x| x.idx)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<T> Node<T> {
    pub fn num_nodes(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|node| node.num_nodes())
            .sum::<usize>()
    }
}
