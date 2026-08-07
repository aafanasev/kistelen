use kistelen::Secret;

#[derive(Secret)]
struct Card {
    #[secret(search = r"([0-9]{4}", replacement = "x")]
    number: String,
}

fn main() {}
