use flex_value::{arr, obj, NULL};

fn arr() -> Option<()> {
    let arr = arr![1, 0.2, true, "string", NULL];

    let obj = obj! {
        "key" => "value",
        "key1" => "value1",
        "arr" => arr.clone(),
    };

    println!("{:#?}", obj);

    None
}

fn main() {
    arr();
}
