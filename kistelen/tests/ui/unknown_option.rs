use kistelen::Secret;

#[derive(Secret)]
struct Account {
    #[secret(hidden)]
    password: String,
}

fn main() {}
