
const NBA_SCOREBOARD_URL: &str = "https://cdn.nba.com/static/json/liveData/scoreboard/todaysScoreboard_00.json";

use std::fs;
use tabled::{Tabled, Table};
use anyhow::Result;


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
        for (i ,period_info) in game_periods.iter().enumerate() {
          quarter_data[i] = period_info.get("score").unwrap().as_i64().unwrap();
        }
        Ok(QuarterBreakdown{q1: quarter_data[0], q2: quarter_data[1], q3: quarter_data[2], q4: quarter_data[3], total: final_score})
    }
    
    pub fn fetch_scoreboard() -> Result<()> {
        let r = ureq::get(NBA_SCOREBOARD_URL).call()?;
        let json: serde_json::Value = r.into_json().unwrap();
        let games = json.get("scoreboard").unwrap().get("games").unwrap().as_array().unwrap();
        for g in games {
            let mut scoreboard_table: Vec<QuarterBreakdown> = Vec::new();
            let game_id = g.get("gameId").unwrap().as_str().unwrap();
            let home = g.get("homeTeam").unwrap();
            let away = g.get("awayTeam").unwrap();
            let formatted_home_team_name = extract_team_name(home)?; //format!("({}) {} {}", teamTriCode, teamCity, teamName);
            let formatted_away_team_name = extract_team_name(away)?;
            let home_quarter_data = extract_quarter_info(home)?;
            scoreboard_table.push(home_quarter_data);
            let away_quarter_data = extract_quarter_info(away)?;
            scoreboard_table.push(away_quarter_data);
            let table = Table::new(scoreboard_table).to_string();
            println!("game_id: {}", game_id);
            println!("Home: {}\nAway: {}", formatted_home_team_name, formatted_away_team_name);
            println!("{}", table);
        }
        
        Ok(())
    }