use kistelen::Secret;

#[derive(Secret)]
struct Card {
    #[secret(search = r"\d+", search = r"\w+", replacement = "x")]
    number: String,
}

fn main() {}
