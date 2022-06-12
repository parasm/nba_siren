mod nba;

use nba::db::{SaveToDB, SaveToDataframe};
use nba::endpoints::{BoxScoreDefensive, PlayByPlayV2, CommonAllPlayers};
use clap::{Parser, Subcommand};

use crate::nba::endpoints::VidForPlay;

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
            match (endpoint, player_name, team_name) {
                (Some(e), _, _) => {
                    fetch_endpoint(&e);
                },
                (_, Some(p), _) => {
                    let all_p_frames = CommonAllPlayers::new(
                        Default::default(),
                        Default::default(),
                    );
                    all_p_frames.save_to_db_file().unwrap();
                    let res = all_p_frames.search_table("commonallplayers", vec!["display_first_last", "person_id"], vec!["display_first_last"], &p).unwrap();
                    println!("{:?}", res);
                },
                (_, _, Some(t)) => {
                    let all_p_frames = CommonAllPlayers::new(
                        Default::default(),
                        Default::default(),
                    );
                    all_p_frames.save_to_db_file().unwrap();
                    let res = all_p_frames.search_table("commonallplayers", vec!["team_id", "team_name"], vec!["team_city","team_name","team_abbreviation","team_code"], &t).unwrap();
                    if res.len() > 0 {
                        println!("{:?}", res.get(0).unwrap());
                    }
                    
                },
                (_, _, _) => println!("unsupported args")

            }
            
        }
        Commands::Savestaticdata => {
            let player_info = CommonAllPlayers::new(
                    Default::default(),
                    Default::default(),
            );
            player_info.save_to_db_file().unwrap();
            player_info.load_dataframes().unwrap();
        }
        Commands::Scoreboard => {
            nba::live_data::fetch_scoreboard().unwrap();
        }
        Commands::Test => {

            let p = PlayByPlayV2::new(
                Default::default(),
                Default::default(),
                nba::params::GameID::ID("0042100401".to_string()),
                Some(1628369),
                Some("reb".to_string()),
            );
            p.save_to_db_file().unwrap();
            let b = p.check_table_exists("fake").unwrap();
            println!("{b}");
            let res = p.search_table("playbyplay_0042100401", vec!["game_id", "player1_name"], vec!["homedescription", "neutraldescription", "visitordescription"], "foul").unwrap();
            println!("{:?}", res);

        }
        Commands::Playbyplay {game_id, player_id, keyword, save_videos} => {
            let pid = player_id.map(|id_str| {
                id_str.parse::<i64>().unwrap()
            });
            let p = PlayByPlayV2::new(
                Default::default(),
                Default::default(),
                nba::params::GameID::ID(game_id),
                pid,
                keyword,

            );
            if save_videos {
                p.save_video_db().unwrap();
            }else{
                p.print_play_by_play().unwrap();
            }
        }
        Commands::Vidforplay {game_id, game_event_id } => {
            let p = VidForPlay::new(
                nba::params::GameID::ID(game_id),
                game_event_id,
            );
            let u = p.get_video_url().unwrap();
            webbrowser::open(&u).unwrap();
            println!("{}", u);
        }
        Commands::Boxscore {game_id, defensive} => {
            if defensive {
                let boxscore = BoxScoreDefensive ::new(
                    nba::params::GameID::ID(game_id)
                );
                boxscore.save_to_db_file().unwrap();
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
