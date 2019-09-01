use analyze::definition::{Class, CompilationUnit, Decl, Package, Root};
use analyze::resolve::assign_type;
use analyze::resolve::scope::Scope;
use analyze::tpe::ClassType;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone)]
pub struct Node<'def> {
    pub class: *const Class<'def>,
    pub dependents: HashSet<*const Class<'def>>,
    pub dependencies: HashSet<*const Class<'def>>,
    pub fulfilled: HashSet<*const Class<'def>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Grapher<'def, 'def_ref> {
    pub nodes: Vec<Node<'def>>,
    pub map: HashMap<*const Class<'def>, usize>,
    pub pool: HashSet<*const Class<'def>>,
    pub scope: Scope<'def, 'def_ref>,
    pub root: &'def_ref Root<'def>,
}

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
        parent_node_opt: Option<*mut Node<'def>>,
    ) {
        match decl {
            Decl::Class(class) => self.collect_class(class, parent_node_opt),
            _ => (),
        };
    }

    pub fn collect_class<'scope_ref, 'node_ref>(
        &mut self,
        class: &'def_ref Class<'def>,
        parent_node_opt: Option<*mut Node<'def>>,
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

        let node_ptr = match self.map.get(&(class as *const Class<'def>)) {
            Some(index) => self.nodes.get_mut(*index).unwrap() as *mut Node<'def>,
            None => {
                let node = Node {
                    class: class as *const Class<'def>,
                    dependents: HashSet::new(),
                    dependencies: HashSet::new(),
                    fulfilled: HashSet::new(),
                };
                self.insert_node(node, parent_node_opt.is_some()) as *mut Node<'def>
            }
        };
        for decl in &class.decls {
            self.collect_decl(decl, Some(node_ptr));
        }

        let node = unsafe { &mut (*node_ptr) };
        if let Some(extend_node) = extend_node_opt {
            let extend_node = unsafe { &mut (*extend_node) };
            node.dependencies
                .insert(extend_node.class as *const Class<'def>);
            extend_node
                .dependents
                .insert(node.class as *const Class<'def>);
        }

        if let Some(parent_node) = parent_node_opt {
            let parent_node = unsafe { &mut (*parent_node) };
            node.dependencies
                .insert(parent_node.class as *const Class<'def>);
            parent_node
                .dependents
                .insert(node.class as *const Class<'def>);
        }
    }

    fn insert_node(&mut self, node: Node<'def>, has_outer: bool) -> &mut Node<'def> {
        self.update_pool(&node, has_outer);
        let key = node.class;
        let index = self.nodes.len();
        self.nodes.push(node);
        self.map.insert(key, index);
        self.nodes.get_mut(index).unwrap()
    }

    fn update_pool(&mut self, node: &Node<'def>, has_outer: bool) {
        let has_super_class = unsafe { &(*node.class) }
            .extend_opt
            .borrow()
            .as_ref()
            .is_some();

        if !has_super_class && !has_outer {
            self.pool.insert(node.class);
        } else {
            self.pool.remove(&node.class);
        }
    }

    pub fn collect_node<'type_ref>(
        &mut self,
        class_type: &'type_ref ClassType<'def>,
    ) -> Option<*mut Node<'def>> {
        let resolved =
            if let Some(resolved) = assign_type::resolve_class_type(class_type, &self.scope) {
                resolved
            } else {
                return None;
            };

        if let Some(class) = class_type.def_opt.get() {
            if let Some(&index) = self.map.get(&(class as *const Class<'def>)) {
                return Some(self.nodes.get_mut(index).unwrap() as *mut Node<'def>);
            }
        }

        resolved.def_opt.get().map(|class| {
            let node = Node {
                class,
                dependents: HashSet::new(),
                dependencies: HashSet::new(),
                fulfilled: HashSet::new(),
            };
            self.insert_node(node, false) as *mut Node<'def>
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
        assert_eq!(grapher.pool, HashSet::from_iter(vec![test]));
        assert_eq!(
            grapher.get(test).unwrap().dependents,
            HashSet::from_iter(vec![test2, inner])
        );
        assert_eq!(
            grapher.get(test2).unwrap().dependents,
            HashSet::from_iter(vec![test3])
        );

        assert_eq!(
            grapher.get(inner).unwrap().dependencies,
            HashSet::from_iter(vec![test])
        );
        assert_eq!(
            grapher.get(test2).unwrap().dependencies,
            HashSet::from_iter(vec![test])
        );
        assert_eq!(
            grapher.get(test3).unwrap().dependencies,
            HashSet::from_iter(vec![test2])
        );
    }
}
