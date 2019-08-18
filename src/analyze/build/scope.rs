pub struct Scope {
    pub paths: Vec<String>,
}

impl Scope {
    pub fn wrap<F, V>(&mut self, name: &str, func: F) -> V
    where
        F: Fn(&mut Scope) -> V,
    {
        self.push(name);
        let result = func(self);
        self.pop();
        result
    }

    pub fn push(&mut self, name: &str) {
        self.paths.push(String::from(name));
    }

    pub fn pop(&mut self) {
        self.paths.pop();
    }

    pub fn pop_and_return<V>(&mut self, value: V) -> V {
        self.pop();
        value
    }

    pub fn get_import_path(&self) -> String {
        self.paths.join(".")
    }
}
