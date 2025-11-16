use crate::lir::{LirFormat, LirType};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TopoSortError {
    #[error("Circular dependency detected involving type: {0}")]
    CircularDependency(String),
    #[error("Unknown type reference: {0}")]
    UnknownType(String),
}

pub fn topological_sort(format: &mut LirFormat) -> Result<(), TopoSortError> {
    let type_map: HashMap<String, &LirType> = format
        .types
        .iter()
        .map(|t| (t.name.clone(), t))
        .collect();

    let mut dependencies: HashMap<String, HashSet<String>> = HashMap::new();

    for lir_type in &format.types {
        let mut deps = HashSet::new();

        for field in &lir_type.fields {
            let dep_type = extract_base_type(&field.type_info);

            if type_map.contains_key(dep_type) {
                deps.insert(dep_type.to_string());
            }
        }

        dependencies.insert(lir_type.name.clone(), deps);
    }

    let sorted_names = kahn_sort(&dependencies)?;

    let mut sorted_types = Vec::new();
    for name in sorted_names {
        if let Some(lir_type) = format.types.iter().find(|t| t.name == name) {
            sorted_types.push(lir_type.clone());
        }
    }

    format.types = sorted_types;

    Ok(())
}

fn extract_base_type(type_str: &str) -> &str {
    if let Some(bracket_pos) = type_str.find('[') {
        &type_str[..bracket_pos]
    } else {
        type_str
    }
}

fn kahn_sort(
    dependencies: &HashMap<String, HashSet<String>>,
) -> Result<Vec<String>, TopoSortError> {
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

    for (node, _) in dependencies {
        in_degree.insert(node.clone(), 0);
        adj_list.insert(node.clone(), Vec::new());
    }

    for (node, deps) in dependencies {
        for dep in deps {
            adj_list
                .get_mut(dep)
                .ok_or_else(|| TopoSortError::UnknownType(dep.clone()))?
                .push(node.clone());

            *in_degree
                .get_mut(node)
                .ok_or_else(|| TopoSortError::UnknownType(node.clone()))? += 1;
        }
    }

    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, &degree)| degree == 0)
        .map(|(name, _)| name.clone())
        .collect();

    let mut result = Vec::new();

    while let Some(node) = queue.pop_front() {
        result.push(node.clone());

        if let Some(neighbors) = adj_list.get(&node) {
            for neighbor in neighbors {
                let degree = in_degree
                    .get_mut(neighbor)
                    .ok_or_else(|| TopoSortError::UnknownType(neighbor.clone()))?;

                *degree -= 1;

                if *degree == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    if result.len() != dependencies.len() {
        let unprocessed: Vec<String> = dependencies
            .keys()
            .filter(|k| !result.contains(k))
            .cloned()
            .collect();

        return Err(TopoSortError::CircularDependency(
            unprocessed.join(", "),
        ));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kahn_sort_simple() {
        let mut deps = HashMap::new();
        deps.insert("A".to_string(), HashSet::new());
        deps.insert("B".to_string(), HashSet::from(["A".to_string()]));
        deps.insert("C".to_string(), HashSet::from(["B".to_string()]));

        let result = kahn_sort(&deps).unwrap();

        assert_eq!(result.len(), 3);
        let a_pos = result.iter().position(|x| x == "A").unwrap();
        let b_pos = result.iter().position(|x| x == "B").unwrap();
        let c_pos = result.iter().position(|x| x == "C").unwrap();

        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_kahn_sort_circular() {
        let mut deps = HashMap::new();
        deps.insert("A".to_string(), HashSet::from(["B".to_string()]));
        deps.insert("B".to_string(), HashSet::from(["A".to_string()]));

        let result = kahn_sort(&deps);
        assert!(result.is_err());
    }
}
