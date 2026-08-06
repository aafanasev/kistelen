use kistelen::Secret;

#[derive(Secret)]
#[secret(skip)]
struct Account {
    password: String,
}

fn main() {}
