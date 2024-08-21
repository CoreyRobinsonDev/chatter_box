#[derive(serde::Serialize)]
pub struct MessageOut {
    pub text: String,
    pub user: String,
    pub date: chrono::DateTime<chrono::Utc>
}


