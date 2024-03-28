// Copyright 2024 Cloudflare, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::mem::replace;

type Index = usize;

const NULL: Index = usize::MAX;
const HEAD: Index = 0;
const TAIL: Index = 1;
const OFFSET: Index = 2;

#[derive(Debug)]
struct Node {
    pub(crate) prev: Index,
    pub(crate) next: Index,
    pub(crate) data: u64,
}

#[derive(Debug)]
struct Nodes {
    head: Node,
    tail: Node,
    data_nodes: Vec<Node>,
}

impl Nodes {
    fn with_capacity(capacity: usize) -> Self {
        Nodes {
            head: Node {
                prev: NULL,
                next: TAIL,
                data: 0,
            },
            tail: Node {
                prev: HEAD,
                next: NULL,
                data: 0,
            },
            data_nodes: Vec::with_capacity(capacity),
        }
    }

    fn new_node(&mut self, data: u64) -> Index {
        let node = Node {
            prev: NULL,
            next: NULL,
            data,
        };
        self.data_nodes.push(node);
        self.data_nodes.len() - 1 + OFFSET
    }

    fn len(&self) -> usize {
        self.data_nodes.len()
    }

    fn head(&self) -> &Node {
        &self.head
    }

    fn tail(&self) -> &Node {
        &self.tail
    }
}

impl std::ops::Index<usize> for Nodes {
    type Output = Node;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            HEAD => &self.head,
            TAIL => &self.tail,
            _ => &self.data_nodes[index - OFFSET],
        }
    }
}

impl std::ops::IndexMut<usize> for Nodes {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            HEAD => &mut self.head,
            TAIL => &mut self.tail,
            _ => &mut self.data_nodes[index - OFFSET],
        }
    }
}

pub struct LinkedList {
    nodes: Nodes,
    free: Vec<Index>,
}

impl LinkedList {
    pub fn with_capacity(capacity: usize) -> Self {
        LinkedList {
            nodes: Nodes::with_capacity(capacity),
            free: vec![],
        }
    }

    fn new_node(&mut self, data: u64) -> Index {
        if let Some(index) = self.free.pop() {
            self.nodes[index].data = data;
            index
        } else {
            self.nodes.new_node(data)
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len() - self.free.len()
    }

    fn valid_index(&self, index: Index) -> bool {
        index != HEAD && index != TAIL && index - OFFSET < self.nodes.len()
    }

    fn node(&self, index: Index) -> Option<&Node> {
        if self.valid_index(index) {
            Some(&self.nodes[index])
        } else {
            None
        }
    }

    pub fn peek(&self, index: Index) -> Option<u64> {
        self.node(index).map(|n| n.data)
    }

    fn peek_unchecked(&self, index: Index) -> &u64 {
        &self.nodes[index].data
    }

    pub fn exist_near_head(&self, value: u64, search_limit: usize) -> bool {
        let mut current_node = HEAD;
        for _ in 0..search_limit {
            current_node = self.nodes[current_node].next;
            if current_node == TAIL {
                return false;
            }
            if self.nodes[current_node].data == value {
                return true;
            }
        }
        false
    }

    fn insert_after(&mut self, node_index: Index, at: Index) {
        assert!(at != TAIL && at != node_index);

        let next = replace(&mut self.nodes[at].next, node_index);
        let node = &mut self.nodes[node_index];
        node.next = next;
        node.prev = at;

        self.nodes[next].prev = node_index;
    }

    pub fn push_head(&mut self, data: u64) -> Index {
        let new_node_index = self.new_node(data);
        self.insert_after(new_node_index, HEAD);
        new_node_index
    }

    pub fn push_tail(&mut self, data: u64) -> Index {
        let new_node_index = self.new_node(data);
        self.insert_after(new_node_index, self.nodes.tail.prev);
        new_node_index
    }

    fn lift(&mut self, index: Index) -> u64 {
        assert!(index != HEAD && index != TAIL);

        let node = &mut self.nodes[index];
        let prev = replace(&mut node.prev, NULL);
        let next = replace(&mut node.next, NULL);
        let data = node.data;

        assert!(prev != NULL && next != NULL);

        self.nodes[prev].next = next;
        self.nodes[next].prev = prev;

        data
    }

    pub fn remove(&mut self, index: Index) -> u64 {
        self.free.push(index);
        self.lift(index)
    }

    pub fn pop_tail(&mut self) -> Option<u64> {
        let data_tail = self.nodes.tail().prev;
        if data_tail == HEAD {
            None
        } else {
            Some(self.remove(data_tail))
        }
    }

    pub fn promote(&mut self, index: Index) {
        if self.nodes.head().next == index {
            return;
        }
        self.lift(index);
        self.insert_after(index, HEAD);
    }

    fn next(&self, index: Index) -> Index {
        self.nodes[index].next
    }

    fn prev(&self, index: Index) -> Index {
        self.nodes[index].prev
    }

    pub fn head(&self) -> Option<Index> {
        let data_head = self.nodes.head().next;
        if data_head == TAIL {
            None
        } else {
            Some(data_head)
        }
    }

    pub fn tail(&self) -> Option<Index> {
        let data_tail = self.nodes.tail().prev;
        if data_tail == HEAD {
            None
        } else {
            Some(data_tail)
        }
    }

    pub fn iter(&self) -> LinkedListIter<'_> {
        LinkedListIter {
            list: self,
            head: HEAD,
            tail: TAIL,
            len: self.len(),
        }
    }
}

pub struct LinkedListIter<'a> {
    list: &'a LinkedList,
    head: Index,
    tail: Index,
    len: usize,
}

impl<'a> Iterator for LinkedListIter<'a> {
    type Item = &'a u64;

    fn next(&mut self) -> Option<Self::Item> {
        let next_index = self.list.next(self.head);
        if next_index == TAIL || next_index == NULL {
            None
        } else {
            self.head = next_index;
            self.len -= 1;
            Some(self.list.peek_unchecked(next_index))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> DoubleEndedIterator for LinkedListIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let prev_index = self.list.prev(self.tail);
        if prev_index == HEAD || prev_index == NULL {
            None
        } else {
            self.tail = prev_index;
            self.len -= 1;
            Some(self.list.peek_unchecked(prev_index))
        }
    }
}
