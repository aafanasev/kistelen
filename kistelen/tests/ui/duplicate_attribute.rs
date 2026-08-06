use kistelen::Secret;

#[derive(Secret)]
struct Account {
    #[secret]
    #[secret(skip)]
    password: String,
}

fn main() {}
