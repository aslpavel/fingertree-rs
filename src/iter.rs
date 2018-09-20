use std::collections::VecDeque;
use std::iter::FusedIterator;
use std::ops::Deref;

use super::FingerTree;
use measure::Measured;
use node::{Node, NodeInner};
use reference::Refs;
use tree::Tree;

enum IterFrame<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    Node(Node<R, V>),
    Tree(Tree<R, V>),
}

impl<'a, R, V> From<&'a Node<R, V>> for IterFrame<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn from(node: &'a Node<R, V>) -> Self {
        IterFrame::Node(node.clone())
    }
}

impl<'a, R, V> From<&'a Tree<R, V>> for IterFrame<R, V>
where
    R: Refs<V>,
    V: Measured,
{
    fn from(tree: &'a Tree<R, V>) -> Self {
        IterFrame::Tree(tree.clone())
    }
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

    fn push_back<F: Into<IterFrame<R, V>>>(&mut self, frame: F) {
        self.frames.push_back(frame.into())
    }

    fn push_front<F: Into<IterFrame<R, V>>>(&mut self, frame: F) {
        self.frames.push_front(frame.into())
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
                IterFrame::Node(node) => match node {
                    Node::Leaf(value) => return Some(value.clone()),
                    Node::Node(node) => match node.deref() {
                        NodeInner::Node2 { left, right, .. } => {
                            self.push_back(right);
                            self.push_back(left);
                            continue;
                        }
                        NodeInner::Node3 {
                            left,
                            middle,
                            right,
                            ..
                        } => {
                            self.push_back(right);
                            self.push_back(middle);
                            self.push_back(left);
                            continue;
                        }
                    },
                },
                IterFrame::Tree(tree) => match tree {
                    Tree::Empty => continue,
                    Tree::Single(node) => {
                        self.push_back(&node);
                        continue;
                    }
                    Tree::Deep(deep) => {
                        for node in deep.right.as_ref().iter().rev() {
                            self.push_back(node);
                        }
                        self.push_back(&deep.spine);
                        for node in deep.left.as_ref().iter().rev() {
                            self.push_back(node);
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
                IterFrame::Node(node) => match node {
                    Node::Leaf(value) => return Some(value.clone()),
                    Node::Node(node) => match node.deref() {
                        NodeInner::Node2 { left, right, .. } => {
                            self.push_front(left);
                            self.push_front(right);
                            continue;
                        }
                        NodeInner::Node3 {
                            left,
                            middle,
                            right,
                            ..
                        } => {
                            self.push_front(left);
                            self.push_front(middle);
                            self.push_front(right);
                            continue;
                        }
                    },
                },
                IterFrame::Tree(tree) => match tree {
                    Tree::Empty => continue,
                    Tree::Single(node) => {
                        self.push_front(&node);
                        continue;
                    }
                    Tree::Deep(deep) => {
                        for node in deep.left.as_ref() {
                            self.push_front(node);
                        }
                        self.push_front(&deep.spine);
                        for node in deep.right.as_ref() {
                            self.push_front(node);
                        }
                        continue;
                    }
                },
            }
        }
    }
}
