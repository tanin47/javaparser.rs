use analyze::definition::{Class, CompilationUnit, Decl, FieldDef, Method, Package, Root};
use std::collections::HashMap;

pub struct IdHash {
    pub underlying: HashMap<String, usize>,
}

impl IdHash {
    pub fn get_by_id<T>(&self, id: &str) -> Option<&T> {
        self.underlying
            .get(id)
            .map(|p| unsafe { &*((*p) as *const T) })
    }
}

pub fn apply(root: &Root) -> IdHash {
    let mut id_hash = IdHash {
        underlying: HashMap::new(),
    };

    build_root(root, &mut id_hash);

    id_hash
}

fn build_root(root: &Root, id_hash: &mut IdHash) {
    for package in &root.subpackages {
        build_package(package, id_hash);
    }

    for unit in &root.units {
        build_unit(unit, id_hash);
    }
}

fn build_package(package: &Package, id_hash: &mut IdHash) {
    for subpackage in &package.subpackages {
        build_package(subpackage, id_hash);
    }

    for unit in &package.units {
        build_unit(unit, id_hash);
    }
}

fn build_unit(unit: &CompilationUnit, id_hash: &mut IdHash) {
    build_decl(&unit.main, id_hash);

    for other in &unit.others {
        build_decl(other, id_hash);
    }
}

fn build_decl(decl: &Decl, id_hash: &mut IdHash) {
    match decl {
        Decl::Class(class) => build_class(class, id_hash),
        Decl::Interface(interface) => (),
    };
}

fn build_class(class: &Class, id_hash: &mut IdHash) {
    id_hash
        .underlying
        .insert(class.id.to_owned(), class as *const Class as usize);

    for method in &class.methods {
        build_method(method, id_hash);
    }

    for field_group in &class.field_groups {
        for field in &field_group.items {
            build_field(field, id_hash);
        }
    }
}

fn build_method(method: &Method, id_hash: &mut IdHash) {
    id_hash
        .underlying
        .insert(method.id.to_owned(), method as *const Method as usize);
}

fn build_field(field: &FieldDef, id_hash: &mut IdHash) {
    id_hash
        .underlying
        .insert(field.id.to_owned(), field as *const FieldDef as usize);
}
