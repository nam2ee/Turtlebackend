use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Community {
    pub admin: String,                  // DAO 관리자 공개키
    pub time_limit: u64,                // 시간 제한(초)
    pub base_fee: u64,                  // 기본 수수료(lamports)
    pub ai_moderation: bool,            // AI 모더레이션 활성화 여부
    pub deposit_share: u8,              // 고품질 콘텐츠에 분배되는 예치금 비율(0-100%)
    pub last_activity_timestamp: u64,   // 마지막 활동 타임스탬프
    pub total_deposit: u64,             // 총 예치금액
    pub active_proposal_count: u64,     // 활성 제안 수
    pub content_count: u64,             // 콘텐츠 수
    pub depositor_count: u64,           // 예치자 수
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Content {
    pub author: String,                 // 작성자 공개키
    pub content_hash: String,           // 콘텐츠 해시(텍스트 + 이미지 참조)
    pub content_uri: String,            // 콘텐츠 상세 URI(예: IPFS 링크)
    pub timestamp: u64,                 // 생성 타임스탬프
    pub votes: u64,                     // 투표 수
}



#[derive(Clone, Serialize, Deserialize)]
pub struct Depositor {
    pub pubkey: String,                 // 예치자 공개키
    pub amount: u64,                    // 예치 금액
    pub locked_until: u64,              // 잠금 해제 시간
    pub voting_power: u64,              // 투표 파워(예치금액 기반)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,                        // 제안 ID
    pub proposal_type: u8,    // 제안 유형(TimeLimit - 0, BaseFee - 1, AiModeration - 2)
    pub new_value: u64,                 // 새 값
    pub voting_end_time: u64,           // 투표 종료 시간
    pub yes_votes: u64,                 // 찬성표
    pub no_votes: u64,                  // 반대표
    pub is_executed: bool,              // 실행 여부
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Daopda{
    pub address: String            // 실행 여부
}
