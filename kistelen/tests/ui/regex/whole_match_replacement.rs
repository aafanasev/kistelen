use kistelen::Secret;

#[derive(Secret)]
struct Card {
    #[secret(search = r"\d+", replacement = "$0")]
    number: String,
}

fn main() {}
