use serde::{Deserialize, Serialize};


#[derive(Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,           // 사용자 고유 ID
    pub user_name: String,
    pub user_address: String,
    pub github_account: String,
    pub x_account: String,
    pub tg_account: String,
    pub user_bio: String,
    pub user_avatar: Option<Vec<u8>>,  // 바이너리 이미지 데이터
    pub avatar_content_type: Option<String>,  // 이미지 MIME 타입 (예: "image/jpeg")
}
