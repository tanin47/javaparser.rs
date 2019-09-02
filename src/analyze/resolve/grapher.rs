use analyze::definition::{Class, CompilationUnit, Decl, Package, Root};
use analyze::resolve::assign_type;
use analyze::resolve::scope::Scope;
use analyze::tpe::ClassType;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Node<'def> {
    pub class: *const Class<'def>,
    pub dependents: HashSet<NodeIndex>,
    pub dependencies: HashSet<NodeIndex>,
    pub fulfilled: Mutex<HashSet<NodeIndex>>,
}
unsafe impl<'a> Sync for Node<'a> {}
unsafe impl<'a> Send for Node<'a> {}

type NodeIndex = usize;

#[derive(Debug)]
pub struct Grapher<'def, 'def_ref> {
    pub nodes: Vec<Node<'def>>,
    pub map: HashMap<*const Class<'def>, NodeIndex>,
    pub pool: HashSet<NodeIndex>,
    pub scope: Scope<'def, 'def_ref>,
    pub root: &'def_ref Root<'def>,
}
unsafe impl<'def, 'def_ref> Sync for Grapher<'def, 'def_ref> {}
unsafe impl<'def, 'def_ref> Send for Grapher<'def, 'def_ref> {}

impl<'def, 'def_ref> Grapher<'def, 'def_ref> {
    pub fn new(root: &'def_ref Root<'def>) -> Grapher<'def, 'def_ref> {
        Grapher {
            nodes: vec![],
            map: HashMap::new(),
            pool: HashSet::new(),
            scope: Scope {
                root,
                levels: vec![],
                specific_imports: vec![],
                wildcard_imports: vec![],
            },
            root,
        }
    }

    pub fn get(&self, class: *const Class<'def>) -> Option<&Node<'def>> {
        match self.map.get(&class) {
            Some(&index) => Some(self.nodes.get(index).unwrap()),
            None => None,
        }
    }

    //    pub fn get_mut(&mut self, class: *const Class<'def>) -> Option<&mut Node<'def>> {
    //        match self.map.get(&class) {
    //            Some(&index) => Some(self.nodes.get_mut(index).unwrap()),
    //            None => None,
    //        }
    //    }

    pub fn collect(&mut self) {
        for package in &self.root.subpackages {
            self.collect_package(package);
        }

        for unit in &self.root.units {
            self.collect_unit(unit);
        }
    }

    pub fn collect_package(&mut self, package: &'def_ref Package<'def>) {
        self.scope.enter_package(package);
        for unit in &package.subpackages {
            self.collect_package(package);
        }
        for unit in &package.units {
            self.collect_unit(unit);
        }
        self.scope.leave()
    }

    pub fn collect_unit<'scope_ref>(&mut self, unit: &'def_ref CompilationUnit<'def>) {
        for import in &unit.imports {
            self.scope.add_import(import)
        }

        self.collect_decl(&unit.main, None);

        for other in &unit.others {
            self.collect_decl(other, None);
        }
    }

    pub fn collect_decl<'scope_ref, 'node_ref>(
        &mut self,
        decl: &'def_ref Decl<'def>,
        parent_node_opt: Option<NodeIndex>,
    ) {
        match decl {
            Decl::Class(class) => self.collect_class(class, parent_node_opt),
            _ => (),
        };
    }

    pub fn collect_class<'scope_ref, 'node_ref>(
        &mut self,
        class: &'def_ref Class<'def>,
        parent_node_opt: Option<NodeIndex>,
    ) {
        let extend_node_opt = {
            let resolved_opt = match class.extend_opt.borrow().as_ref() {
                Some(extend) => assign_type::resolve_class_type(extend, &self.scope),
                None => None,
            };
            class.extend_opt.replace(resolved_opt);

            match class.extend_opt.borrow().as_ref() {
                Some(extend) => self.collect_node(extend),
                None => None,
            }
        };

        let node_index = match self.map.get(&(class as *const Class<'def>)) {
            Some(index) => *index,
            None => {
                let node = Node {
                    class: class as *const Class<'def>,
                    dependents: HashSet::new(),
                    dependencies: HashSet::new(),
                    fulfilled: Mutex::new(HashSet::new()),
                };
                self.insert_node(node, parent_node_opt.is_some())
            }
        };
        for decl in &class.decls {
            self.collect_decl(decl, Some(node_index));
        }

        if let Some(extend_node_index) = extend_node_opt {
            {
                let node = self.nodes.get_mut(node_index).unwrap();
                node.dependencies.insert(extend_node_index);
            }
            {
                let extend_node = self.nodes.get_mut(extend_node_index).unwrap();
                extend_node.dependents.insert(node_index);
            }
        }

        if let Some(parent_node_index) = parent_node_opt {
            {
                let node = self.nodes.get_mut(node_index).unwrap();
                node.dependencies.insert(parent_node_index);
            }
            {
                let parent_node = self.nodes.get_mut(parent_node_index).unwrap();
                parent_node.dependents.insert(node_index);
            }
        }
    }

    fn insert_node(&mut self, node: Node<'def>, has_outer: bool) -> NodeIndex {
        println!("Insert {:?}", unsafe { &(*node.class) }.name.fragment);
        let key = node.class;
        let index = self.nodes.len();
        self.update_pool(&node, index, has_outer);
        self.nodes.push(node);
        self.map.insert(key, index);
        index
    }

    fn update_pool(&mut self, node: &Node<'def>, index: usize, has_outer: bool) {
        let has_super_class = unsafe { &(*node.class) }
            .extend_opt
            .borrow()
            .as_ref()
            .is_some();

        if !has_super_class && !has_outer {
            self.pool.insert(index);
        } else {
            self.pool.remove(&index);
        }
    }

    pub fn collect_node<'type_ref>(
        &mut self,
        class_type: &'type_ref ClassType<'def>,
    ) -> Option<NodeIndex> {
        let resolved =
            if let Some(resolved) = assign_type::resolve_class_type(class_type, &self.scope) {
                resolved
            } else {
                return None;
            };

        if let Some(class) = class_type.def_opt.get() {
            if let Some(&index) = self.map.get(&(class as *const Class<'def>)) {
                return Some(index);
            }
        }

        resolved.def_opt.get().map(|class| {
            let node = Node {
                class,
                dependents: HashSet::new(),
                dependencies: HashSet::new(),
                fulfilled: Mutex::new(HashSet::new()),
            };
            self.insert_node(node, false)
        })
    }
}

#[cfg(test)]
mod tests {
    use analyze;
    use analyze::definition::{Class, CompilationUnit, Decl, Import, Method, Package, Root};
    use analyze::resolve::grapher::Grapher;
    use analyze::resolve::merge;
    use analyze::tpe::{ClassType, PackagePrefix, Prefix, Type};
    use std::cell::{Cell, RefCell};
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use test_common::{code, parse, span};

    #[test]
    fn test_simple() {
        let raw1 = r#"
package dev;

class Test3 extends Test2 {}
        "#
        .to_owned();
        let raw2 = r#"
package dev;

class Test {
  class Inner {}
}
        "#
        .to_owned();
        let raw3 = r#"
package dev;

class Test2 extends Test {}
        "#
        .to_owned();
        let tokens1 = code(&raw1);
        let tokens2 = code(&raw2);
        let tokens3 = code(&raw3);
        let unit1 = parse(&tokens1);
        let unit2 = parse(&tokens2);
        let unit3 = parse(&tokens3);

        let root1 = analyze::build::apply(&unit1);
        let root2 = analyze::build::apply(&unit2);
        let root3 = analyze::build::apply(&unit3);
        let root = merge::apply(vec![root1, root2, root3]);

        let mut grapher = Grapher::new(&root);
        grapher.collect();

        let test = root.find("dev").unwrap().find_class("Test").unwrap();
        let inner = root
            .find("dev")
            .unwrap()
            .find("Test")
            .unwrap()
            .find_class("Inner")
            .unwrap();
        let test2 = root.find("dev").unwrap().find_class("Test2").unwrap();
        let test3 = root.find("dev").unwrap().find_class("Test3").unwrap();
        assert_eq!(
            grapher.pool,
            HashSet::from_iter(vec![*grapher.map.get(&test).unwrap()])
        );
        assert_eq!(grapher.nodes.len(), 4);
        assert_eq!(
            grapher.get(test).unwrap().dependents,
            HashSet::from_iter(vec![
                *grapher.map.get(&test2).unwrap(),
                *grapher.map.get(&inner).unwrap()
            ])
        );
        assert_eq!(
            grapher.get(test2).unwrap().dependents,
            HashSet::from_iter(vec![*grapher.map.get(&test3).unwrap()])
        );

        assert_eq!(
            grapher.get(inner).unwrap().dependencies,
            HashSet::from_iter(vec![*grapher.map.get(&test).unwrap()])
        );
        assert_eq!(
            grapher.get(test2).unwrap().dependencies,
            HashSet::from_iter(vec![*grapher.map.get(&test).unwrap()])
        );
        assert_eq!(
            grapher.get(test3).unwrap().dependencies,
            HashSet::from_iter(vec![*grapher.map.get(&test2).unwrap()])
        );
    }
}
