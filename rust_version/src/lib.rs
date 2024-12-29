#[derive(serde::Serialize)]
pub struct AddListing {
    pub description: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct ListingResponse {
    pub id: i64,
    pub description: String,
}
