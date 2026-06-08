use quantix_cli::db::TDengineClient;
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const BASIC_ROOT_TAOSDATA: &str = "Basic cm9vdDp0YW9zZGF0YQ==";

#[tokio::test]
async fn execute_sql_uses_basic_auth_rest_sql_and_plain_sql_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/sql"))
        .and(header("authorization", BASIC_ROOT_TAOSDATA))
        .and(body_string_contains("show databases"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "rows": 0,
            "data": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = TDengineClient::new(&server.uri(), "root:taosdata").unwrap();

    client.execute_sql("show databases").await.unwrap();
}

#[tokio::test]
async fn create_tick_table_qualifies_configured_database() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/sql"))
        .and(header("authorization", BASIC_ROOT_TAOSDATA))
        .and(body_string_contains(
            "CREATE STABLE IF NOT EXISTS market_data.tick_data",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "rows": 0,
            "data": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client =
        TDengineClient::new_with_database(&server.uri(), "root:taosdata", "market_data").unwrap();

    client.create_tick_table().await.unwrap();
}

#[tokio::test]
async fn insert_ticks_qualifies_configured_database() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/sql"))
        .and(header("authorization", BASIC_ROOT_TAOSDATA))
        .and(body_string_contains(
            "INSERT INTO market_data.t_600000 USING market_data.tick_data",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "rows": 1,
            "data": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client =
        TDengineClient::new_with_database(&server.uri(), "root:taosdata", "market_data").unwrap();

    client
        .insert_ticks("600000", &[(1_749_164_400_000, 10.5, 100, 1050.0, 1)])
        .await
        .unwrap();
}
