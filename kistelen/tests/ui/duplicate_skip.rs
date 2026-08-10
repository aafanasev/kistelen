use kistelen::Secret;

#[derive(Secret)]
#[secret]
struct Account {
    #[secret(skip, skip)]
    id: u32,
}

fn main() {}
