use kistelen::Secret;

#[derive(Secret)]
struct Account {
    #[secret(partial, partial)]
    password: String,
}

fn main() {}
