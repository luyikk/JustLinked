use std::iter::FusedIterator;
use std::mem::swap;

type NodePtr<T> = *mut Node<T>;

struct Node<T> {
    prev: Option<NodePtr<T>>,
    next: Option<Box<Node<T>>>,
    key: usize,
    data: Option<T>,
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Self {
            prev: None,
            next: None,
            key: 0,
            data: None,
        }
    }
}

impl<T> Node<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        Node {
            prev: None,
            next: None,
            key: 0,
            data: Some(data),
        }
    }
    #[inline]
    pub fn into_inner(self) -> Option<T> {
        self.data
    }
    #[inline]
    pub fn get_ptr(self: &mut Box<Self>) -> NodePtr<T> {
        self.as_mut() as NodePtr<T>
    }
}

pub struct LURLinked<T> {
    map: slab::Slab<NodePtr<T>>,
    hand: Node<T>,
}

impl<T> Default for LURLinked<T> {
    fn default() -> Self {
        let mut r = Self {
            map: Default::default(),
            hand: Default::default(),
        };

        r.hand.prev = Some(r.get_hand_ptr());
        r
    }
}

impl<T> LURLinked<T> {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn get_hand_ptr(&mut self) -> NodePtr<T> {
        &mut self.hand as NodePtr<T>
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[inline]
    pub fn push(&mut self, datum: T) -> usize {
        let mut node = Box::new(Node::new(datum));
        let ptr = node.get_ptr();
        let key = self.map.insert(ptr);
        node.key = key;
        if let Some(mut next) = self.hand.next.take() {
            next.prev = Some(node.get_ptr());
            node.next = Some(next);
            node.prev = Some(self.get_hand_ptr());
            self.hand.next = Some(node);
        } else {
            node.prev = Some(self.get_hand_ptr());
            self.hand.prev = Some(ptr);
            self.hand.next = Some(node);
        }
        key
    }

    #[inline]
    pub fn remove(&mut self, key: usize) -> Option<T> {
        if let Some(current_ptr) = self.map.try_remove(key) {
            unsafe {
                let prev_p = (*current_ptr).prev?;
                let mut current = (*prev_p).next.take()?;
                if let Some(mut next) = current.next.take() {
                    next.prev = Some(prev_p);
                    (*prev_p).next = Some(next);
                } else {
                    self.hand.prev = Some(prev_p);
                }
                current.into_inner()
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn move_front(&mut self, key: usize) -> Option<usize> {
        if let Some(ref_current_ptr) = self.map.get(key) {
            unsafe {
                let current_pointer = *ref_current_ptr;
                let prev_pointer = (*current_pointer).prev?;

                let front_node_ref = self.hand.next.as_mut()?;
                let current_next_ref = (*current_pointer).next.as_mut();
                if let Some(mut next) = current_next_ref {
                    next.prev = Some(prev_pointer);
                } else {
                    self.hand.prev = Some(prev_pointer);
                }

                (*current_pointer).prev = front_node_ref.prev.replace(current_pointer);
                swap(&mut (*prev_pointer).next, &mut (*current_pointer).next);
                swap(&mut (*current_pointer).next, &mut self.hand.next);
            }
            Some(key)
        } else {
            None
        }
    }

    #[inline]
    pub fn remove_last(&mut self) -> Option<T> {
        if !self.is_empty() {
            let key = unsafe { (**self.hand.prev.as_ref()?).key };
            self.remove(key)
        } else {
            None
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.hand.next = None;
        self.hand.prev = Some(self.get_hand_ptr());
        self.map.clear();
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            entries: &self.hand,
            entries_back: &self.hand,
            len: self.len(),
        }
    }
}

pub struct Iter<'a, T> {
    entries: &'a Node<T>,
    entries_back: &'a Node<T>,
    len: usize,
}

impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            entries_back: self.entries.clone(),
            len: self.len,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (usize, &'a T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref next) = self.entries.next {
                if self.len > 0 {
                    self.entries = next.as_ref();
                    self.len -= 1;
                    if let Some(ref datum) = next.data {
                        return Some((next.key, datum));
                    }
                }else{
                    break;
                }
            } else {
                break;
            }
        }
        debug_assert_eq!(self.len, 0);
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(next) = self.entries_back.prev {
                if self.len > 0 {
                    let next = unsafe { &*next };
                    self.entries_back = next;
                    self.len -= 1;
                    if let Some(ref datum) = next.data {
                        return Some((next.key, datum));
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        debug_assert_eq!(self.len, 0);
        None
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}

impl<T> FusedIterator for Iter<'_, T> {}
