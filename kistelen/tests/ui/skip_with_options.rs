use kistelen::Secret;

#[derive(Secret)]
#[secret]
struct Card {
    #[secret(skip, with = "x")]
    number: String,
}

fn main() {}
