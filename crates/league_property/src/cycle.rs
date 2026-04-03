use std::cmp;
use std::collections::{HashMap, HashSet};

use crate::{collect_dependencies, ClassMap, EnumInfo};

pub fn detect_cyclic_types(
    class_map: &ClassMap,
    merged_enums: &HashMap<u32, EnumInfo>,
) -> Vec<HashSet<u32>> {
    let graph = build_graph(class_map, merged_enums);
    let ctx = TarjanContext::new(graph);
    ctx.find_sccs()
}

fn build_graph(
    class_map: &ClassMap,
    merged_enums: &HashMap<u32, EnumInfo>,
) -> HashMap<u32, HashSet<u32>> {
    let mut graph = HashMap::new();

    // 1. Edges from Structs
    for (struct_hash, fields) in class_map {
        let deps = graph.entry(*struct_hash).or_default();
        for field_data in fields.values() {
            collect_dependencies(field_data, merged_enums, deps);
        }
    }

    // 2. Edges from Enums
    for (enum_hash, info) in merged_enums {
        let deps = graph.entry(*enum_hash).or_default();
        for variant_hash in &info.variants {
            deps.insert(*variant_hash);
        }
    }

    graph
}

struct TarjanContext {
    graph: HashMap<u32, HashSet<u32>>,
    ids: HashMap<u32, u32>,
    low: HashMap<u32, u32>,
    on_stack: HashSet<u32>,
    stack: Vec<u32>,
    id_counter: u32,
    sccs: Vec<HashSet<u32>>,
}

impl TarjanContext {
    fn new(graph: HashMap<u32, HashSet<u32>>) -> Self {
        Self {
            graph,
            ids: HashMap::new(),
            low: HashMap::new(),
            on_stack: HashSet::new(),
            stack: Vec::new(),
            id_counter: 0,
            sccs: Vec::new(),
        }
    }

    fn find_sccs(mut self) -> Vec<HashSet<u32>> {
        let keys: Vec<u32> = self.graph.keys().cloned().collect();
        for node in keys {
            if !self.ids.contains_key(&node) {
                self.dfs(node);
            }
        }
        self.sccs
    }

    fn dfs(&mut self, at: u32) {
        self.stack.push(at);
        self.on_stack.insert(at);
        self.ids.insert(at, self.id_counter);
        self.low.insert(at, self.id_counter);
        self.id_counter += 1;

        // Clone neighbors to avoid borrowing issues during recursive calls
        let neighbors = self.graph.get(&at).cloned().unwrap_or_default();

        for to in neighbors {
            self.visit_neighbor(at, to);
        }

        self.collect_scc(at);
    }

    fn visit_neighbor(&mut self, at: u32, to: u32) {
        if !self.ids.contains_key(&to) {
            self.dfs(to);
            let to_low = *self.low.get(&to).unwrap();
            let at_low = self.low.get_mut(&at).unwrap();
            *at_low = cmp::min(*at_low, to_low);
            return;
        }

        if self.on_stack.contains(&to) {
            let to_id = *self.ids.get(&to).unwrap();
            let at_low = self.low.get_mut(&at).unwrap();
            *at_low = cmp::min(*at_low, to_id);
        }
    }

    fn collect_scc(&mut self, at: u32) {
        let current_low = *self.low.get(&at).unwrap();
        let current_id = *self.ids.get(&at).unwrap();

        if current_low != current_id {
            return;
        }

        let mut current_scc = HashSet::new();
        while let Some(node) = self.stack.pop() {
            self.on_stack.remove(&node);
            current_scc.insert(node);
            if node == at {
                break;
            }
        }

        // Filter out non-cyclic single nodes (unless self-loop)
        if current_scc.len() > 1 || self.has_self_loop(at) {
            self.sccs.push(current_scc);
        }
    }

    fn has_self_loop(&self, node: u32) -> bool {
        self.graph
            .get(&node)
            .map_or(false, |deps| deps.contains(&node))
    }
}
