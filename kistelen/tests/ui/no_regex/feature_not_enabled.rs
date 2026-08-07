use kistelen::Secret;

#[derive(Secret)]
struct Card {
    #[secret(search = r"\d+", replacement = "x")]
    number: String,
}

fn main() {}
