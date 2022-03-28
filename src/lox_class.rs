#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: &String) -> Self {
        Self { name: name.clone() }
    }
}

impl std::string::ToString for LoxClass {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}
