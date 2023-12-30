#[macro_use]
extern crate lazy_static;

use google_cloud_bigquery::client::{Client as bigQueryClient, ClientConfig as bigQueryClientConfig};
use google_cloud_bigquery::http::tabledata::insert_all::InsertAllRequest;
use google_cloud_storage::client::{Client as gcsClient, ClientConfig as gcsClientConfig};
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use time::OffsetDateTime;

use domain::team_stats_mapper::TeamStatsMapper;
use team_stats_bq_row_mapper::TeamStatsBQRowMapper;

mod team_stats_bq_row_mapper;

mod domain;

lazy_static! {
    static ref INGESTION_DATE: Option<OffsetDateTime> = Some(OffsetDateTime::now_utc());
}

#[tokio::main]
async fn main() {
    println!("Reading team stats raw data from Cloud Storage...");

    let gcs_client_config = gcsClientConfig::default().with_auth().await.unwrap();
    let gcs_client = gcsClient::new(gcs_client_config);

    // Download the file
    let input_file_as_bytes_res = gcs_client.download_object(&GetObjectRequest {
        bucket: "mazlum_dev".to_string(),
        object: "airflow/team_league/elt/input_teams_stats_raw.json".to_string(),
        ..Default::default()
    }, &Range::default()).await;

    let result_file_as_string = match input_file_as_bytes_res {
        Ok(v) => v,
        Err(e) => panic!("Error when reading the input file: {}", e),
    };

    let team_stats_domain_list = TeamStatsMapper::map_to_team_stats_domains(
        *INGESTION_DATE,
        result_file_as_string,
    );

    let (config, project_id) = bigQueryClientConfig::new_with_auth().await.unwrap();
    let client = bigQueryClient::new(config).await.unwrap();

    let team_stats_table_bq_rows = TeamStatsBQRowMapper::map_to_team_stats_bigquery_rows(team_stats_domain_list);

    let request = InsertAllRequest {
        rows: team_stats_table_bq_rows,
        ..Default::default()
    };
    let result = client.tabledata().insert(
        project_id.unwrap().as_str(),
        "mazlum_test",
        "team_stat",
        &request,
    ).await.unwrap();

    let bigquery_insert_errors = result.insert_errors;
    match bigquery_insert_errors {
        None => println!("The Team Stats domain data was correctly loaded to BigQuery"),
        Some(e) => panic!("Error when trying to load the Team Stats domain data to BigQuery : {:#?}", e),
    }
}
