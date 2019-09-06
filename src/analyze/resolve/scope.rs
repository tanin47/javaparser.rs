use analyze;
use analyze::definition::{Class, Decl, Package, Root};
use analyze::tpe::{ClassType, EnclosingType, PackagePrefix};

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
    pub enclosing: EnclosingTypeDef<'def>,
    pub is_static: bool,
}
unsafe impl<'def> Send for WildcardImport<'def> {}

#[derive(Debug, PartialEq, Clone)]
pub enum Level<'def> {
    EnclosingType(EnclosingTypeDef<'def>),
    Local,
}
unsafe impl<'def> Send for Level<'def> {}

#[derive(Debug, PartialEq, Clone)]
pub enum EnclosingTypeDef<'def> {
    Package(*const Package<'def>),
    Class(*const Class<'def>),
}

impl<'def> EnclosingTypeDef<'def> {
    pub fn find(&self, name: &str) -> Option<EnclosingTypeDef<'def>> {
        match self {
            EnclosingTypeDef::Package(package) => unsafe { (**package).find(name) },
            EnclosingTypeDef::Class(class) => {
                unsafe { (**class).find(name) }.map(|r| EnclosingTypeDef::Class(r))
            }
        }
    }
    pub fn find_class(&self, name: &str) -> Option<&Class<'def>> {
        match self.find(name) {
            Some(EnclosingTypeDef::Package(_)) => panic!(),
            Some(EnclosingTypeDef::Class(class)) => Some(unsafe { &*class }),
            None => None,
        }
    }

    pub fn to_type(&self) -> EnclosingType<'def> {
        match self {
            EnclosingTypeDef::Package(package) => {
                let package = unsafe { &(**package) };
                EnclosingType::Package(PackagePrefix {
                    name: &package.name,
                    def: package as *const Package,
                })
            }
            EnclosingTypeDef::Class(class) => {
                let class = unsafe { &(**class) };
                EnclosingType::Class(class.to_type())
            }
        }
    }
}

impl<'def, 'r> Scope<'def, 'r> {
    pub fn add_import(&mut self, import: &analyze::definition::Import) {
        let mut imported: EnclosingTypeDef = if let Some(first) = import.components.first() {
            self.root.find(first.as_str()).unwrap()
        } else {
            return;
        };

        for component in &import.components[1..] {
            if let Some(enclosing) = self.resolve_type_def_at(&imported, component) {
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
            let class = if let EnclosingTypeDef::Class(class) = imported {
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
            .push(Level::EnclosingType(EnclosingTypeDef::Package(package)));
    }

    pub fn leave(&mut self) {
        self.levels.pop();
    }

    pub fn enter_class(&mut self, class: &'r Class<'def>) {
        self.levels
            .push(Level::EnclosingType(EnclosingTypeDef::Class(class)));
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
        self.root
            .find_package(name)
            .map(|p| p as *const Package<'def>)
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
                    if let EnclosingTypeDef::Package(_) = enclosing {
                        break;
                    }
                }
                Level::Local => (),
            };
        }

        if let Some(result) = self.resolve_type_with_specific_import(name) {
            return Some(result);
        }

        self.root.find(name).map(|e| e.to_type())
    }

    pub fn resolve_type_with_specific_import(&self, name: &str) -> Option<EnclosingType<'def>> {
        for import in &self.specific_imports {
            let class = unsafe { &(*import.class) };
            if class.name.fragment == name {
                return Some(EnclosingType::Class(class.to_type()));
            }
        }

        None
    }

    pub fn resolve_type_at(
        &self,
        current: &EnclosingTypeDef<'def>,
        name: &str,
    ) -> Option<EnclosingType<'def>> {
        match current {
            EnclosingTypeDef::Package(package) => {
                let package = unsafe { &(**package) };

                if let Some(enclosing) = package.find(name) {
                    return Some(enclosing.to_type());
                }
            }
            EnclosingTypeDef::Class(class) => {
                let class = unsafe { &(**class) }.to_type();

                if let Some(found) = class.find_inner_class(name) {
                    return Some(EnclosingType::Class(found));
                }

                if let Some(extend_class) = class.extend_opt.borrow().as_ref() {
                    if let Some(found) = extend_class.find_inner_class(name) {
                        return Some(EnclosingType::Class(found));
                    }
                }
            }
        };

        None
    }

    pub fn resolve_type_def_at(
        &self,
        current: &EnclosingTypeDef<'def>,
        name: &str,
    ) -> Option<EnclosingTypeDef<'def>> {
        match current {
            EnclosingTypeDef::Package(package) => {
                if let Some(found) = unsafe { &(**package) }.find(name) {
                    return Some(found);
                }
            }
            EnclosingTypeDef::Class(class) => {
                let class = unsafe { &(**class) };
                for decl in &class.decls {
                    if let Decl::Class(subclass) = decl {
                        if subclass.name.fragment == name {
                            return Some(EnclosingTypeDef::Class(subclass));
                        }
                    }
                }
            }
        };

        None
    }
}
