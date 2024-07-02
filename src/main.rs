use lists::sixth::LinkedList;

fn main() {
    let mut list = LinkedList::new();
    list.push_front(1);
    list.push_front(1);
    list.push_front(1);
    list.push_front(1);

    // list.push_back(2);
    // list.push_back(3);
    // list.push_back(4);

    let mut cur = list.cursor_mut();
    cur.move_next();
    let mut x = cur.split_before();

    println!("{}", x.len());
    println!("{:?}", x.pop_front());

    println!("{:?}", list.pop_front());

    println!("{}", list.len());
    //println!("{}", list.back().unwrap())
}
