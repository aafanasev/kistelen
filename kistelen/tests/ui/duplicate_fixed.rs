use kistelen::Secret;

#[derive(Secret)]
struct Account {
    #[secret(fixed = 3, fixed = 8)]
    password: String,
}

fn main() {}
