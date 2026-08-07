use kistelen::Secret;

#[derive(Secret)]
struct Card {
    #[secret(fixed = 0)]
    number: String,
}

fn main() {}
