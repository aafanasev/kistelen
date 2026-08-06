use kistelen::Secret;

#[derive(Secret)]
struct Account {
    #[secret = "***"]
    password: String,
}

fn main() {}
