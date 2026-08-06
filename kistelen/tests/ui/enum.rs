use kistelen::Secret;

#[derive(Secret)]
enum Credential {
    Password(String),
    Token { value: String },
}

fn main() {}
