mod nba;

use nba::endpoints::{NBAEndpoint, SaveToDB};
use clap::{Parser, Subcommand};

//TODOs
// Ability to lookup consts like team id, player id,
// sync constants 
// Call any endpoint, cmd to build parameters for you

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct NBACli {
    #[clap(subcommand)]
    cmd: Commands
}

#[derive(Debug, Subcommand)]
enum Commands {
    Lookup {
        #[clap(short, long, required = false)]
        team_name: Option<String>,

        #[clap(short, long, required = false)]
        player_name: Option<String>,

        #[clap(short, long)]
        endpoint: Option<String>,
    },
    Scoreboard,
    Boxscore {
        game_id: String,

        #[clap(short, long)]
        defensive: bool,
    },
    Playbyplay {
        game_id: String,

        #[clap(short, long, required = false)]
        player_id: Option<String>,

        #[clap(short, long, required = false)]
        keyword: Option<String>,

        #[clap(short, long)]
        save_videos: bool
    },
    Vidforplay {
        game_id: String,

        game_event_id: String,
    },
    Savestaticdata,
    Test,
}

fn fetch_endpoint(endpoint: &str) {
    let endpoint_metadata = nba::live_data::get_endpoint_metadata().unwrap();
    let metadata = endpoint_metadata.get(endpoint);
    if metadata.is_none() {
        println!("No info found for {}", endpoint);
        return;
    }
    let endpoint_params = metadata.unwrap().get("params").unwrap();
    for (param_name, val) in endpoint_params.as_object().unwrap() {
        println!("{}: {}", param_name, val.as_str().unwrap());
    }
}


fn main() {
    let args = NBACli::parse();
    match args.cmd {
        Commands::Lookup { endpoint, player_name, team_name } => {
            if let Some(e) = endpoint {
                fetch_endpoint(&e);
            }else if let Some(name) = player_name {
                let all_p_frames = nba::endpoints::CommonAllPlayers{
                    league_id: Default::default(),
                    season: Default::default(),
                }.load_dataframes().unwrap();
                let player_info_df = all_p_frames.get("CommonAllPlayers").unwrap();
                let name_col = player_info_df.column("DISPLAY_FIRST_LAST").expect("df should have a name column");
                let name_mask = name_col.utf8().unwrap().contains(&name).unwrap();
                let name_df = player_info_df.filter(&name_mask).expect("Failed filtering player data");
                println!("{}", name_df);

            }else if let Some(name) = team_name {
                let all_p_frames = nba::endpoints::CommonAllPlayers{
                    league_id: Default::default(),
                    season: Default::default(),
                }.load_dataframes().unwrap();
                let player_info_df = all_p_frames.get("CommonAllPlayers").unwrap();
                let name_col = player_info_df.column("TEAM_NAME").expect("df should have a name column");
                let abbr_col = player_info_df.column("TEAM_ABBREVIATION").expect("df should have a name column");
                let name_mask = name_col.utf8().unwrap().contains(&name).unwrap() | abbr_col.utf8().unwrap().contains(&name).unwrap();
                let team_df = player_info_df.filter(&name_mask).expect("Failed filtering player data");
                println!("{}", team_df);
            }
            
        }
        Commands::Savestaticdata => {
            let all_p_frames = nba::endpoints::CommonAllPlayers{
                league_id: Default::default(),
                season: Default::default(),
            }.save_static_data().unwrap();
        }
        Commands::Scoreboard => {
            nba::live_data::fetch_scoreboard().unwrap();
        }
        Commands::Test => {

            let p = nba::endpoints::PlayByPlayV2{
                game_id: nba::params::GameID::ID("0042100401".to_string()),
                player_id: Some(1628369),
                keyword: Some("reb".to_string()),
                start_period: Default::default(),
                end_period: Default::default()
            };
            p.save_video_db().unwrap();
            //p.save_to_db_file().unwrap();


        }
        Commands::Playbyplay {game_id, player_id, keyword, save_videos} => {
            let pid = if let Some(id_str) = player_id {
                Some(id_str.parse::<i64>().unwrap())
            }else {
                None
            };
            let p = nba::endpoints::PlayByPlayV2{
                game_id: nba::params::GameID::ID(game_id),
                player_id: pid,
                keyword: keyword,
                start_period: Default::default(),
                end_period: Default::default()
            };
            if save_videos {
                //p.save_video().unwrap();
                p.save_video_db().unwrap();
            }else{
                p.print_play_by_play().unwrap();
            }
        }
        Commands::Vidforplay {game_id, game_event_id } => {
            let p = nba::endpoints::VidForPlay{
                game_id: nba::params::GameID::ID(game_id),
                game_event_id: game_event_id,
            };
            let u = p.get_video_url().unwrap();
            webbrowser::open(&u).unwrap();
            println!("{}", u);
        }
        Commands::Boxscore {game_id, defensive} => {
            if defensive {
                let boxscore = nba::endpoints::BoxScoreDefensive {
                    game_id: nba::params::GameID::ID(game_id)
                };
                let boxscore_frames = boxscore.load_dataframes().unwrap();
                for (data_set_name, dataframe) in boxscore_frames {
                    println!("{}\n{}", data_set_name, dataframe);
                }
            }else {
                nba::live_data::fetch_scoreboard().unwrap();
            }
            
        }
    }
}
