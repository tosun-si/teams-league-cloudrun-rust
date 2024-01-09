use std::collections::HashMap;
use std::str;

use time::OffsetDateTime;

use crate::domain;
use crate::domain::team_stats_structs::TeamScorerRaw;
use crate::domain::team_stats_structs::TeamStats;
use crate::domain::team_stats_structs::TeamStatsRaw;

pub struct TeamStatsMapper;

impl TeamStatsMapper {
    pub fn map_to_team_stats_domains(ingestion_date: Option<OffsetDateTime>,
                                     team_slogans: HashMap<&str, &str>,
                                     result_file_as_bytes: Vec<u8>) -> Vec<TeamStats> {
        str::from_utf8(result_file_as_bytes.as_slice())
            .expect("")
            .split('\n')
            .filter(|team_stats_raw| !team_stats_raw.is_empty())
            .map(TeamStatsMapper::deserialize_to_team_stats_raw_object)
            .map(|team_stats_raw| TeamStatsMapper::map_to_team_stats_domain(ingestion_date, team_slogans.clone(), team_stats_raw))
            .collect::<Vec<TeamStats>>()
    }

    fn deserialize_to_team_stats_raw_object(team_stats_raw_str: &str) -> TeamStatsRaw {
        let res_deserialization = serde_json::from_str(team_stats_raw_str);

        match res_deserialization {
            Ok(team_stats_raw) => team_stats_raw,
            Err(e) => panic!("couldn't deserialize the team stats str to object: {}", e),
        }
    }

    fn map_to_team_stats_domain(ingestion_date: Option<OffsetDateTime>,
                                team_slogans: HashMap<&str, &str>,
                                team_stats_raw: TeamStatsRaw) -> TeamStats {
        let team_name = team_stats_raw.team_name;

        let team_total_goals: i64 = team_stats_raw.scorers
            .iter()
            .map(|scorer| scorer.goals)
            .sum();

        let top_scorer_raw: &TeamScorerRaw = team_stats_raw.scorers
            .iter()
            .max_by_key(|scorer| scorer.goals)
            .unwrap_or_else(|| panic!("Top scorer not found for the team !! {team_name}"));

        let best_passer_raw: &TeamScorerRaw = team_stats_raw.scorers
            .iter()
            .max_by_key(|scorer| scorer.goal_assists)
            .unwrap_or_else(|| panic!("Best passer not found for the team !! {team_name}"));

        let top_scorer = domain::team_stats_structs::TopScorerStats {
            first_name: top_scorer_raw.scorer_first_name.to_string(),
            last_name: top_scorer_raw.scorer_last_name.to_string(),
            goals: top_scorer_raw.goals,
            games: top_scorer_raw.games,
        };

        let best_passer = domain::team_stats_structs::BestPasserStats {
            first_name: best_passer_raw.scorer_first_name.to_string(),
            last_name: best_passer_raw.scorer_last_name.to_string(),
            goal_assists: best_passer_raw.goal_assists,
            games: best_passer_raw.games,
        };

        let team_slogan = team_slogans.get(team_name.as_str())
            .unwrap_or_else(|| panic!("Slogan not found for the team {team_name}"));

        TeamStats {
            team_name: team_name,
            team_score: team_stats_raw.team_score,
            team_total_goals: team_total_goals,
            team_slogan: team_slogan.to_string(),
            top_scorer_stats: top_scorer,
            best_passer_stats: best_passer,
            ingestion_date: ingestion_date,
        }
    }
}
