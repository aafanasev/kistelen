use kistelen::Secret;

#[derive(Secret)]
struct Card {
    #[secret(fixed = 4, with = "xy")]
    number: String,
}

fn main() {}
