use unique_uuid::{unique_tag, UniqueTypeTag};

#[derive(UniqueTypeTag)]
pub struct Test;

fn main() {
    println!("Hello, world!");
    let test = unique_tag!("test1");
    println!("Tag for \"test1\": {:?}", test);

    let test2 = unique_tag!("test1");
    println!("Tag for \"test1\": {:?}", test2);

    let test3 = unique_tag!("test2");
    println!("Tag for \"test2\": {:?}", test3);

    // For types
    println!("Tag for type struct Test: {:?}", Test::TYPE_TAG);
}
