
    use core::fmt;
    use std::{fmt::Display};
    use chrono::format::strftime;


    pub trait NBAParam {
        fn get_formatted_param(&self) -> ();
    }

    pub enum LastNGames {
        N(i32)
    }

    pub enum GameID {
        ID(String)
    }

    pub enum Period {
        P(i8) 
    }

    pub enum LeagueID {
        NBA,
    }

    pub enum Season {
        S(String)
    }

    pub struct StartPeriod(Period);
    pub struct EndPeriod(Period);
    

    impl Display for GameID {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                GameID::ID(id) => {
                    write!(f, "GameID={}", id)
                }
            }
        }
    }

    impl Display for Period {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Period::P(period_num) => write!(f, "Period={}", period_num)
            }
            
        }
    }

    impl Display for StartPeriod {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self.0 {
                Period::P(period_num) => write!(f, "StartPeriod={}",period_num)
            }
            
        }
    }

    impl Display for EndPeriod {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self.0 {
                Period::P(period_num) => write!(f, "EndPeriod={}",period_num)
            }
            
        }
    }

    impl Display for Season {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Season::S(season) => write!(f, "Season={}", season)
            }
        }
    }

    impl Display for LeagueID {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                LeagueID::NBA => write!(f, "LeagueID=00")
            }
        }
    }

    impl Default for LeagueID {
        fn default() -> Self { LeagueID::NBA }
    }

    impl Default for Season {
        fn default() -> Self { 
            let current_date = chrono::Utc::now();
            let next_date = current_date - chrono::Duration::days(365);
            let first_year = next_date.format("%Y").to_string();
            let second_year = current_date.format("%y").to_string();
            Season::S(format!("{}-{}", first_year, second_year))
        }
    }

    impl Default for StartPeriod {
        fn default() -> Self { StartPeriod(Period::P(0)) }
    }

    impl Default for EndPeriod {
        fn default() -> Self { EndPeriod(Period::P(0)) }
    }

    impl Default for GameID {
        fn default() -> Self { GameID::ID("".to_string()) }
    }

    impl Default for Period {
        fn default() -> Self { Period::P(0) }
    }

    impl Default for LastNGames {
        fn default() -> Self { LastNGames::N(0) }
    }
