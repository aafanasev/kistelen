use kistelen::Secret;

#[derive(Secret)]
union Raw {
    integer: u32,
    float: f32,
}

fn main() {}
