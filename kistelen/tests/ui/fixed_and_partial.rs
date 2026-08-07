use kistelen::Secret;

#[derive(Secret)]
struct Card {
    #[secret(fixed = 4, partial)]
    number: String,
}

fn main() {}
