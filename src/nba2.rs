pub mod endpoints;
pub mod params;

pub use params as nba_params;
pub use endpoints as nba_endpoints;

pub mod nba {
// For some reason for a while this fixed my ? usage return errors
// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
use std::fs;
use serde::ser::Error;
use tabled::{Tabled, Table};
use polars::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use anyhow::Result;

const NBA_SCOREBOARD_URL: &str = "https://cdn.nba.com/static/json/liveData/scoreboard/todaysScoreboard_00.json";
const NBA_STATS_BASE_URL: &str = "https://stats.nba.com/stats/";

#[derive(Tabled)]
struct QuarterBreakdown {
    q1: i64,
    q2: i64,
    q3: i64,
    q4: i64,
    total: i64,
}

pub fn get_endpoint_metadata() -> Result<serde_json::Value> {
    let data = fs::read_to_string("endpoints_v2.json")?;
    let json: serde_json::Value =
        serde_json::from_str(&data)?;
    Ok(json)
}

fn extract_team_name(team_data: &serde_json::Value) -> Result<String> {
    let team_name = team_data.get("teamName").unwrap().as_str().unwrap();
    let team_city = team_data.get("teamCity").unwrap().as_str().unwrap();
    let team_tri_code = team_data.get("teamTricode").unwrap().as_str().unwrap();
    Ok(format!("({}) {} {}", team_tri_code, team_city, team_name))
}

fn extract_quarter_info(team_data: &serde_json::Value) -> Result<QuarterBreakdown> {
    let final_score = team_data.get("score").unwrap().as_i64().unwrap();
    let game_periods = team_data.get("periods").unwrap().as_array().unwrap();
    let mut quarter_data: Vec<i64> = vec![0; 4];
    for p in game_periods {
      let q = p.get("period").unwrap().as_i64().unwrap()-1;
      quarter_data[q as usize] = p.get("score").unwrap().as_i64().unwrap();
    }
    Ok(QuarterBreakdown{q1: quarter_data[0], q2: quarter_data[0], q3: quarter_data[0], q4: quarter_data[0], total: final_score})
}

pub async fn fetch_scoreboard() -> Result<()> {
    let r = ureq::get(NBA_SCOREBOARD_URL).call()?;
    let json: serde_json::Value = r.into_json().unwrap();
    let games = json.get("scoreboard").unwrap().get("games").unwrap().as_array().unwrap();
    for g in games {
        let mut scoreboard_table: Vec<QuarterBreakdown> = Vec::new();
        let game_id = g.get("gameId").unwrap().as_str().unwrap();
        println!("game_id: {}", game_id);
        let home = g.get("homeTeam").unwrap();
        let away = g.get("awayTeam").unwrap();
        let formatted_home_team_name = extract_team_name(home)?; //format!("({}) {} {}", teamTriCode, teamCity, teamName);
        let formatted_away_team_name = extract_team_name(away)?;
        println!("Home: {}\nAway: {}", formatted_home_team_name, formatted_away_team_name);
        let home_periods = home.get("periods").unwrap().as_array().unwrap();
        let mut quarter_data: Vec<i64> = vec![0; 4];
        for p in home_periods {
          let q = p.get("period").unwrap().as_i64().unwrap()-1;
          quarter_data[q as usize] = p.get("score").unwrap().as_i64().unwrap();
        }
        let home_quarter_data = extract_quarter_info(home)?;
        scoreboard_table.push(home_quarter_data);
        let away_quarter_data = extract_quarter_info(away)?;
        scoreboard_table.push(away_quarter_data);
        let table = Table::new(scoreboard_table).to_string();
        println!("{}", table);
    }
    
    Ok(())
}



}
