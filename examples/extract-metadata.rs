use oasysdb::prelude::*;

fn main() {
    // Inserting a metadata value into a record.
    let data: &str = "This is an example.";
    let vector = Vector::random(128);
    let record = Record::new(&vector, &data.into());

    // Extracting the metadata value.
    let metadata = record.data.clone();
    let data = match metadata {
        Metadata::Text(value) => value,
        _ => panic!("Data is not a text."),
    };

    println!("{}", data);
}
