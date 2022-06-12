
use crate::nba::endpoints::NBAEndpoint;

use polars::prelude::*;
use serde_json::Value;
use std::{collections::HashMap, time::Instant};
use anyhow::Result;

use rusqlite::{Connection, params};

pub trait SaveToDB: NBAEndpoint {
    fn get_db_connection(&self) -> &Connection;
    fn check_table_exists(&self, table_name: &str) -> Result<bool> {
        let db_conn = self.get_db_connection();
        let mut find_table_stmt = db_conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=? ")?;
        let found_table = find_table_stmt.query_row::<String,_,  _>(params![table_name], |r| {
            let t = r.get(0);
            t
        });
        Ok(found_table.is_ok())
    }
    fn search_table(&self, table_name: &str, select_column_names: Vec<&str>, search_column_names: Vec<&str>, keyword: &str ) -> Result<Vec<HashMap<String, String>>> {
        let db_conn = self.get_db_connection();
        let where_stmt = search_column_names.iter().map(|&col|{
            format!("{col} like '%{keyword}%'")
        }).collect::<Vec<String>>().join(" OR ");
        let select_stmt = select_column_names.join(", ");
        let search_stmt = format!("SELECT {} FROM {table_name} WHERE {}", &select_stmt, &where_stmt);
        let mut sql_search_stmt = db_conn.prepare(&search_stmt)?;
        let mut results: Vec<HashMap<String, String>> = Vec::new();
        println!("{search_stmt}");
        let results_iter = sql_search_stmt.query_map::<_, _, _>(params![], |row|{
            let mut r:HashMap<String, String> = HashMap::new();
            select_column_names.iter().enumerate().for_each(|(i, &col)|{
                let row_val;
                if col.contains("_id") {
                    let num: i32 = row.get(i).unwrap_or(0);
                    row_val = num.to_string();
                }else {
                    row_val = row.get(i).unwrap_or(Default::default());
                }
                r.insert(col.to_string(), row_val);
            });
            results.push(r);
            Ok(())
        })?;
        results_iter.for_each(drop);

        Ok(results)

    }
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
        let db_conn = self.get_db_connection();
        if self.check_table_exists(table_name)? {
            return Ok(());
        }
        // let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
        // db_conn.execute(&cleanup, [])?;
        let mut create_sql = format!("CREATE TABLE IF NOT EXISTS {} ( ", table_name);
        for (pos, row) in json_rows.iter().enumerate() {
            let row_array = row.as_array().unwrap();
            if pos == 0 {
                let create_stmt = self.get_create_statement(headers, row_array);
                create_sql.push_str(&create_stmt);
            }
            let insert_stmt = self.get_insert_row_statement(table_name, row_array);
            create_sql.push_str(&insert_stmt);
        }

        db_conn.execute_batch(
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
            let data_set_name = data_set["name"].as_str().unwrap().to_lowercase();
            self.create_table(&data_set_name, data_set_headers, data_set_values)?;
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

// Deprecated dataframe video loading
// pub fn save_video(&self) -> Result<()> {
//     if self.player_id.is_none() || self.keyword.is_none() {
//         return Ok(());
//     }
//     let df_load_start = Instant::now();
//     let playbyplay_frames = self.load_dataframes().unwrap();
//     let playbyplay_df = playbyplay_frames.get("PlayByPlay").unwrap();
//     let pid = self.player_id.as_ref().unwrap();
//     let keyword = self.keyword.as_ref().unwrap();

//     let filtered_df = filter_on_description(&playbyplay_df, &keyword, *pid)?;
//     print_df(&filtered_df);
    
//     let video_avail_flag = filtered_df.column("VIDEO_AVAILABLE_FLAG")?;
//     let video_mask = video_avail_flag.eq(1 as i32);
//     let video_df = filtered_df.filter(&video_mask)?;

//     let play_event_nums = video_df.column("EVENTNUM")?;
//     let video_list_file_name = "video_list.txt";
//     let mut video_list_file = std::fs::File::create(video_list_file_name)?;
//     let game_str = self.game_id.to_string();
//     let game_id_str = game_str.split("=").last().unwrap();
//     let video_id = format!("{}_{}_{}", keyword, game_id_str, pid).replace(" ", "");
//     let df_load_duration = df_load_start.elapsed();
//     let video_start_time = Instant::now();
//     for event_num_option in play_event_nums.i64()? {
//         if let Some(event_num) = event_num_option {
//             let video_url = get_url_for_video(&self.game_id.to_string(), &event_num.to_string())?;
//             let video_file_name = format!("play_videos/{}_{}.mp4", &video_id, event_num);
//             save_video(&video_url, &video_file_name)?;
//             video_list_file.write(format!("file {}\n", &video_file_name).as_bytes())?;
//         }
        
//     }
//     let video_save_duration = video_start_time.elapsed();
//     let output_file = format!("ALL_{}.mp4", video_id);
//     let video_combine_start = Instant::now();
//     combine_videos(video_list_file_name, &output_file);
//     let video_combine_duration = video_combine_start.elapsed();
//     println!("Loading game df took {:?}", df_load_duration);
//     println!("Saving videos took {:?}", video_save_duration);
//     println!("Combining videos took {:?}", video_combine_duration);
//     Ok(())
// }