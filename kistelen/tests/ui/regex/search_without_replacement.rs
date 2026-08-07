use kistelen::Secret;

#[derive(Secret)]
struct Card {
    #[secret(search = r"\d+")]
    number: String,
}

fn main() {}
