use axum::extract::{Multipart, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::error::Error as StdError;
use std::fmt;
use axum::Json;
use serde::{Deserialize, Serialize};
use turtle_database::basic_db::{SafeDatabase};
use turtle_service::parser::profile::UserProfile;

// Query parameters struct for the get_profile_by_address endpoint
#[derive(Deserialize)]
pub struct AddressQuery {
    address: String,
}

// Response struct for the get_profile_by_address endpoint


#[derive(Debug)]
pub enum ProfileError {
    MultipartError(String),
    DatabaseError(String),
    SerializationError(String),
}

// ProfileError에 Display 트레이트 구현 (Error 트레이트 구현에 필요)
impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileError::MultipartError(msg) => write!(f, "Multipart error: {}", msg),
            ProfileError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ProfileError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

// ProfileError에 std::error::Error 트레이트 구현
impl StdError for ProfileError {}

// ProfileError에 IntoResponse 트레이트 구현
impl IntoResponse for ProfileError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ProfileError::MultipartError(msg) => (StatusCode::BAD_REQUEST, msg),
            ProfileError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ProfileError::SerializationError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        // 에러 메시지와 상태 코드 반환
        (status, error_message).into_response()
    }
}

pub async fn profile_write<T: SafeDatabase>(
    State(database): State<T>,
    mut multipart: Multipart
) -> Result<StatusCode, ProfileError>
{
    // 사용자 프로필 데이터 초기화
    let mut user_profile = UserProfile {
        user_id: String::new(),
        user_name: String::new(),
        user_address: String::new(),
        github_account: String::new(),
        x_account: String::new(),
        tg_account: String::new(),
        user_bio: String::new(),
        user_avatar: None,
        avatar_content_type: None,
    };

    // multipart 필드 처리
    while let Some(field) = multipart.next_field().await.map_err(|e| ProfileError::MultipartError(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "user_id" => {
                user_profile.user_id = field.text().await.map_err(|e| ProfileError::MultipartError(e.to_string()))?;
            },
            "user_name" => {
                user_profile.user_name = field.text().await.map_err(|e| ProfileError::MultipartError(e.to_string()))?;
            },
            "user_address" => {
                user_profile.user_address = field.text().await.map_err(|e| ProfileError::MultipartError(e.to_string()))?;
            },
            "github_account" => {
                user_profile.github_account = field.text().await.map_err(|e| ProfileError::MultipartError(e.to_string()))?;
            },
            "x_account" => {
                user_profile.x_account = field.text().await.map_err(|e| ProfileError::MultipartError(e.to_string()))?;
            },
            "tg_account" => {
                user_profile.tg_account = field.text().await.map_err(|e| ProfileError::MultipartError(e.to_string()))?;
            },
            "user_bio" => {
                user_profile.user_bio = field.text().await.map_err(|e| ProfileError::MultipartError(e.to_string()))?;
            },
            "user_avatar" => {
                // 이미지 데이터 처리
                let content_type = field.content_type().map(|ct| ct.to_string());
                let data = field.bytes().await.map_err(|e| ProfileError::MultipartError(e.to_string()))?;

                if !data.is_empty() {
                    user_profile.user_avatar = Some(data.to_vec());
                    user_profile.avatar_content_type = content_type;
                }
            },
            _ => continue,
        }
    }

    if user_profile.user_address.is_empty() {
        return Err(ProfileError::MultipartError("User ID is required".to_string()));
    }

    let profile_json = serde_json::to_string(&user_profile)
        .map_err(|e| ProfileError::SerializationError(e.to_string()))?;

    database.write(&user_profile.user_address, &profile_json, "user_profiles")
        .map_err(|e| ProfileError::DatabaseError(e.to_string()))?;

    Ok(StatusCode::OK)
}


pub async fn get_profile_by_address<T: SafeDatabase>(
    State(database): State<T>,
    Query(query): Query<AddressQuery>,
) -> Result<Json<UserProfile>, ProfileError> {
    // Validate address
    if query.address.is_empty() {
        return Err(ProfileError::MultipartError("Address is required".to_string()));
    }

    // Try to read the profile from the database
    let profile_data = database.read(&query.address, "user_profiles")
        .map_err(|e| ProfileError::DatabaseError(e.to_string()))?;

    // Check if the profile exists
    if let Some(data) = profile_data {
        // Parse the profile from JSON
        let profile_str = String::from_utf8(data)
            .map_err(|e| ProfileError::SerializationError(format!("Invalid UTF-8: {}", e)))?;

        let profile: UserProfile = serde_json::from_str(&profile_str)
            .map_err(|e| ProfileError::SerializationError(format!("Invalid JSON: {}", e)))?;

        // Return the existing profile
        Ok(Json(
            profile))
    } else {
        // Create a default profile with only the address field
        let default_profile = UserProfile {
            user_id: String::new(),
            user_name: String::new(),
            user_address: query.address.clone(),
            github_account: String::new(),
            x_account: String::new(),
            tg_account: String::new(),
            user_bio: String::new(),
            user_avatar: None,
            avatar_content_type: None,
        };

        // Return the default profile
        Ok(Json(default_profile))
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::extract::FromRequest;
    use axum::http::Request;
    use tempfile::tempdir;
    use turtle_database::basic_db::InnerDatabase;
    use turtle_service::parser::profile::UserProfile;

    use axum::extract::Query;


    // 테스트용 멀티파트 바디 생성 함수
    fn create_multipart_body(fields: Vec<(&str, &str)>, file_field: Option<(&str, &str, &[u8])>) -> (String, Vec<u8>) {
        let boundary = "test_boundary";
        let mut body = Vec::new();

        // 텍스트 필드 추가
        for (name, value) in fields {
            body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            body.extend_from_slice(format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", name).as_bytes());
            body.extend_from_slice(value.as_bytes());
            body.extend_from_slice(b"\r\n");
        }

        // 파일 필드 추가 (있는 경우)
        if let Some((name, filename, data)) = file_field {
            body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            body.extend_from_slice(format!(
                "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                name, filename
            ).as_bytes());
            body.extend_from_slice(b"Content-Type: image/jpeg\r\n\r\n");
            body.extend_from_slice(data);
            body.extend_from_slice(b"\r\n");
        }

        // 종료 경계 추가
        body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

        (format!("multipart/form-data; boundary={}", boundary), body)
    }

    #[tokio::test]
    async fn test_profile_write_success() -> Result<(), Box<dyn std::error::Error>> {
        // 임시 디렉토리 생성
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");

        // 데이터베이스 초기화
        let db = InnerDatabase::new(&db_path)?;

        // 테스트용 멀티파트 데이터 생성
        let fields = vec![
            ("user_id", "test_user"),
            ("user_name", "Test User"),
            ("user_address", "0xabcdef123456789"),
            ("github_account", "testuser"),
            ("x_account", "@testuser"),
            ("tg_account", "@test_user"),
            ("user_bio", "This is a test bio"),
        ];

        // 테스트용 아바타 이미지 데이터
        let avatar_data = [1, 2, 3, 4, 5]; // 간단한 바이너리 데이터
        let file_field = Some(("user_avatar", "avatar.jpg", &avatar_data[..]));

        let (content_type, body_bytes) = create_multipart_body(fields, file_field);

        // Multipart 추출기 생성
        let request = Request::builder()
            .header("content-type", content_type)
            .body(Body::from(body_bytes))?;

        let multipart = Multipart::from_request(request, &()).await?;

        // profile_write 함수 호출
        let result = profile_write(State(db.clone()), multipart).await?;

        // 결과 확인 - 성공해야 함
        assert_eq!(result, StatusCode::OK);

        // 데이터베이스에서 저장된 프로필 읽기
        let profile_data = db.read("0xabcdef123456789", "user_profiles")?;
        assert!(profile_data.is_some(), "Profile data not found in database");

        // 저장된 데이터 검증
        if let Some(data) = profile_data {
            let profile_str = String::from_utf8(data)?;
            let profile: UserProfile = serde_json::from_str(&profile_str)?;

            assert_eq!(profile.user_id, "test_user");
            assert_eq!(profile.user_name, "Test User");
            assert_eq!(profile.user_address, "0xabcdef123456789");
            assert_eq!(profile.github_account, "testuser");
            assert_eq!(profile.x_account, "@testuser");
            assert_eq!(profile.tg_account, "@test_user");
            assert_eq!(profile.user_bio, "This is a test bio");

            // 아바타 검증
            assert!(profile.user_avatar.is_some());
            if let Some(avatar) = &profile.user_avatar {
                assert_eq!(avatar, &avatar_data);
            }
            assert_eq!(profile.avatar_content_type, Some("image/jpeg".to_string()));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_profile_write_missing_address() -> Result<(), Box<dyn std::error::Error>> {
        // 임시 디렉토리 생성
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");

        // 데이터베이스 초기화
        let db = InnerDatabase::new(&db_path)?;

        // user_address가 없는 멀티파트 데이터 생성
        let fields = vec![
            ("user_id", "test_user"),
            ("user_name", "Test User"),
            // user_address 필드 생략
            ("github_account", "testuser"),
        ];

        let (content_type, body_bytes) = create_multipart_body(fields, None);

        // Multipart 추출기 생성
        let request = Request::builder()
            .header("content-type", content_type)
            .body(Body::from(body_bytes))?;

        let multipart = Multipart::from_request(request, &()).await?;

        // profile_write 함수 호출 - 여기서는 에러를 기대하므로 ? 연산자를 사용하지 않음
        let result = profile_write(State(db.clone()), multipart).await;

        // 결과 확인 - 에러가 발생해야 함
        match result {
            Err(ProfileError::MultipartError(msg)) => {
                assert_eq!(msg, "User ID is required");
                Ok(())
            },
            _ => Err("Expected MultipartError with 'User ID is required' message".into()),
        }
    }

    #[tokio::test]
    async fn test_profile_write_empty_fields() -> Result<(), Box<dyn std::error::Error>> {
        // 임시 디렉토리 생성
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");

        // 데이터베이스 초기화
        let db = InnerDatabase::new(&db_path)?;

        // 일부 필드가 빈 멀티파트 데이터 생성
        let fields = vec![
            ("user_id", ""),
            ("user_name", ""),
            ("user_address", "0xabcdef123456789"), // 이 필드만 값이 있음
            ("github_account", ""),
            ("x_account", ""),
            ("tg_account", ""),
            ("user_bio", ""),
        ];

        let (content_type, body_bytes) = create_multipart_body(fields, None);

        // Multipart 추출기 생성
        let request = Request::builder()
            .header("content-type", content_type)
            .body(Body::from(body_bytes))?;

        let multipart = Multipart::from_request(request, &()).await?;

        // profile_write 함수 호출
        let result = profile_write(State(db.clone()), multipart).await?;

        // 결과 확인 - 성공해야 함 (user_address가 있으므로)
        assert_eq!(result, StatusCode::OK);

        // 데이터베이스에서 저장된 프로필 읽기
        let profile_data = db.read("0xabcdef123456789", "user_profiles")?;
        assert!(profile_data.is_some(), "Profile data not found in database");

        // 저장된 데이터 검증
        if let Some(data) = profile_data {
            let profile_str = String::from_utf8(data)?;
            let profile: UserProfile = serde_json::from_str(&profile_str)?;

            assert_eq!(profile.user_id, "");
            assert_eq!(profile.user_name, "");
            assert_eq!(profile.user_address, "0xabcdef123456789");
            assert_eq!(profile.github_account, "");
            assert_eq!(profile.x_account, "");
            assert_eq!(profile.tg_account, "");
            assert_eq!(profile.user_bio, "");
            assert!(profile.user_avatar.is_none());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_profile_by_address_existing() -> Result<(), Box<dyn std::error::Error>> {
        // Create temporary directory
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");

        // Initialize database
        let db = InnerDatabase::new(&db_path)?;

        // Create a test profile
        let test_address = "0xabcdef123456789";
        let test_profile = UserProfile {
            user_id: "test_user".to_string(),
            user_name: "Test User".to_string(),
            user_address: test_address.to_string(),
            github_account: "testuser".to_string(),
            x_account: "@testuser".to_string(),
            tg_account: "@test_user".to_string(),
            user_bio: "This is a test bio".to_string(),
            user_avatar: None,
            avatar_content_type: None,
        };

        // Save the profile to the database
        let profile_json = serde_json::to_string(&test_profile)?;
        db.write(test_address, &profile_json, "user_profiles")?;

        // Create query parameters
        let query = AddressQuery {
            address: test_address.to_string(),
        };

        // Call get_profile_by_address function
        let result = get_profile_by_address(State(db), Query(query)).await?;

        // Check the result
        let response = result.0;
        assert!(response.exists);
        assert_eq!(response.profile.user_id, "test_user");
        assert_eq!(response.profile.user_name, "Test User");
        assert_eq!(response.profile.user_address, test_address);
        assert_eq!(response.profile.github_account, "testuser");
        assert_eq!(response.profile.x_account, "@testuser");
        assert_eq!(response.profile.tg_account, "@test_user");
        assert_eq!(response.profile.user_bio, "This is a test bio");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_profile_by_address_nonexistent() -> Result<(), Box<dyn std::error::Error>> {
        // Create temporary directory
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");

        // Initialize database
        let db = InnerDatabase::new(&db_path)?;

        // Create query parameters for a non-existent address
        let test_address = "0xnonexistent123";
        let query = AddressQuery {
            address: test_address.to_string(),
        };

        // Call get_profile_by_address function
        let result = get_profile_by_address(State(db), Query(query)).await?;

        // Check the result
        let response = result.0;
        assert!(!response.exists);
        assert_eq!(response.profile.user_address, test_address);
        assert!(response.profile.user_id.is_empty());
        assert!(response.profile.user_name.is_empty());
        assert!(response.profile.github_account.is_empty());
        assert!(response.profile.x_account.is_empty());
        assert!(response.profile.tg_account.is_empty());
        assert!(response.profile.user_bio.is_empty());
        assert!(response.profile.user_avatar.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_profile_by_address_empty_address() -> Result<(), Box<dyn std::error::Error>> {
        // Create temporary directory
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_db");

        // Initialize database
        let db = InnerDatabase::new(&db_path)?;

        // Create query parameters with an empty address
        let query = AddressQuery {
            address: "".to_string(),
        };

        // Call get_profile_by_address function
        let result = get_profile_by_address(State(db), Query(query)).await;

        // Check that it returns an error
        match result {
            Err(ProfileError::MultipartError(msg)) => {
                assert_eq!(msg, "Address is required");
                Ok(())
            },
            _ => Err("Expected MultipartError with 'Address is required' message".into()),
        }
    }

}
