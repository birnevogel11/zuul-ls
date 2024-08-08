use std::rc::Rc;

use bimap::BiMap;

use petgraph::algo::toposort;
use petgraph::algo::DfsSpace;
use petgraph::algo::{dijkstra, min_spanning_tree};
use petgraph::data::FromElements;
use petgraph::dot::{Config, Dot};
use petgraph::graph::DiGraph;
use petgraph::graph::{NodeIndex, UnGraph};

pub fn study_graph_topo() {
    let a = "a".to_string();
    let b = "b".to_string();
    let c = "c".to_string();
    let d = "d".to_string();

    let mut g1 = DiGraph::<&String, ()>::new();
    let mut bimap = BiMap::new();

    bimap.insert(&a, g1.add_node(&a));
    bimap.insert(&b, g1.add_node(&b));
    bimap.insert(&c, g1.add_node(&c));
    bimap.insert(&d, g1.add_node(&d));

    let f = |x: &String| *bimap.get_by_left(x).unwrap();
    g1.add_edge(f(&a), f(&b), ());
    g1.add_edge(f(&a), f(&d), ());
    g1.add_edge(f(&b), f(&c), ());
    g1.add_edge(f(&c), f(&d), ());

    let mut space = DfsSpace::default();
    let xs = toposort(&g1, Some(&mut space)).unwrap();
    let ys: Vec<_> = xs.iter().map(|x| bimap.get_by_right(x).unwrap()).collect();
    println!("{:?}", xs);
    println!("{:?}", ys);
}

pub fn study_graph_topo_origin() {
    // Create an undirected graph with `i32` nodes and edges with `()` associated data.
    let g = UnGraph::<i32, ()>::from_edges(&[(1, 2), (2, 3), (3, 4), (1, 4)]);

    // Find the shortest path from `1` to `4` using `1` as the cost for every edge.
    let node_map = dijkstra(&g, 1.into(), Some(4.into()), |_| 1);
    assert_eq!(&1i32, node_map.get(&NodeIndex::new(4)).unwrap());

    // Get the minimum spanning tree of the graph as a new graph, and check that
    // one edge was trimmed.
    let mst = UnGraph::<_, _>::from_elements(min_spanning_tree(&g));
    assert_eq!(g.raw_edges().len() - 1, mst.raw_edges().len());

    // Output the tree to `graphviz` `DOT` format
    println!("{:?}", Dot::with_config(&mst, &[Config::EdgeNoLabel]));
    // graph {
    //     0 [label="\"0\""]
    //     1 [label="\"0\""]
    //     2 [label="\"0\""]
    //     3 [label="\"0\""]
    //     1 -- 2
    //     3 -- 4
    //     2 -- 3
    // }
}
