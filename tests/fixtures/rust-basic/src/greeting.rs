// rust-language R-4 fixture: flat sibling module file - the second module
// form (`mod greeting;` resolves here, not to a directory).

pub fn greeting() -> String {
    "hello rust-basic".to_string()
}
