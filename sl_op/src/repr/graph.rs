use std::collections::{HashMap, HashSet};

use crate::MacroInformation;

#[derive(Debug)]
pub struct DependencyGraph {
    pub roots: HashSet<String>,
    pub edges: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn construct_dependency_graph(
        information: &MacroInformation,
    ) -> syn::Result<DependencyGraph> {
        let mut edges = HashMap::new();
        let mut roots = HashSet::new();

        for (class_name, class) in &information.classes {
            if let Some(parent) = &class.borrow().parent {
                if !information.classes.contains_key(parent) {
                    return Err(syn::Error::new(
                        class.borrow().ident.span(),
                        format!("Class `{}` does not exist to inherit from.", parent),
                    ));
                }
                edges
                    .entry(parent.clone())
                    .or_insert_with(HashSet::new)
                    .insert(class_name.clone());
            } else {
                roots.insert(class_name.clone());
            }
        }

        Ok(DependencyGraph { roots, edges })
    }

    pub fn waterfall_dependencies(
        &self,
        information: &mut MacroInformation,
    ) -> syn::Result<HashSet<&String>> {
        let mut visited = HashSet::new();
        let mut to_visit: Vec<(&String, &String)> = Vec::new();
        for root in &self.roots {
            visited.insert(root);
            if let Some(edges) = self.edges.get(root) {
                for edge in edges {
                    if !visited.contains(edge) {
                        to_visit.push((root, edge));
                    }
                }
            }
        }
        while let Some((parent, node)) = to_visit.pop() {
            if visited.contains(node) {
                continue;
            }
            visited.insert(node);
            let parent_class = information.classes.get(parent).expect(
                "Internal State Error: Dependency graph references class which does not exist.",
            );
            let child_class = information.classes.get(node).expect(
                "Internal State Error: Dependency graph references class which does not exist.",
            );
            child_class
                .borrow_mut()
                .add_parent_information(parent_class)?;
            if let Some(edges) = self.edges.get(node) {
                for edge in edges {
                    if !visited.contains(edge) {
                        to_visit.push((node, edge));
                    }
                }
            }
        }

        Ok(visited)
    }
}
