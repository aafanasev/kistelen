use kistelen::Secret;

#[derive(Secret)]
struct Code {
    #[secret(search = r"(\d{2})(\d{2})", replacement = "$3")]
    value: String,
}

fn main() {}
