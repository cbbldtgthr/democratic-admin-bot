#[derive(serde::Serialize)]
pub struct AddListing {
    pub user_telegram_id: u64,
    pub description: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct ListingResponse {
    pub id: i64,
    pub user_telegram_id: u64,
    pub description: String,
}
