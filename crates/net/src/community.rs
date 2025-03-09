use axum::extract::{Multipart, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::error::Error as StdError;
use std::fmt;
use axum::Json;
use serde::{Deserialize, Serialize};
use turtle_database::basic_db::{SafeDatabase};
use turtle_service::parser::community::{Community, Content, Depositor, Proposal, Daopda};
use std::collections::HashMap;

// 다양한 쿼리 파라미터를 위한 구조체들
#[derive(Deserialize)]
pub struct PdaQuery {
    pda: String,
}

#[derive(Deserialize)]
pub struct ContentCreateQuery {
    pda: String,
}

#[derive(Deserialize)]
pub struct DepositorCreateQuery {
    pda: String,
}

#[derive(Deserialize)]
pub struct ProposalCreateQuery {
    pda: String,
}

// 응답 구조체들
#[derive(Serialize)]
pub struct PdasResponse {
    pdas: Vec<String>,
}

#[derive(Serialize)]
pub struct CommunitiesResponse {
    communities: Vec<Community>,
}

#[derive(Serialize)]
pub struct ContentsResponse {
    contents: Vec<Content>,
}

#[derive(Serialize)]
pub struct DepositorsResponse {
    depositors: Vec<Depositor>,
}

#[derive(Serialize)]
pub struct ProposalsResponse {
    proposals: Vec<Proposal>,
}

// 에러 타입
#[derive(Debug)]
pub enum DaoError {
    MultipartError(String),
    DatabaseError(String),
    SerializationError(String),
    ValidationError(String),
}

impl fmt::Display for DaoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DaoError::MultipartError(msg) => write!(f, "Multipart error: {}", msg),
            DaoError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            DaoError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            DaoError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl StdError for DaoError {}

impl IntoResponse for DaoError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            DaoError::MultipartError(msg) => (StatusCode::BAD_REQUEST, msg),
            DaoError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            DaoError::SerializationError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            DaoError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        (status, error_message).into_response()
    }
}

// DAOPDA 테이블 관련 함수들
pub async fn save_pda<T: SafeDatabase>(
    State(database): State<T>,
    Json(daopda): Json<Daopda>,
) -> Result<StatusCode, DaoError> {
    // PDA 유효성 검사
    if daopda.address.is_empty() {
        return Err(DaoError::ValidationError("PDA cannot be empty".to_string()));
    }

    // 데이터베이스에 저장 - key와 value 모두 PDA
    database.write(&daopda.address, &daopda.address, "daopda")
        .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

    Ok(StatusCode::OK)
}

pub async fn get_all_pdas<T: SafeDatabase>(
    State(database): State<T>,
) -> Result<Json<PdasResponse>, DaoError> {
    // 데이터베이스에서 모든 PDA 읽기
    let pda_entries: HashMap<Vec<u8>, Vec<u8>> = database.read_all("daopda")
        .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

    let mut pdas = Vec::new();
    for (key_bytes, _) in pda_entries {
        // key_bytes를 문자열로 변환
        let key_str = String::from_utf8(key_bytes)
            .map_err(|e| DaoError::SerializationError(format!("Invalid UTF-8 in key: {}", e)))?;
        pdas.push(key_str);
    }

    Ok(Json(PdasResponse { pdas }))
}

// COMMUNITY 테이블 관련 함수들
pub async fn save_community<T: SafeDatabase>(
    State(database): State<T>,
    Query(query): Query<PdaQuery>,
    Json(community): Json<Community>,
) -> Result<StatusCode, DaoError> {
    // PDA 유효성 검사
    if query.pda.is_empty() {
        return Err(DaoError::ValidationError("PDA cannot be empty".to_string()));
    }

    // JSON 직렬화
    let community_json = serde_json::to_string(&community)
        .map_err(|e| DaoError::SerializationError(e.to_string()))?;

    // 데이터베이스에 저장 - key는 PDA, value는 Community
    database.write(&query.pda, &community_json, "community")
        .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

    Ok(StatusCode::OK)
}

pub async fn get_all_communities<T: SafeDatabase>(
    State(database): State<T>,
) -> Result<Json<CommunitiesResponse>, DaoError> {
    // 데이터베이스에서 모든 커뮤니티 읽기
    let community_entries: HashMap<Vec<u8>, Vec<u8>> = database.read_all("community")
        .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

    let mut communities = Vec::new();
    for (_, value_bytes) in community_entries {
        let community_data = String::from_utf8(value_bytes)
            .map_err(|e| DaoError::SerializationError(format!("Invalid UTF-8: {}", e)))?;

        let community: Community = serde_json::from_str(&community_data)
            .map_err(|e| DaoError::SerializationError(format!("Invalid JSON: {}", e)))?;

        communities.push(community);
    }

    Ok(Json(CommunitiesResponse { communities }))
}

pub async fn get_community_by_pda<T: SafeDatabase>(
    State(database): State<T>,
    Query(query): Query<PdaQuery>,
) -> Result<Json<Community>, DaoError> {
    // PDA 유효성 검사
    if query.pda.is_empty() {
        return Err(DaoError::ValidationError("PDA cannot be empty".to_string()));
    }

    // 데이터베이스에서 커뮤니티 읽기
    let community_data = database.read(&query.pda, "community")
        .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

    if let Some(data) = community_data {
        let community_str = String::from_utf8(data)
            .map_err(|e| DaoError::SerializationError(format!("Invalid UTF-8: {}", e)))?;

        let community: Community = serde_json::from_str(&community_str)
            .map_err(|e| DaoError::SerializationError(format!("Invalid JSON: {}", e)))?;

        Ok(Json(community))
    } else {
        Err(DaoError::ValidationError(format!("Community with PDA {} not found", query.pda)))
    }
}

// CONTENT 테이블 관련 함수들
pub async fn save_content<T: SafeDatabase>(
    State(database): State<T>,
    Query(query): Query<ContentCreateQuery>,
    Json(content): Json<Content>,
) -> Result<StatusCode, DaoError> {
    // PDA 유효성 검사
    if query.pda.is_empty() {
        return Err(DaoError::ValidationError("PDA cannot be empty".to_string()));
    }

    // 커뮤니티 조회하여 content_count 및 last_activity_timestamp 업데이트
    let community_data = database.read(&query.pda, "community")
        .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

    if let Some(data) = community_data {
        let community_str = String::from_utf8(data.clone())
            .map_err(|e| DaoError::SerializationError(format!("Invalid UTF-8: {}", e)))?;

        let mut community: Community = serde_json::from_str(&community_str)
            .map_err(|e| DaoError::SerializationError(format!("Invalid JSON: {}", e)))?;

        // content_count 증가
        community.content_count += 1;

        // last_activity_timestamp 업데이트
        community.last_activity_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 콘텐츠 키 생성 (pda_n 형식)
        let content_key = format!("{}_{}", query.pda, community.content_count);

        // 콘텐츠 JSON 직렬화
        let content_json = serde_json::to_string(&content)
            .map_err(|e| DaoError::SerializationError(e.to_string()))?;

        // 업데이트된 커뮤니티 JSON 직렬화
        let updated_community_json = serde_json::to_string(&community)
            .map_err(|e| DaoError::SerializationError(e.to_string()))?;

        // 데이터베이스에 콘텐츠 저장
        database.write(&content_key, &content_json, "content")
            .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

        // 데이터베이스에 업데이트된 커뮤니티 정보 저장
        database.write(&query.pda, &updated_community_json, "community")
            .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

        Ok(StatusCode::OK)
    } else {
        Err(DaoError::ValidationError(format!("Community with PDA {} not found", query.pda)))
    }
}

pub async fn get_contents_by_pda<T: SafeDatabase>(
    State(database): State<T>,
    Query(query): Query<PdaQuery>,
) -> Result<Json<ContentsResponse>, DaoError> {
    // PDA 유효성 검사
    if query.pda.is_empty() {
        return Err(DaoError::ValidationError("PDA cannot be empty".to_string()));
    }

    // 데이터베이스에서 모든 콘텐츠 읽기
    let content_entries: HashMap<Vec<u8>, Vec<u8>> = database.read_all("content")
        .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

    // PDA에 해당하는 콘텐츠만 필터링
    let prefix = format!("{}_", query.pda);
    let mut contents = Vec::new();

    for (key_bytes, value_bytes) in content_entries {
        // 바이너리 키를 문자열로 변환
        let key_str = match String::from_utf8(key_bytes) {
            Ok(s) => s,
            Err(_) => continue, // UTF-8이 아닌 키는 건너뜀
        };

        if key_str.starts_with(&prefix) {
            let content_str = match String::from_utf8(value_bytes) {
                Ok(s) => s,
                Err(e) => return Err(DaoError::SerializationError(format!("Invalid UTF-8: {}", e))),
            };

            let content: Content = serde_json::from_str(&content_str)
                .map_err(|e| DaoError::SerializationError(format!("Invalid JSON: {}", e)))?;

            contents.push(content);
        }
    }

    Ok(Json(ContentsResponse { contents }))
}

// DEPOSIT 테이블 관련 함수들
pub async fn save_depositor<T: SafeDatabase>(
    State(database): State<T>,
    Query(query): Query<DepositorCreateQuery>,
    Json(depositor): Json<Depositor>,
) -> Result<StatusCode, DaoError> {
    // PDA 유효성 검사
    if query.pda.is_empty() {
        return Err(DaoError::ValidationError("PDA cannot be empty".to_string()));
    }

    // 커뮤니티 조회하여 depositor_count 및 last_activity_timestamp 업데이트
    let community_data = database.read(&query.pda, "community")
        .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

    if let Some(data) = community_data {
        let community_str = String::from_utf8(data.clone())
            .map_err(|e| DaoError::SerializationError(format!("Invalid UTF-8: {}", e)))?;

        let mut community: Community = serde_json::from_str(&community_str)
            .map_err(|e| DaoError::SerializationError(format!("Invalid JSON: {}", e)))?;

        // depositor_count 증가
        community.depositor_count += 1;

        // last_activity_timestamp 업데이트
        community.last_activity_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // depositor 키 생성 (pda_n 형식)
        let depositor_key = format!("{}_{}", query.pda, community.depositor_count);

        // depositor JSON 직렬화
        let depositor_json = serde_json::to_string(&depositor)
            .map_err(|e| DaoError::SerializationError(e.to_string()))?;

        // 업데이트된 커뮤니티 JSON 직렬화
        let updated_community_json = serde_json::to_string(&community)
            .map_err(|e| DaoError::SerializationError(e.to_string()))?;

        // 데이터베이스에 depositor 저장
        database.write(&depositor_key, &depositor_json, "depositor")
            .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

        // 데이터베이스에 업데이트된 커뮤니티 정보 저장
        database.write(&query.pda, &updated_community_json, "community")
            .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

        Ok(StatusCode::OK)
    } else {
        Err(DaoError::ValidationError(format!("Community with PDA {} not found", query.pda)))
    }
}

pub async fn get_depositors_by_pda<T: SafeDatabase>(
    State(database): State<T>,
    Query(query): Query<PdaQuery>,
) -> Result<Json<DepositorsResponse>, DaoError> {
    // PDA 유효성 검사
    if query.pda.is_empty() {
        return Err(DaoError::ValidationError("PDA cannot be empty".to_string()));
    }

    // 데이터베이스에서 모든 depositor 읽기
    let depositor_entries: HashMap<Vec<u8>, Vec<u8>> = database.read_all("depositor")
        .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

    // PDA에 해당하는 depositor만 필터링
    let prefix = format!("{}_", query.pda);
    let mut depositors = Vec::new();

    for (key_bytes, value_bytes) in depositor_entries {
        // 바이너리 키를 문자열로 변환
        let key_str = match String::from_utf8(key_bytes) {
            Ok(s) => s,
            Err(_) => continue, // UTF-8이 아닌 키는 건너뜀
        };

        if key_str.starts_with(&prefix) {
            let depositor_str = match String::from_utf8(value_bytes) {
                Ok(s) => s,
                Err(e) => return Err(DaoError::SerializationError(format!("Invalid UTF-8: {}", e))),
            };

            let depositor: Depositor = serde_json::from_str(&depositor_str)
                .map_err(|e| DaoError::SerializationError(format!("Invalid JSON: {}", e)))?;

            depositors.push(depositor);
        }
    }

    Ok(Json(DepositorsResponse { depositors }))
}

// PROPOSAL 테이블 관련 함수들
pub async fn save_proposal<T: SafeDatabase>(
    State(database): State<T>,
    Query(query): Query<ProposalCreateQuery>,
    Json(proposal): Json<Proposal>,
) -> Result<StatusCode, DaoError> {
    // PDA 유효성 검사
    if query.pda.is_empty() {
        return Err(DaoError::ValidationError("PDA cannot be empty".to_string()));
    }

    // 커뮤니티 조회하여 active_proposal_count 및 last_activity_timestamp 업데이트
    let community_data = database.read(&query.pda, "community")
        .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

    if let Some(data) = community_data {
        let community_str = String::from_utf8(data.clone())
            .map_err(|e| DaoError::SerializationError(format!("Invalid UTF-8: {}", e)))?;

        let mut community: Community = serde_json::from_str(&community_str)
            .map_err(|e| DaoError::SerializationError(format!("Invalid JSON: {}", e)))?;

        // active_proposal_count 증가
        community.active_proposal_count += 1;

        // last_activity_timestamp 업데이트
        community.last_activity_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // proposal 키 생성 (pda_n 형식)
        let proposal_key = format!("{}_{}", query.pda, community.active_proposal_count);

        // proposal JSON 직렬화
        let proposal_json = serde_json::to_string(&proposal)
            .map_err(|e| DaoError::SerializationError(e.to_string()))?;

        // 업데이트된 커뮤니티 JSON 직렬화
        let updated_community_json = serde_json::to_string(&community)
            .map_err(|e| DaoError::SerializationError(e.to_string()))?;

        // 데이터베이스에 proposal 저장
        database.write(&proposal_key, &proposal_json, "proposal")
            .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

        // 데이터베이스에 업데이트된 커뮤니티 정보 저장
        database.write(&query.pda, &updated_community_json, "community")
            .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

        Ok(StatusCode::OK)
    } else {
        Err(DaoError::ValidationError(format!("Community with PDA {} not found", query.pda)))
    }
}

pub async fn get_proposals_by_pda<T: SafeDatabase>(
    State(database): State<T>,
    Query(query): Query<PdaQuery>,
) -> Result<Json<ProposalsResponse>, DaoError> {
    // PDA 유효성 검사
    if query.pda.is_empty() {
        return Err(DaoError::ValidationError("PDA cannot be empty".to_string()));
    }

    // 데이터베이스에서 모든 proposal 읽기
    let proposal_entries: HashMap<Vec<u8>, Vec<u8>> = database.read_all("proposal")
        .map_err(|e| DaoError::DatabaseError(e.to_string()))?;

    // PDA에 해당하는 proposal만 필터링
    let prefix = format!("{}_", query.pda);
    let mut proposals = Vec::new();

    for (key_bytes, value_bytes) in proposal_entries {
        // 바이너리 키를 문자열로 변환
        let key_str = match String::from_utf8(key_bytes) {
            Ok(s) => s,
            Err(_) => continue, // UTF-8이 아닌 키는 건너뜀
        };

        if key_str.starts_with(&prefix) {
            let proposal_str = match String::from_utf8(value_bytes) {
                Ok(s) => s,
                Err(e) => return Err(DaoError::SerializationError(format!("Invalid UTF-8: {}", e))),
            };

            let proposal: Proposal = serde_json::from_str(&proposal_str)
                .map_err(|e| DaoError::SerializationError(format!("Invalid JSON: {}", e)))?;

            proposals.push(proposal);
        }
    }

    Ok(Json(ProposalsResponse { proposals }))
}

