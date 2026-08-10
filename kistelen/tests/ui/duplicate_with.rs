use kistelen::Secret;

#[derive(Secret)]
struct Account {
    #[secret(with = "FIRST", with = "SECOND")]
    password: String,
}

fn main() {}
