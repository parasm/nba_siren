
use crate::nba::params::*;
use polars::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::{collections::HashMap, io::{Read, Write}, time::Instant, env};
use anyhow::Result;
use std::process::Command;
use rusqlite::{Connection, NO_PARAMS, MappedRows, params, named_params, ToSql};

const NBA_BASE_URL: &str = "https://stats.nba.com/stats";

#[derive(Debug)]
struct PlayDbInfo {
    game_id: String,
    eventnum: i64,
}

#[derive(Serialize, Deserialize)]
struct PlayerInfo {
    player_id: String,
    full_name: String,
    team_code: String,
    years: String,
}

#[derive(Serialize, Deserialize)]
struct PlayerInfoFile {
    players: HashMap<String, PlayerInfo>,
}

pub struct PlayByPlayV2 {
    pub start_period: StartPeriod,
    pub end_period: EndPeriod,
    pub game_id: GameID,
    pub player_id: Option<i64>,
    pub keyword: Option<String>,
}

pub struct BoxScoreDefensive {
    pub game_id: GameID
}

pub struct VidForPlay {
    pub game_id: GameID,
    pub game_event_id: String,
}

pub struct CommonAllPlayers {
    pub league_id: LeagueID,
    pub season: Season
}


impl SaveToDB for PlayByPlayV2 {
}

impl SaveToDB for CommonAllPlayers {
}

impl SaveToDB for BoxScoreDefensive {
}

impl SaveToDataframe for PlayByPlayV2 {
}

impl SaveToDataframe for CommonAllPlayers {
}

impl SaveToDataframe for BoxScoreDefensive {
}


pub trait SaveToDB: NBAEndpoint {
    fn get_insert_row_statement(&self, table_name: &str, row: &Vec<Value>) -> String {
        let mut insert_stmt = format!("INSERT INTO {} VALUES (NULL,", table_name);
        let row_size = row.len();
        for (pos, col_val) in row.iter().enumerate() {
            if col_val.is_null() {
                insert_stmt.push_str("NULL");
            }else if col_val.is_f64() {
                let num_val = col_val.as_f64().unwrap().to_string();
                insert_stmt.push_str(&num_val);
            }else if col_val.is_i64() {
                let num_val = col_val.as_i64().unwrap().to_string();
                insert_stmt.push_str(&num_val);
            }else if col_val.is_string() {
                let col_val_str = col_val.as_str().unwrap_or("").replace("'", "");
                insert_stmt.push('\'');
                insert_stmt.push_str(&col_val_str);
                insert_stmt.push('\'');
            }
            if pos != row_size -1 {
                insert_stmt.push(',');
            } 
            
        }
        insert_stmt.push_str("); \n");
        insert_stmt
        
    }
    fn get_create_statement(&self, headers: &Vec<Value>, row: &Vec<Value>) -> String {
        let mut create_inserts = String::from("id integer primary key, ");
        for (pos, val) in row.iter().enumerate() {
            let col_name = headers[pos].as_str().unwrap().to_lowercase();
            if val.is_f64() {
                let col_row = format!(" {} FLOAT", col_name);
                create_inserts.push_str(&col_row);
            }else if val.is_i64() {
                let col_row = format!(" {} INTEGER", col_name);
                create_inserts.push_str(&col_row);
            }else {
                let col_row = format!(" {} TEXT", col_name);
                create_inserts.push_str(&col_row);
            }
            if pos != row.len() - 1 {
                create_inserts.push(',');
            }
        }
        create_inserts.push_str("); \n");
        create_inserts

    }
    fn create_table(&self, table_name: &str,  headers: &Vec<Value>, json_rows: &Vec<Value>) -> Result<()> {
        let conn = Connection::open("nba_siren.db")?;
        let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
        conn.execute(&cleanup, [])?;
        let mut create_sql = format!("CREATE TABLE IF NOT EXISTS {} ( ", table_name.to_lowercase());
        //let sql_statements = json_rows.iter().map(|r| self.insert_row(r, headers) ).collect();
        for (pos, row) in json_rows.iter().enumerate() {
            let row_array = row.as_array().unwrap();
            if pos == 0 {
                let create_stmt = self.get_create_statement(headers, row_array);
                create_sql.push_str(&create_stmt);
            }
            let insert_stmt = self.get_insert_row_statement(table_name, row_array);
            create_sql.push_str(&insert_stmt);
        }

        conn.execute_batch(
            &create_sql,
        )?;

        Ok(())
    }
    fn save_to_db_file(&self) -> Result<()> {
        let endpoint_json = self.send_request().unwrap();
        let load_start = Instant::now();
        let result_sets = endpoint_json["resultSets"].as_array().unwrap();
        for data_set in result_sets {
            let data_set_values = data_set["rowSet"].as_array().unwrap();
            let data_set_headers = data_set["headers"].as_array().unwrap();
            let data_set_name = data_set["name"].as_str().unwrap();
            self.create_table(data_set_name, data_set_headers, data_set_values)?;
        }
        let sql_load_duration = load_start.elapsed();
        println!("sql loading took {:?}", &sql_load_duration);
        Ok(())
    }
}

pub trait SaveToDataframe: NBAEndpoint {
    fn load_dataframes(&self) -> Result<HashMap<String, DataFrame>> {
        let endpoint_json = self.send_request().unwrap();
        let load_start = Instant::now();
        let result_sets = endpoint_json["resultSets"].as_array().unwrap();
        let mut stats_dataframes: HashMap<String, DataFrame> = HashMap::new();
        for data_set in result_sets {
            let data_set_values = data_set["rowSet"].as_array().unwrap();
            let data_set_headers = data_set["headers"].as_array().unwrap();
            let mut headers_to_values: HashMap<&str, Vec<&Value>> = HashMap::new();
            data_set_values.iter().for_each(|r| insert_row_values(&mut headers_to_values, &r, data_set_headers));
    
            let mut df_series: Vec<Series> = Vec::new();
            for (col_name, json_values) in headers_to_values {
                if json_values.is_empty() { continue; }
                if let Some(first_non_null_val) = json_values.iter().find(|&v| !v.is_null()) {
                    if first_non_null_val.is_i64() {
                        let typed_data = json_values.iter().map(|&v| v.as_i64().unwrap_or(0)).collect::<Vec<i64>>();
                        df_series.push(Series::new(col_name, typed_data));
                    } else if first_non_null_val.is_f64()  {
                        let typed_data = json_values.iter().map(|&v| v.as_f64().unwrap_or(0.0)).collect::<Vec<f64>>();
                        df_series.push(Series::new(col_name, typed_data));
                    } else {
                        let typed_data = json_values.iter().map(|&v| v.as_str().unwrap_or("").to_lowercase()).collect::<Vec<String>>();
                        df_series.push(Series::new(col_name, typed_data));
                    }
                }else {
                    let v: Vec<&str> = vec![];
                    df_series.push(Series::new(col_name, v));
                }
    
            }
            let data_set_name = data_set["name"].as_str().unwrap();
            stats_dataframes.insert(data_set_name.to_string(), DataFrame::new(df_series)?);
        }
        let df_load_duration = load_start.elapsed();
        println!("datafram loading took {:?}", &df_load_duration);
        Ok(stats_dataframes)
    }
}

pub trait NBAEndpoint {
    fn send_request(&self) -> Result<Value>;
}

impl NBAEndpoint for CommonAllPlayers {
    fn send_request(&self) -> Result<Value> {
        let endpoint_url = format!("{}/commonallplayers?{}&{}&IsOnlyCurrentSeason=0", NBA_BASE_URL, self.league_id, self.season);
        Ok(fetch_nba_json(endpoint_url))
    }
}

impl NBAEndpoint for BoxScoreDefensive {
    fn send_request(&self) -> Result<Value> {
        let endpoint_url = format!("{}/boxscoreplayertrackv2?{}", NBA_BASE_URL, self.game_id);
        Ok(fetch_nba_json(endpoint_url))
    }
}

impl NBAEndpoint for PlayByPlayV2 {
    fn send_request(&self) -> Result<Value> {
        let endpoint_url = format!("{}/playbyplayv2?{}&{}&{}", NBA_BASE_URL, self.game_id, self.end_period, self.start_period);
        Ok(fetch_nba_json(endpoint_url))
    }
}

impl NBAEndpoint for VidForPlay {
    fn send_request(&self) -> Result<Value> {
        let endpoint_url = format!("{}/videoeventsasset?{}&GameEventID={}", NBA_BASE_URL, self.game_id, self.game_event_id);
        Ok(fetch_nba_json(endpoint_url))
    }
}

impl VidForPlay {
    pub fn get_video_url(&self) -> Result<String> {
        let video_detail_json = self.send_request()?;
        let video_url_json = video_detail_json["resultSets"]["Meta"]["videoUrls"].as_array().unwrap();
        let url = video_url_json[0]["lurl"].as_str().unwrap();
        let u = url.to_string();
        save_video(url, "test_video")?;
        return Ok(u)
    }
}

impl PlayByPlayV2 {
    pub fn print_play_by_play(&self) -> Result<()> {
        let playbyplay_frames = self.load_dataframes().unwrap();
        let playbyplay_df = playbyplay_frames.get("PlayByPlay").unwrap();
        let player_id_col = playbyplay_df.column("PLAYER1_ID")?;
        if let Some(p) = self.player_id {
            let mask = player_id_col.eq(p);
            let filtered_df = playbyplay_df.filter(&mask).unwrap();
            print_df(&filtered_df);
        }else {
            print_df(playbyplay_df);
        }        
        Ok(())
    }
    pub fn save_video_db(&self) -> Result<()> {
        if self.player_id.is_none() || self.keyword.is_none() {
            return Ok(());
        }
        self.save_to_db_file()?;
        let conn = Connection::open("nba_siren.db")?;
        let load_start = Instant::now();
        let pid = self.player_id.as_ref().unwrap().to_string();
        let keyword = self.keyword.as_ref().unwrap();
        let game_str = self.game_id.to_string();
        let game_id_str = game_str.split("=").last().unwrap();
        let sql_keyword = format!("%{}%", keyword);
        let mut stmt = conn.prepare(
            "SELECT game_id, eventnum FROM PlayByPlay 
            WHERE game_id = ?1
            AND video_available_flag=1 
            AND player1_id = ?2
            AND (homedescription LIKE ?3 
            OR neutraldescription LIKE ?3 
            OR visitordescription LIKE ?3 );",
        )?;

        let player_info_rows  = stmt.query_map::<PlayDbInfo, _, _>(params![game_id_str, &pid, sql_keyword], |row|{
            Ok(PlayDbInfo{
                game_id: row.get(0)?,
                eventnum: row.get(1)?
            })
        })?;


        let video_list_file_name = "video_list.txt";
        let mut video_list_file = std::fs::File::create(video_list_file_name)?;

        let video_id = format!("{}_{}_{}", keyword, game_id_str, pid).replace(" ", "");
        let df_load_duration = load_start.elapsed();
        let video_start_time = Instant::now();
        for r in player_info_rows {
            let res = r.unwrap();
            let event_num = res.eventnum.to_string();
            let video_url = get_url_for_video(&game_str, &event_num)?;
            let video_file_name = format!("play_videos/{}_{}.mp4", &video_id, event_num);
            save_video(&video_url, &video_file_name)?;
            video_list_file.write(format!("file {}\n", &video_file_name).as_bytes())?;
        }

        let video_save_duration = video_start_time.elapsed();
        let output_file = format!("ALL_{}.mp4", video_id);
        let video_combine_start = Instant::now();
        combine_videos(video_list_file_name, &output_file);
        let video_combine_duration = video_combine_start.elapsed();
        println!("Loading game data from sql took {:?}", df_load_duration);
        println!("Saving videos took {:?}", video_save_duration);
        println!("Combining videos took {:?}", video_combine_duration);
        Ok(())

    }
    pub fn save_video(&self) -> Result<()> {
        if self.player_id.is_none() || self.keyword.is_none() {
            return Ok(());
        }
        let df_load_start = Instant::now();
        let playbyplay_frames = self.load_dataframes().unwrap();
        let playbyplay_df = playbyplay_frames.get("PlayByPlay").unwrap();
        let pid = self.player_id.as_ref().unwrap();
        let keyword = self.keyword.as_ref().unwrap();

        let filtered_df = filter_on_description(&playbyplay_df, &keyword, *pid)?;
        print_df(&filtered_df);
        
        let video_avail_flag = filtered_df.column("VIDEO_AVAILABLE_FLAG")?;
        let video_mask = video_avail_flag.eq(1 as i32);
        let video_df = filtered_df.filter(&video_mask)?;

        let play_event_nums = video_df.column("EVENTNUM")?;
        let video_list_file_name = "video_list.txt";
        let mut video_list_file = std::fs::File::create(video_list_file_name)?;
        let game_str = self.game_id.to_string();
        let game_id_str = game_str.split("=").last().unwrap();
        let video_id = format!("{}_{}_{}", keyword, game_id_str, pid).replace(" ", "");
        let df_load_duration = df_load_start.elapsed();
        let video_start_time = Instant::now();
        for event_num_option in play_event_nums.i64()? {
            if let Some(event_num) = event_num_option {
                let video_url = get_url_for_video(&self.game_id.to_string(), &event_num.to_string())?;
                let video_file_name = format!("play_videos/{}_{}.mp4", &video_id, event_num);
                save_video(&video_url, &video_file_name)?;
                video_list_file.write(format!("file {}\n", &video_file_name).as_bytes())?;
            }
            
        }
        let video_save_duration = video_start_time.elapsed();
        let output_file = format!("ALL_{}.mp4", video_id);
        let video_combine_start = Instant::now();
        combine_videos(video_list_file_name, &output_file);
        let video_combine_duration = video_combine_start.elapsed();
        println!("Loading game df took {:?}", df_load_duration);
        println!("Saving videos took {:?}", video_save_duration);
        println!("Combining videos took {:?}", video_combine_duration);
        Ok(())
    }
}

impl CommonAllPlayers {
    pub fn save_static_data(&self) -> Result<()> {
        let fetched_frames = self.load_dataframes()?;
        let player_info_df = fetched_frames.get("CommonAllPlayers").expect("Failed to load static info df");
        let df_size = player_info_df.height();
        let mut players: HashMap<String, PlayerInfo> = HashMap::new();
        for i in 0..df_size {
            //let row = player_info_df.get_row(i);
            if let Some(row) = player_info_df.get(i) {
                let player_id = row[0].to_string();
                let full_name = row[1].to_string();
                let team_code = row[5].to_string();
                let years = format!("{} - {}", row[2].to_string(), row[8].to_string());
                players.insert(player_id.clone(), 
                    PlayerInfo {
                        player_id,
                        full_name,
                        team_code,
                        years,
                    }
                );
                
            }
        }
        print_df(player_info_df);
        let players_info_file_data = PlayerInfoFile {
            players
        };
        let player_info_writer = std::fs::File::create("static_data/player_info.json").unwrap();
        serde_json::to_writer(player_info_writer, &players_info_file_data).expect("Failed to save playerinfo file");
        Ok(())
    }
}


fn insert_row_values<'a>(headers_to_values: &mut HashMap<&'a str, Vec<&'a Value>>, row: &'a Value, headers: &'a Vec<Value>) -> () {
    let row_array = row.as_array().unwrap();
    for (pos, col_val) in row_array.iter().enumerate() {
        let col_name = headers[pos].as_str().unwrap();
        if let Some(series_values) = headers_to_values.get_mut(col_name) {
            series_values.push(col_val);
        }else {
            headers_to_values.insert(col_name, Vec::new());
        }
        
    }
}

fn combine_videos(video_list_file_name: &str, output_file_name: &str) {
    let mut ffmpeg_cmd = "./ffmpeg";
    if env::consts::OS == "windows" {
        ffmpeg_cmd = "./ffmpeg.exe";
    }
    let video_processor = Command::new(ffmpeg_cmd)
        .args(["-f", "concat", "-i", video_list_file_name, "-c", "copy", output_file_name, "-y"]).output().unwrap();
    println!("Saving {}, status: {}", video_processor.status, output_file_name);
    if !video_processor.status.success() {
        let e = String::from_utf8(video_processor.stderr).unwrap();
        println!("{}", e);
    }
}

fn print_df(df: &DataFrame) -> () {
    let col_names = df.get_column_names();
    let df_size = df.height();
    for i in 0..df_size {
        if let Some(row) = df.get(i) {
            row.iter().enumerate().for_each(|(i, v)|  print!("{} : {}, ", col_names[i], v.to_string()))
        }
        print!("\n");
    }
}

fn save_video(video_url: &str, file_name: &str) -> Result<()> {
    let req = ureq::get(video_url).call().unwrap();
    let len: usize = req.header("Content-Length")
    .unwrap()
    .parse().unwrap();
    let mut bytes: Vec<u8> = Vec::with_capacity(len);
    req.into_reader().read_to_end(&mut bytes).unwrap();
    let mut f = std::fs::File::create(file_name).unwrap();
    f.write_all(&bytes).unwrap();
    Ok(())  
}


fn fetch_nba_json(endpoint_url:String) -> Value {
    let r = ureq::get(&endpoint_url)
    .set("Host","stats.nba.com")
    .set("User-Agent","Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:72.0) Gecko/20100101 Firefox/72.0")
    .set("Accept","application/json, text/plain, */*")
    .set("Accept-Language","en-US,en;q=0.5")
    .set("Accept-Encoding","gzip, deflate, br")
    // .set("x-nba-stats-origin","stats")
    // .set("x-nba-stats-token","true")
    .set("Connection","keep-alive")
    .set("Referer","https://stats.nba.com/")
    .set("Pragma","no-cache")
    .set("Cache-Control","no-cache")
    .call().expect("Failed to fetch data from nba server");
    r.into_json().expect("Failed to fetch data from nba server")
}

fn get_url_for_video(game_id: &str, game_event_id: &str) -> Result<String> {
    let endpoint_url = format!("{}/videoeventsasset?{}&GameEventID={}", NBA_BASE_URL, game_id, game_event_id);
    let video_detail_json = fetch_nba_json(endpoint_url);
    let video_url_json = video_detail_json["resultSets"]["Meta"]["videoUrls"].as_array().unwrap();
    let url = video_url_json[0]["lurl"].as_str().unwrap();
    Ok(url.to_string())
}

fn filter_on_description(playbyplay_df: &DataFrame, keyword: &str, player_id: i64) -> Result<DataFrame> {
    let player_id_col = playbyplay_df.column("PLAYER1_ID")?;
    let home_desc_col = playbyplay_df.column("HOMEDESCRIPTION")?;
    let away_desc_col = playbyplay_df.column("VISITORDESCRIPTION")?;
    let neut_desc_col = playbyplay_df.column("NEUTRALDESCRIPTION")?;
    let mask = home_desc_col.utf8()?.contains(&keyword)? | away_desc_col.utf8()?.contains(&keyword)? | neut_desc_col.utf8()?.contains(&keyword)?;
    let player_mask = mask & player_id_col.eq(player_id);
    Ok(playbyplay_df.filter(&player_mask)?)
}