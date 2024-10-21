#[derive(Debug)]
struct Node<'a, T> {
	data: T,
	next: Option<&'a Node<'a, T>>
}

/*struct Node<'a> {
	data: i32,
	next: Option<&'a Node<'a>>
}*/

fn iterate_nodes<T: std::fmt::Debug + std::fmt::Display>(head: &Node<T>){
	let mut nd = head;

	loop {
		println!("Node data: {} {:?}", nd.data, nd.next);

		nd = match nd.next {
			Some(n) => n,
			None => break,
		};
	}
}

fn main() {
	let head = &mut Node::<i32> {data: 1, next: None};

	iterate_nodes(head);

	println!("Head = {{{} {:?}}}", head.data, head.next);

	head.next = Some(&Node {data: 2, next: None});

	iterate_nodes(head);
}
