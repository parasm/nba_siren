
use crate::nba::params::*;
use crate::nba::db::{SaveToDB, SaveToDataframe};

use std::thread;
use polars::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::{collections::HashMap, io::{Read, Write}, time::Instant, env};
use anyhow::Result;
use std::process::Command;
use rusqlite::{Connection, params};

const NBA_BASE_URL: &str = "https://stats.nba.com/stats";

struct PlayDbInfo {
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
    db_connection: Connection,
}

pub struct BoxScoreDefensive {
    pub game_id: GameID,
    db_connection: Connection
}

pub struct VidForPlay {
    pub game_id: GameID,
    pub game_event_id: String,
    db_connection: Connection,
}

pub struct CommonAllPlayers {
    pub league_id: LeagueID,
    pub season: Season,
    db_connection: Connection,
}


impl SaveToDB for PlayByPlayV2 {
    fn get_db_connection(&self) -> &Connection {
        &self.db_connection
    }
    fn save_to_db_file(&self) -> Result<()> {
        let endpoint_json = self.send_request().unwrap();
        let load_start = Instant::now();
        let result_sets = endpoint_json["resultSets"].as_array().unwrap();
        for data_set in result_sets {
            let data_set_values = data_set["rowSet"].as_array().unwrap();
            let data_set_headers = data_set["headers"].as_array().unwrap();
            let data_set_name = data_set["name"].as_str().unwrap();
            if data_set_name == "PlayByPlay" {
                let game_str = match &self.game_id {
                    GameID::ID(game) => game,
                };
                let table_name = format!("{data_set_name}_{game_str}");
                self.create_table(&table_name, data_set_headers, data_set_values)?;
            }
        }
        let sql_load_duration = load_start.elapsed();
        println!("sql loading took {:?}", &sql_load_duration);
        Ok(())
    }
}

impl SaveToDB for CommonAllPlayers {
    fn get_db_connection(&self) -> &Connection {
        &self.db_connection
    }
}

impl SaveToDB for BoxScoreDefensive {
    fn get_db_connection(&self) -> &Connection {
        &self.db_connection
    }
}

impl SaveToDB for VidForPlay {
    fn get_db_connection(&self) -> &Connection {
        &self.db_connection
    }
}

impl SaveToDataframe for PlayByPlayV2 {
}

impl SaveToDataframe for CommonAllPlayers {
}

impl SaveToDataframe for BoxScoreDefensive {
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

impl BoxScoreDefensive {
    pub fn new(game_id: GameID) -> BoxScoreDefensive {
        let db_connection = Connection::open("nba_siren.db").unwrap();
        BoxScoreDefensive {
            game_id,
            db_connection
        }
    }
}

impl VidForPlay {
    pub fn new(game_id: GameID, game_event_id: String) -> VidForPlay {
        let db_connection = Connection::open("nba_siren.db").unwrap();
        VidForPlay {
            game_id,
            game_event_id,
            db_connection
        }
    }
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
    pub fn new(start_period: StartPeriod, end_period: EndPeriod,
        game_id: GameID,
        player_id: Option<i64>,
        keyword: Option<String>,) -> PlayByPlayV2 {
        let db_connection = Connection::open("playbyplay.db").unwrap();
        PlayByPlayV2 {
            start_period,
            end_period,
            game_id,
            player_id,
            keyword,
            db_connection
        }
    }
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
        let conn = self.get_db_connection();
        let load_start = Instant::now();
        let pid = self.player_id.as_ref().unwrap().to_string();
        let keyword = self.keyword.as_ref().unwrap();
        let game_str = self.game_id.to_string();
        let game_id_str = match &self.game_id {
            GameID::ID(game) => game,
        };
        let sql_keyword = format!("%{}%", keyword);
        let table_name = format!("playbyplay_{game_id_str}");
        let mut stmt = conn.prepare(
            "SELECT game_id, eventnum FROM ?1 
            WHERE game_id = ?2
            AND video_available_flag=1 
            AND player1_id = ?3
            AND (homedescription LIKE ?4
            OR neutraldescription LIKE ?4 
            OR visitordescription LIKE ?4 );",
        )?;

        let player_info_rows  = stmt.query_map::<PlayDbInfo, _, _>(params![table_name, game_id_str, &pid, sql_keyword], |row|{
            Ok(PlayDbInfo{
                eventnum: row.get(1)?
            })
        })?;


        let video_list_file_name = "video_list.txt";
        let mut video_list_file = std::fs::File::create(video_list_file_name)?;

        let video_id = format!("{}_{}_{}", keyword, game_id_str, pid).replace(" ", "");
        let df_load_duration = load_start.elapsed();
        let video_start_time = Instant::now();
        let mut save_vid_handles = Vec::new();
        for r in player_info_rows {
            let res = r.unwrap();
            let event_num = res.eventnum.to_string();
            let video_url = get_url_for_video(&game_str, &event_num)?;
            let video_file_name = format!("play_videos/{}_{}.mp4", &video_id, event_num);
            video_list_file.write(format!("file {}\n", &video_file_name).as_bytes())?;
            let save_video_thread = thread::spawn(move || {
                save_video(&video_url, &video_file_name)
            });
            save_vid_handles.push(save_video_thread);
        }

        for handle in save_vid_handles {
            handle.join().unwrap()?;
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
}

impl CommonAllPlayers {
    pub fn new(league_id: LeagueID, season: Season) -> CommonAllPlayers {
        let db_connection = Connection::open("all_players.db").unwrap();
        CommonAllPlayers {
            league_id,
            season,
            db_connection
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
