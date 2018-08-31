use std::collections::VecDeque;
use std::iter::FusedIterator;

use super::FingerTree;
use measure::Measured;
use node::{Node, NodeInner};
use reference::Refs;
use tree::{Tree, TreeInner};

enum IterFrame<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    Node(Node<R, V>),
    Tree(Tree<R, V>),
}

pub struct Iter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    frames: VecDeque<IterFrame<R, V>>,
}

impl<R, V> Iter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    pub(crate) fn new(ft: &FingerTree<R, V>) -> Self {
        let mut frames = VecDeque::new();
        frames.push_back(IterFrame::Tree(ft.rec.clone()));
        Iter { frames }
    }
}

impl<R, V> FusedIterator for Iter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
}

impl<R, V> Iterator for Iter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.frames.pop_back()? {
                IterFrame::Node(node) => match node.as_ref() {
                    NodeInner::Leaf(value) => return Some(value.clone()),
                    NodeInner::Node2 { left, right, .. } => {
                        self.frames.push_back(IterFrame::Node(right.clone()));
                        self.frames.push_back(IterFrame::Node(left.clone()));
                        continue;
                    }
                    NodeInner::Node3 {
                        left,
                        middle,
                        right,
                        ..
                    } => {
                        self.frames.push_back(IterFrame::Node(right.clone()));
                        self.frames.push_back(IterFrame::Node(middle.clone()));
                        self.frames.push_back(IterFrame::Node(left.clone()));
                        continue;
                    }
                },
                IterFrame::Tree(tree) => match tree.as_ref() {
                    TreeInner::Empty => continue,
                    TreeInner::Single(node) => {
                        self.frames.push_back(IterFrame::Node(node.clone()));
                        continue;
                    }
                    TreeInner::Deep {
                        left, spine, right, ..
                    } => {
                        for node in right.as_ref().iter().rev() {
                            self.frames.push_back(IterFrame::Node(node.clone()));
                        }
                        self.frames.push_back(IterFrame::Tree(spine.clone()));
                        for node in left.as_ref().iter().rev() {
                            self.frames.push_back(IterFrame::Node(node.clone()));
                        }
                        continue;
                    }
                },
            }
        }
    }
}

impl<R, V> DoubleEndedIterator for Iter<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.frames.pop_front()? {
                IterFrame::Node(node) => match node.as_ref() {
                    NodeInner::Leaf(value) => return Some(value.clone()),
                    NodeInner::Node2 { left, right, .. } => {
                        self.frames.push_front(IterFrame::Node(left.clone()));
                        self.frames.push_front(IterFrame::Node(right.clone()));
                        continue;
                    }
                    NodeInner::Node3 {
                        left,
                        middle,
                        right,
                        ..
                    } => {
                        self.frames.push_front(IterFrame::Node(left.clone()));
                        self.frames.push_front(IterFrame::Node(middle.clone()));
                        self.frames.push_front(IterFrame::Node(right.clone()));
                        continue;
                    }
                },
                IterFrame::Tree(tree) => match tree.as_ref() {
                    TreeInner::Empty => continue,
                    TreeInner::Single(node) => {
                        self.frames.push_front(IterFrame::Node(node.clone()));
                        continue;
                    }
                    TreeInner::Deep {
                        left, spine, right, ..
                    } => {
                        for node in left.as_ref() {
                            self.frames.push_front(IterFrame::Node(node.clone()));
                        }
                        self.frames.push_front(IterFrame::Tree(spine.clone()));
                        for node in right.as_ref() {
                            self.frames.push_front(IterFrame::Node(node.clone()));
                        }
                        continue;
                    }
                },
            }
        }
    }
}
