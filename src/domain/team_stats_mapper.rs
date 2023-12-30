use std::str;

use time::OffsetDateTime;

use crate::domain;
use crate::domain::team_stats_structs::TeamStatsRaw;
use crate::domain::team_stats_structs::TeamStats;

pub struct TeamStatsMapper;

impl TeamStatsMapper {
    pub fn map_to_team_stats_domains(ingestion_date: Option<OffsetDateTime>,
                                     result_file_as_string: Vec<u8>) -> Vec<TeamStats> {
        str::from_utf8(result_file_as_string.as_slice())
            .expect("")
            .split("\n")
            .filter(|team_stats_raw| !team_stats_raw.is_empty())
            .map(|team_stats_str| TeamStatsMapper::deserialize_to_team_stats_raw_object(team_stats_str))
            .map(|team_stats_raw| TeamStatsMapper::map_to_team_stats_domain(ingestion_date, team_stats_raw))
            .collect::<Vec<TeamStats>>()
    }

    fn deserialize_to_team_stats_raw_object(team_stats_raw_str: &str) -> TeamStatsRaw {
        let res_deserialization = serde_json::from_str(&team_stats_raw_str);

        match res_deserialization {
            Ok(team_stats_raw) => team_stats_raw,
            Err(e) => panic!("couldn't deserialize the team stats str to object: {}", e),
        }
    }

    fn map_to_team_stats_domain(ingestion_date: Option<OffsetDateTime>,
                                team_stats_raw: TeamStatsRaw) -> TeamStats {
        let team_total_goals: i64 = team_stats_raw.scorers
            .iter()
            .map(|scorer| scorer.goals)
            .sum();

        let top_scorer_raw: &domain::team_stats_structs::TeamScorerRaw = team_stats_raw.scorers
            .iter()
            .max_by_key(|scorer| scorer.goals)
            .expect("Top scorer not found !!");

        let best_passer_raw: &domain::team_stats_structs::TeamScorerRaw = team_stats_raw.scorers
            .iter()
            .max_by_key(|scorer| scorer.goalAssists)
            .expect("Bet passer not found !!");

        let top_scorer = domain::team_stats_structs::TopScorerStats {
            firstName: top_scorer_raw.scorerFirstName.to_string(),
            lastName: top_scorer_raw.scorerLastName.to_string(),
            goals: top_scorer_raw.goals,
            games: top_scorer_raw.games,
        };

        let best_passer = domain::team_stats_structs::BestPasserStats {
            firstName: best_passer_raw.scorerFirstName.to_string(),
            lastName: best_passer_raw.scorerLastName.to_string(),
            goalAssists: best_passer_raw.goalAssists,
            games: best_passer_raw.games,
        };

        TeamStats {
            teamName: team_stats_raw.teamName,
            teamScore: team_stats_raw.teamScore,
            teamTotalGoals: team_total_goals,
            teamSlogan: "Test".to_string(),
            topScorerStats: top_scorer,
            bestPasserStats: best_passer,
            ingestionDate: ingestion_date,
        }
    }
}
