use std::str;

use google_cloud_bigquery::client::{Client as bigQueryClient, ClientConfig as bigQueryClientConfig};
use google_cloud_bigquery::http::tabledata::insert_all::{InsertAllRequest, Row};
use google_cloud_storage::client::{Client as gcsClient, ClientConfig as gcsClientConfig};
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
// use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

// lazy_static! {
//     static ref INGESTION_DATE: Option<OffsetDateTime> = Some(OffsetDateTime::now_utc());
// }

#[derive(Debug, Serialize, Deserialize)]
struct TeamScorerRaw {
    scorerFirstName: String,
    scorerLastName: String,
    goals: i64,
    goalAssists: i64,
    games: i64,
}


#[derive(Debug, Serialize, Deserialize)]
struct TeamStatsRaw {
    teamName: String,
    teamScore: i64,
    scorers: Vec<TeamScorerRaw>,
}

#[derive(Serialize, Deserialize, Clone)]
struct TopScorerStatsRow {
    firstName: String,
    lastName: String,
    goals: i64,
    games: i64,
}

#[derive(Serialize, Deserialize, Clone)]
struct BestPasserStatsRow {
    firstName: String,
    lastName: String,
    goalAssists: i64,
    games: i64,
}

#[derive(Serialize, Deserialize, Clone)]
struct TeamStatsRow
{
    teamName: String,
    teamScore: i64,
    teamTotalGoals: i64,
    teamSlogan: String,
    topScorerStats: TopScorerStatsRow,
    bestPasserStats: BestPasserStatsRow,
    #[serde(with = "time::serde::rfc3339::option")]
    ingestionDate: Option<OffsetDateTime>,
}

fn deserialize_to_team_stats_raw_object(team_stats_raw_str: &str) -> TeamStatsRaw {
    let res_deserialization = serde_json::from_str(&team_stats_raw_str);

    match res_deserialization {
        Ok(team_stats_raw) => team_stats_raw,
        Err(e) => panic!("couldn't deserialize the team stats str to object: {}", e),
    }
}

fn to_bigquery_row(team_stats: TeamStatsRow) -> Row<TeamStatsRow> {
    Row {
        insert_id: None,
        json: team_stats,
    }
}

fn to_team_stats_domain(team_stats_raw: TeamStatsRaw) -> TeamStatsRow {
    let team_total_goals: i64 = team_stats_raw.scorers
        .iter()
        .map(|scorer| scorer.goals)
        .sum();

    let top_scorer_raw: &TeamScorerRaw = team_stats_raw.scorers
        .iter()
        .max_by_key(|scorer| scorer.goals)
        .expect("Top scorer not found !!");

    let best_passer_raw: &TeamScorerRaw = team_stats_raw.scorers
        .iter()
        .max_by_key(|scorer| scorer.goalAssists)
        .expect("Bet passer not found !!");

    let top_scorer = TopScorerStatsRow {
        firstName: top_scorer_raw.scorerFirstName.to_string(),
        lastName: top_scorer_raw.scorerLastName.to_string(),
        goals: top_scorer_raw.goals,
        games: top_scorer_raw.games,
    };

    let best_passer = BestPasserStatsRow {
        firstName: best_passer_raw.scorerFirstName.to_string(),
        lastName: best_passer_raw.scorerLastName.to_string(),
        goalAssists: best_passer_raw.goalAssists,
        games: best_passer_raw.games,
    };

    TeamStatsRow {
        teamName: team_stats_raw.teamName,
        teamScore: team_stats_raw.teamScore,
        teamTotalGoals: team_total_goals,
        teamSlogan: "Test".to_string(),
        topScorerStats: top_scorer,
        bestPasserStats: best_passer,
        ingestionDate: Some(OffsetDateTime::now_utc()),
    }
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

    let team_stats_domain_list = str::from_utf8(result_file_as_string.as_slice())
        .expect("")
        .split("\n")
        .filter(|team_stats_raw| !team_stats_raw.is_empty())
        .map(|team_stats_str| deserialize_to_team_stats_raw_object(team_stats_str))
        .map(|team_stats_raw| to_team_stats_domain(team_stats_raw))
        .collect::<Vec<TeamStatsRow>>();

    let (config, project_id) = bigQueryClientConfig::new_with_auth().await.unwrap();
    let client = bigQueryClient::new(config).await.unwrap();

    let team_stats_table_bq_rows = team_stats_domain_list
        .iter()
        .map(|team_stats| to_bigquery_row(TeamStatsRow::clone(team_stats)))
        .collect::<Vec<Row<TeamStatsRow>>>();

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
