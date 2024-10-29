use crate::types::*;
use crate::Arena;
use crate::ArenaId;

pub fn topo_sort_nodes(nodes: &Arena<Node>, sorted_nodes: &mut Vec<ArenaId<Node>>) {
	let mut stack = nodes.iter().filter(|(_, node)| match node.parent {
		NodeParent::Scene(_) | NodeParent::Orphan => true,
		NodeParent::Node(_) => false,
	}).map(|(id, _)| id).collect::<Vec<_>>();
	while let Some(node_id) = stack.pop() {
		sorted_nodes.push(node_id);
		for (child, _) in nodes.iter().filter(|(_, node)| node.parent == NodeParent::Node(node_id)) {
			stack.push(child);
		}
	}
}

#[cfg(test)]
mod tests {
    use crate::Arena;
	use super::*;

	#[test]
	pub fn topo_sort_nodes_test() {
		let mut scenes = Arena::new();
		let scene_id = scenes.insert(Scene::default());
		let mut nodes = Arena::new();

		let parent1 = nodes.insert(Node {
			parent: NodeParent::Orphan,
			..Default::default()
		});
		let parent2 = nodes.insert(Node {
			parent: NodeParent::Scene(scene_id),
			..Default::default()
		});

		let child1 = nodes.insert(Node {
			parent: NodeParent::Node(parent1),
			..Default::default()
		});

		let child2 = nodes.insert(Node {
			parent: NodeParent::Node(parent2),
			..Default::default()
		});

		let child3 = nodes.insert(Node {
			parent: NodeParent::Node(child1),
			..Default::default()
		});

		let mut sorted_nodes = Vec::new();
		topo_sort_nodes(&nodes, &mut sorted_nodes);
		assert_eq!(sorted_nodes, vec![parent2, child2, parent1, child1, child3]);

	}
}