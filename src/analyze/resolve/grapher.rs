use analyze::definition::{Class, CompilationUnit, Decl, Package, Root};
use analyze::resolve::assign_type;
use analyze::resolve::scope::{EnclosingTypeDef, Level, Scope};
use parse::tree::{ClassType, Type};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Node<'def, 'def_ref> {
    pub class: *const Class<'def>,
    pub index: NodeIndex,
    pub scope: RefCell<Option<Scope<'def, 'def_ref>>>,
    pub dependents: HashSet<NodeIndex>,
    pub dependencies: HashSet<NodeIndex>,
    pub fulfilled: Mutex<HashSet<NodeIndex>>,
}
unsafe impl<'def, 'def_ref> Sync for Node<'def, 'def_ref> {}
unsafe impl<'def, 'def_ref> Send for Node<'def, 'def_ref> {}

type NodeIndex = usize;

#[derive(Debug)]
pub struct Grapher<'def, 'def_ref> {
    pub nodes: Vec<Node<'def, 'def_ref>>,
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

    pub fn get(&self, class: *const Class<'def>) -> Option<&Node<'def, 'def_ref>> {
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
        for subpackage in &package.subpackages {
            self.collect_package(subpackage);
        }
        for unit in &package.units {
            self.collect_unit(unit);
        }
        self.scope.leave()
    }

    pub fn collect_unit<'scope_ref>(&mut self, unit: &'def_ref CompilationUnit<'def>) {
        for import in &unit.imports {
            self.scope.add_import(unsafe { &**import });
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
                Some(extend) => {
                    if let Some(Type::Class(resolved)) =
                        assign_type::resolve_class_or_parameterized_type(extend, &self.scope)
                    {
                        Some(resolved)
                    } else {
                        None
                    }
                }
                None => None,
            };

            if let Some(resolved) = resolved_opt {
                class.extend_opt.replace(Some(resolved));
            }

            match class.extend_opt.borrow().as_ref() {
                Some(extend) => self.collect_node(extend),
                None => None,
            }
        };

        let node_index = match self.map.get(&(class as *const Class<'def>)) {
            Some(index) => *index,
            None => self.create_node(class as *const Class<'def>, parent_node_opt.is_some()),
        };
        self.update_pool(node_index, parent_node_opt.is_some());
        for decl in &class.decls {
            self.scope.enter_class(class);
            self.collect_decl(decl, Some(node_index));
            self.scope.leave();
        }

        {
            let node = self.nodes.get_mut(node_index).unwrap();
            node.scope.replace(Some(self.scope.clone()));
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

    fn create_node(&mut self, class: *const Class<'def>, has_outer: bool) -> NodeIndex {
        let index = self.nodes.len();
        let node = Node {
            class: class as *const Class<'def>,
            index,
            scope: RefCell::new(None),
            dependents: HashSet::new(),
            dependencies: HashSet::new(),
            fulfilled: Mutex::new(HashSet::new()),
        };

        let key = node.class;
        self.map.insert(node.class, index);
        self.nodes.insert(index, node);
        index
    }

    fn update_pool(&mut self, node_index: NodeIndex, has_outer: bool) {
        let node = self.nodes.get(node_index).unwrap();
        let has_super_class = unsafe { &(*node.class) }
            .extend_opt
            .borrow()
            .as_ref()
            .is_some();

        if !has_super_class && !has_outer {
            self.pool.insert(node.index);
        } else {
            self.pool.remove(&node.index);
        }
    }

    pub fn collect_node<'type_ref>(
        &mut self,
        class_type: &'type_ref ClassType<'def>,
    ) -> Option<NodeIndex> {
        let resolved = if let Some(Type::Class(resolved)) =
            assign_type::resolve_class_or_parameterized_type(class_type, &self.scope)
        {
            resolved
        } else {
            return None;
        };

        if let Some(class) = class_type.def_opt {
            if let Some(&index) = self.map.get(&(class as *const Class<'def>)) {
                return Some(index);
            }
        }

        resolved.def_opt.map(|class| self.create_node(class, false))
    }
}

#[cfg(test)]
mod tests {
    use analyze;
    use analyze::definition::{Class, Decl, Method, Package, Root};
    use analyze::resolve::grapher::{Grapher, NodeIndex};
    use analyze::resolve::merge;
    use analyze::test_common::{find_class, make_root, make_tokenss, make_units};
    use parse::tree::CompilationUnit;
    use std::cell::{Cell, RefCell};
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use test_common::{code, parse, span};
    use tokenize::token::Token;

    impl<'def, 'def_ref> Grapher<'def, 'def_ref> {
        fn check_pool(&self, pool: &[&Class<'def>]) {
            assert_eq!(
                self.get_classes(&self.pool),
                self.get_classes(&self.get_node_indices(pool))
            );
        }
        fn check(
            &self,
            class: *const Class<'def>,
            dependents: &[&Class<'def>],
            dependencies: &[&Class<'def>],
        ) {
            let node = self.get(class).unwrap();
            assert_eq!(
                self.get_classes(&node.dependents),
                self.get_classes(&self.get_node_indices(dependents))
            );
            assert_eq!(
                self.get_classes(&node.dependencies),
                self.get_classes(&self.get_node_indices(dependencies))
            );
        }

        fn get_classes(&self, node_indices: &HashSet<NodeIndex>) -> HashSet<&str> {
            let mut list = HashSet::new();
            for node_index in node_indices {
                list.insert(
                    unsafe { &(*self.nodes.get(*node_index).unwrap().class) }
                        .name
                        .fragment,
                );
            }
            list
        }

        fn get_node_indices(&self, classes: &[&Class<'def>]) -> HashSet<NodeIndex> {
            let mut set = HashSet::new();
            for class in classes {
                set.insert(*self.map.get(&((*class) as *const Class)).unwrap());
            }
            set
        }
    }

    #[test]
    fn test_complex_2() {
        let raws = vec![
            r#"
package dev;

class Test<A> extends Super {
  class Inner extends Typed<A> {}
}
        "#
            .to_owned(),
            r#"
package dev;

class Super {
  class A {}
  class Typed<A> {}
}
        "#
            .to_owned(),
        ];
        let tokenss = make_tokenss(&raws);
        let units = make_units(&tokenss);
        let root = make_root(&units);

        let mut grapher = Grapher::new(&root);
        grapher.collect();

        let test = find_class(&root, "dev.Test");
        let inner = find_class(&root, "dev.Test.Inner");
        let super_class = find_class(&root, "dev.Super");
        let super_a = find_class(&root, "dev.Super.A");
        let super_typed = find_class(&root, "dev.Super.Typed");
        assert_eq!(grapher.nodes.len(), 5);
        grapher.check_pool(&vec![super_class]);
        grapher.check(test, &vec![inner], &vec![super_class]);
        grapher.check(inner, &vec![], &vec![test, super_typed]);
        grapher.check(super_class, &vec![test, super_a, super_typed], &vec![]);
        grapher.check(super_a, &vec![], &vec![super_class]);
        grapher.check(super_typed, &vec![inner], &vec![super_class]);
    }

    #[test]
    fn test_complex() {
        let raws = vec![
            r#"
package dev;

class Test<A> {
  class Inner {
    class InnerOfInner extends Super<A> {
      SuperInner method() {}
    } 
  }
}
        "#
            .to_owned(),
            r#"
package dev;

class Super<T> {
  class SuperInner {
    T method() {}
  }
}
        "#
            .to_owned(),
        ];
        let tokenss = make_tokenss(&raws);
        let units = make_units(&tokenss);
        let root = make_root(&units);

        let mut grapher = Grapher::new(&root);
        grapher.collect();

        let test = find_class(&root, "dev.Test");
        let inner = find_class(&root, "dev.Test.Inner");
        let inner_of_inner = find_class(&root, "dev.Test.Inner.InnerOfInner");
        let super_class = find_class(&root, "dev.Super");
        let super_inner = find_class(&root, "dev.Super.SuperInner");
        assert_eq!(grapher.nodes.len(), 5);
        grapher.check_pool(&vec![test, super_class]);
        grapher.check(test, &vec![inner], &vec![]);
        grapher.check(inner, &vec![inner_of_inner], &vec![test]);
        grapher.check(inner_of_inner, &vec![], &vec![inner, super_class]);
        grapher.check(super_class, &vec![inner_of_inner, super_inner], &vec![]);
        grapher.check(super_inner, &vec![], &vec![super_class]);
    }

    #[test]
    fn test_simple() {
        let raws = vec![
            r#"
package dev;

class Test3 extends Test2 {}
        "#
            .to_owned(),
            r#"
package dev;

class Test {
  class Inner {}
}
        "#
            .to_owned(),
            r#"
package dev;

class Test2 extends Test {}
        "#
            .to_owned(),
        ];
        let tokenss = make_tokenss(&raws);
        let units = make_units(&tokenss);
        let root = make_root(&units);

        let mut grapher = Grapher::new(&root);
        grapher.collect();

        let test = find_class(&root, "dev.Test");
        let inner = find_class(&root, "dev.Test.Inner");
        let test2 = find_class(&root, "dev.Test2");
        let test3 = find_class(&root, "dev.Test3");
        assert_eq!(grapher.nodes.len(), 4);
        grapher.check_pool(&vec![test]);
        grapher.check(test, &vec![test2, inner], &vec![]);
        grapher.check(inner, &vec![], &vec![test]);
        grapher.check(test2, &vec![test3], &vec![test]);
        grapher.check(test3, &vec![], &vec![test2]);
    }
}
