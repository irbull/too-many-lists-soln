use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem,
            next: None,
            prev: None,
        }))
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        // new node needs +2 links, everything else should be +0
        let new_head = Node::new(elem);
        match self.head.take() {
            Some(old_head) => {
                // non-empty list, need to connect the old_head
                old_head.borrow_mut().prev = Some(new_head.clone()); // +1 new_head
                new_head.borrow_mut().next = Some(old_head); // +1 old_head
                self.head = Some(new_head); // +1 new_head, -1 old_head
                                            // total: +2 new_head, +0 old_head -- OK!
            }
            None => {
                // empty list, need to set the tail
                self.tail = Some(new_head.clone()); // +1 new_head
                self.head = Some(new_head); // +1 new_head
                                            // total: +2 new_head -- OK!
            }
        }
    }

    pub fn push_back(&mut self, elem: T) {
        let new_tail = Node::new(elem);
        match self.tail.take() {
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(new_tail.clone());
                new_tail.borrow_mut().prev = Some(old_tail);
                self.tail = Some(new_tail);
            }
            None => {
                self.head = Some(new_tail.clone());
                self.tail = Some(new_tail);
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if let Some(old_head) = self.head.take() {
            if let Some(new_head) = take_next(&old_head) {
                take_prev(&new_head);
                self.head = Some(new_head);
            } else {
                self.tail.take();
            }
            Some(Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem)
        } else {
            None
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if let Some(old_tail) = self.tail.take() {
            if let Some(new_tail) = take_prev(&old_tail) {
                take_next(&new_tail);
                self.tail = Some(new_tail);
            } else {
                self.head.take();
            }
            Some(Rc::try_unwrap(old_tail).ok().unwrap().into_inner().elem)
        } else {
            None
        }
    }

    pub fn replace_front(&mut self, elem: T) {
        if let Some(node) = std::borrow::Borrow::borrow(&self.head) {
            node.borrow_mut().elem = elem;
        }
    }

    pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
        if let Some(node) = std::borrow::Borrow::borrow(&self.head) {
            return Some(RefMut::map(node.borrow_mut(), |node| &mut node.elem));
        }
        None
    }

    pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
        if let Some(node) = std::borrow::Borrow::borrow(&self.tail) {
            return Some(RefMut::map(node.borrow_mut(), |node| &mut node.elem));
        }
        None
    }

    pub fn peek_front(&self) -> Option<Ref<T>> {
        self.head
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |node| &node.elem))
    }

    pub fn peek_back(&self) -> Option<Ref<T>> {
        self.tail
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |node| &node.elem))
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}
impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

pub struct IntoIter<T>(List<T>);

fn take_next<T>(node: &Rc<RefCell<Node<T>>>) -> Option<Rc<RefCell<Node<T>>>> {
    node.borrow_mut().next.take()
}
fn take_prev<T>(node: &Rc<RefCell<Node<T>>>) -> Option<Rc<RefCell<Node<T>>>> {
    node.borrow_mut().prev.take()
}

#[cfg(test)]
mod test {

    use super::List;
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop_front(), None);

        // Populate list
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_front(4);
        list.push_front(5);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);

        // ---- back -----

        // Check empty list behaves right
        assert_eq!(list.pop_back(), None);

        // Populate list
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        // Check normal removal
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_back(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_back(4);
        list.push_back(5);

        // Check normal removal
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());
        assert!(list.peek_back().is_none());
        assert!(list.peek_front_mut().is_none());
        assert!(list.peek_back_mut().is_none());

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(&*list.peek_front().unwrap(), &3);
        assert_eq!(&mut *list.peek_front_mut().unwrap(), &mut 3);
        assert_eq!(&*list.peek_back().unwrap(), &1);
        assert_eq!(&mut *list.peek_back_mut().unwrap(), &mut 1);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn rev_iter() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        let mut iter = list.into_iter().rev();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }
}
