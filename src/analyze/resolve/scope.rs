use analyze::definition::{Class, Package, Root};

pub struct Scope<'def, 'r>
where
    'def: 'r,
{
    pub root: &'r Root<'def>,
    pub levels: Vec<Level<'def>>,
}

pub enum Level<'def> {
    Package(*const Package<'def>),
    Class(*const Class<'def>),
    Local,
}

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
    pub fn wrap_package<F>(&mut self, package: &'r Package<'def>, func: F)
    where
        F: Fn(&mut Scope<'def, 'r>) -> (),
    {
        self.levels.push(Level::Package(package));
        func(self);
        self.levels.pop();
    }

    pub fn wrap_class<F>(&mut self, class: &'r Class<'def>, func: F)
    where
        F: Fn(&mut Scope<'def, 'r>) -> (),
    {
        self.levels.push(Level::Class(class));
        func(self);
        self.levels.pop();
    }

    pub fn wrap_local<F>(&mut self, func: F)
    where
        F: Fn(&mut Scope<'def, 'r>) -> (),
    {
        self.levels.push(Level::Local);
        func(self);
        self.levels.pop();
    }

    pub fn resolve_package(&self, name: &str) -> Option<*const Package<'def>> {
        for i in 0..self.levels.len() {
            let current = self.levels.get(self.levels.len() - 1 - i).unwrap();

            match current {
                Level::Package(package) => {
                    for package in unsafe { &(**package).subpackages } {
                        if package.name.as_str() == name {
                            return Some(package);
                        }
                    }
                }
                Level::Class(class) => (),
                Level::Local => (),
            };
        }

        for package in &self.root.subpackages {
            if package.name.as_str() == name {
                return Some(package);
            }
        }

        None
    }

    pub fn resolve_type(&self, name: &str) -> Option<EnclosingType<'def>> {
        for i in 0..self.levels.len() {
            let current = self.levels.get(self.levels.len() - 1 - i).unwrap();

            match current {
                Level::Package(package) => {
                    for class in unsafe { &(**package).classes } {
                        if class.name.fragment == name {
                            return Some(EnclosingType::Class(class));
                        }
                    }
                    for package in unsafe { &(**package).subpackages } {
                        if package.name.as_str() == name {
                            return Some(EnclosingType::Package(package));
                        }
                    }
                }
                Level::Class(class) => {
                    for subclass in unsafe { &(**class).classes } {
                        if subclass.name.fragment == name {
                            return Some(EnclosingType::Class(subclass));
                        }
                    }
                }
                Level::Local => (),
            };
        }

        for class in &self.root.classes {
            if class.name.fragment == name {
                return Some(EnclosingType::Class(class));
            }
        }
        for package in &self.root.subpackages {
            if package.name.as_str() == name {
                return Some(EnclosingType::Package(package));
            }
        }

        None
    }
}
