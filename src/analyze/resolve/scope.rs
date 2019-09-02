use analyze;
use analyze::definition::{Class, Decl, Package, Root};

#[derive(Debug, PartialEq, Clone)]
pub struct Scope<'def, 'r>
where
    'def: 'r,
{
    pub root: &'r Root<'def>,
    pub levels: Vec<Level<'def>>,
    pub specific_imports: Vec<SpecificImport<'def>>,
    pub wildcard_imports: Vec<WildcardImport<'def>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SpecificImport<'def> {
    pub class: *const Class<'def>,
    pub is_static: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WildcardImport<'def> {
    pub enclosing: EnclosingType<'def>,
    pub is_static: bool,
}
unsafe impl<'def> Send for WildcardImport<'def> {}

#[derive(Debug, PartialEq, Clone)]
pub enum Level<'def> {
    EnclosingType(EnclosingType<'def>),
    Local,
}
unsafe impl<'def> Send for Level<'def> {}

#[derive(Debug, PartialEq, Clone)]
pub enum EnclosingType<'def> {
    Package(*const Package<'def>),
    Class(*const Class<'def>),
}

impl<'def> EnclosingType<'def> {
    pub fn find(&self, name: &str) -> Option<EnclosingType<'def>> {
        match self {
            EnclosingType::Package(package) => unsafe { (**package).find(name) },
            EnclosingType::Class(class) => {
                unsafe { (**class).find(name) }.map(|r| EnclosingType::Class(r))
            }
        }
    }
    pub fn find_class(&self, name: &str) -> Option<*const Class<'def>> {
        match self.find(name) {
            Some(EnclosingType::Package(_)) => panic!(),
            Some(EnclosingType::Class(class)) => Some(class),
            None => None,
        }
    }
}

impl<'def, 'r> Scope<'def, 'r> {
    pub fn add_import(&mut self, import: &analyze::definition::Import) {
        let mut imported: EnclosingType = if let Some(first) = import.components.first() {
            self.root.find(first.as_str()).unwrap()
        } else {
            return;
        };

        for component in &import.components[1..] {
            if let Some(enclosing) = self.resolve_type_at(&imported, component) {
                imported = enclosing;
            } else {
                return;
            }
        }

        if import.is_wildcard {
            self.wildcard_imports.push(WildcardImport {
                enclosing: imported,
                is_static: import.is_static,
            });
        } else {
            let class = if let EnclosingType::Class(class) = imported {
                class
            } else {
                return;
            };

            self.specific_imports.push(SpecificImport {
                class,
                is_static: import.is_static,
            });
        };
    }

    pub fn enter_package(&mut self, package: &'r Package<'def>) {
        self.levels
            .push(Level::EnclosingType(EnclosingType::Package(package)));
    }

    pub fn leave(&mut self) {
        self.levels.pop();
    }

    pub fn enter_class(&mut self, class: &'r Class<'def>) {
        self.levels
            .push(Level::EnclosingType(EnclosingType::Class(class)));
    }

    pub fn wrap_local<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut Scope<'def, 'r>) -> (),
    {
        self.levels.push(Level::Local);
        func(self);
        self.levels.pop();
    }

    pub fn resolve_package(&self, name: &str) -> Option<*const Package<'def>> {
        self.root.find_package(name)
    }

    pub fn resolve_type(&self, name: &str) -> Option<EnclosingType<'def>> {
        for i in 0..self.levels.len() {
            let current = self.levels.get(self.levels.len() - 1 - i).unwrap();

            match current {
                Level::EnclosingType(enclosing) => {
                    if let Some(result) = self.resolve_type_at(enclosing, name) {
                        return Some(result);
                    }

                    // We search until the closest package. Java only allows referring to a package using its full path.
                    // There's no such thing as a relative path for package.
                    if let EnclosingType::Package(_) = enclosing {
                        break;
                    }
                }
                Level::Local => (),
            };
        }

        if let Some(result) = self.resolve_type_with_specific_import(name) {
            return Some(result);
        }

        self.root.find(name)
    }

    pub fn resolve_type_with_specific_import(&self, name: &str) -> Option<EnclosingType<'def>> {
        for import in &self.specific_imports {
            if unsafe { (*import.class).name.fragment } == name {
                return Some(EnclosingType::Class(import.class));
            }
        }

        None
    }

    pub fn resolve_type_at(
        &self,
        current: &EnclosingType<'def>,
        name: &str,
    ) -> Option<EnclosingType<'def>> {
        match current {
            EnclosingType::Package(package) => {
                if let Some(found) = unsafe { &(**package) }.find(name) {
                    return Some(found);
                }
            }
            EnclosingType::Class(class) => {
                for decl in unsafe { &(**class).decls } {
                    if let Decl::Class(subclass) = decl {
                        if subclass.name.fragment == name {
                            return Some(EnclosingType::Class(subclass));
                        }
                    }
                }
            }
        };

        None
    }
}
