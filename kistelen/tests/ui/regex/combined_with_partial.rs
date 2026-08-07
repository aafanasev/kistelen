use kistelen::Secret;

#[derive(Secret)]
struct Card {
    #[secret(partial, search = r"\d+", replacement = "x")]
    number: String,
}

fn main() {}
