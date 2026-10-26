// rust-language R-4 fixture: nested module file under the mod.rs sibling.

#[derive(Debug)]
pub struct User {
    pub name: String,
}

pub fn default_user() -> User {
    User {
        name: "ariadne".to_string(),
    }
}
