use axum::{http, Router};
use crate::router::*;
use crate::profile::*;
use crate::community::*;
use turtle_database::basic_db::{SafeDatabase, InnerDatabase};
use tower_http::cors::{Any, CorsLayer};

pub async fn build_server() {
    let shared_state = InnerDatabase::new(".").unwrap();
    let shared_state2 = Clone::clone(&shared_state);
    let components = collect_components();


    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::DELETE,
            http::Method::OPTIONS
        ])
        .allow_headers(Any)
        .allow_credentials(false);




    // Use just one type parameter
    let mut app = main_router(components, shared_state);

    let app = app.layer(cors);



    let listener = tokio::net::TcpListener::bind("0.0.0.0:443").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}



fn collect_components() ->  Vec<(String,Router<InnerDatabase>)> {
    let router_profile_post = post_router_builder("/api/profile".to_string(),profile_write::<InnerDatabase>);
    let router_profile_get = get_router_builder("/api/profile".to_string(),get_profile_by_address::<InnerDatabase>);
    // DAO PDA 관련 라우터
    let router_pda_post = post_router_builder("/api/dao/pda".to_string(), save_pda::<InnerDatabase>);
    let router_pda_get = get_router_builder("/api/dao/pdas".to_string(), get_all_pdas::<InnerDatabase>);

    // DAO Community 관련 라우터
    let router_community_post = post_router_builder("/api/dao/community".to_string(), save_community::<InnerDatabase>);
    let router_community_get_all = get_router_builder("/api/dao/communities".to_string(), get_all_communities::<InnerDatabase>);
    let router_community_get = get_router_builder("/api/dao/community".to_string(), get_community_by_pda::<InnerDatabase>);

    // DAO Content 관련 라우터
    let router_content_post = post_router_builder("/api/dao/content".to_string(), save_content::<InnerDatabase>);
    let router_content_get = get_router_builder("/api/dao/contents".to_string(), get_contents_by_pda::<InnerDatabase>);

    // DAO Depositor 관련 라우터
    let router_depositor_post = post_router_builder("/api/dao/depositor".to_string(), save_depositor::<InnerDatabase>);
    let router_depositor_get = get_router_builder("/api/dao/depositors".to_string(), get_depositors_by_pda::<InnerDatabase>);

    // DAO Proposal 관련 라우터
    let router_proposal_post = post_router_builder("/api/dao/proposal".to_string(), save_proposal::<InnerDatabase>);
    let router_proposal_get = get_router_builder("/api/dao/proposals".to_string(), get_proposals_by_pda::<InnerDatabase>);

    vec![
        // 프로필 라우터
        router_profile_get,
        router_profile_post,

        // DAO 라우터
        router_pda_post,
        router_pda_get,
        router_community_post,
        router_community_get_all,
        router_community_get,
        router_content_post,
        router_content_get,
        router_depositor_post,
        router_depositor_get,
        router_proposal_post,
        router_proposal_get
    ]

}