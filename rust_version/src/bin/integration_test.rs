use rust_version::{AddListing, ListingResponse};

fn add_listing() {
    let description = "My test item".into();
    println!("Sending AddListing");
    let response = reqwest::blocking::Client::new()
        .post("http://127.0.0.1:3000/listing")
        .json(&AddListing { description })
        .send();

    println!("{:?}", response);
}
fn get_listings() {
    println!("Sending GetListings");
    let response = reqwest::blocking::get("http://127.0.0.1:3000/listings")
        .unwrap()
        .json::<Vec<ListingResponse>>()
        .unwrap();
    println!("{:?}", response);
}

fn main() {
    get_listings();
}
