use kistelen::Secret;

#[derive(Secret)]
struct Card {
    #[secret(search = r"\d+", replacement = "x", replacement = "y")]
    number: String,
}

fn main() {}
