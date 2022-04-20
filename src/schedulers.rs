use crate::contexts::{Context, ContextList};


/// Given a Linked List, Q, iterate through Q to find the highest ratio by comparing each ratio within the queue.
/// Once a ratio of element N is greater than the element M with the highest ratio, replace element M with element N
/// as the highest ratio element in the list. Once the end of the list is reached, dequeue the highest ratio element.
/// If the element is at the start of the list, dequeue it and set the list to its next element, returning the element.
/// Otherwise N's neighbors are reassigned to identify each other as their next and previous neighbor, returning the
/// result of N.
pub fn hrrn(pq: ContextList) -> Context {
    Context {}
}
