## NBA CLI Tool

`cargo run -- --help`
```
USAGE:
    nbasiren.exe <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    boxscore
    help              Print this message or the help of the given subcommand(s)
    lookup
    playbyplay
    savestaticdata
    scoreboard
    test
    vidforplay
```

`cargo run scoreboard`
```
game_id: 0042100306
Home: (BOS) Boston Celtics
Away: (MIA) Miami Heat
+----+----+----+----+-------+
| q1 | q2 | q3 | q4 | total |
+----+----+----+----+-------+
| 22 | 24 | 27 | 0  |  73   |
+----+----+----+----+-------+
| 29 | 19 | 34 | 0  |  82   |
+----+----+----+----+-------+
```

`cargo run playbyplay 0042100315`
```
PERSON1TYPE : 5, NEUTRALDESCRIPTION : "", PLAYER2_TEAM_CITY : "dallas", EVENTMSGTYPE : 1, PLAYER2_TEAM_NICKNAME : "mavericks", PERSON3TYPE : 0, PLAYER3_ID : 0, PLAYER1_ID : 203504, PLAYER2_ID : 1628425, PCTIMESTRING : "0:24", PERSON2TYPE : 5, PERIOD : 4, HOMEDESCRIPTION : "", SCORE : "110 - 120", PLAYER1_TEAM_NICKNAME : "mavericks", PLAYER3_TEAM_NICKNAME : "", EVENTNUM : 637, PLAYER1_TEAM_ABBREVIATION : "dal", VISITORDESCRIPTION : "burke 26' 3pt running pull-up jump shot (6 pts) (brown 2 ast)", PLAYER2_TEAM_ID : 1610612742, WCTIMESTRING : "11:20 pm", PLAYER1_TEAM_ID : 1610612742, PLAYER2_NAME : "sterling brown", SCOREMARGIN : "10", PLAYER2_TEAM_ABBREVIATION : "dal", PLAYER3_NAME : "", PLAYER1_TEAM_CITY : "dallas", PLAYER3_TEAM_ID : 0, PLAYER3_TEAM_ABBREVIATION : "", VIDEO_AVAILABLE_FLAG : 1, PLAYER3_TEAM_CITY : "", GAME_ID : "0042100315", PLAYER1_NAME : "trey burke", EVENTMSGACTIONTYPE : 103, 
PERSON1TYPE : 0, NEUTRALDESCRIPTION : "end of 4th period (11:21 pm est)", PLAYER2_TEAM_CITY : "", EVENTMSGTYPE : 13, PLAYER2_TEAM_NICKNAME : "", PERSON3TYPE : 0, PLAYER3_ID : 0, PLAYER1_ID : 0, PLAYER2_ID : 0, PCTIMESTRING : "0:00", PERSON2TYPE : 0, PERIOD : 4, HOMEDESCRIPTION : "", SCORE : "110 - 120", PLAYER1_TEAM_NICKNAME : "", PLAYER3_TEAM_NICKNAME : "", EVENTNUM : 639, PLAYER1_TEAM_ABBREVIATION : "", VISITORDESCRIPTION : "", PLAYER2_TEAM_ID : 0, WCTIMESTRING : "11:21 pm", PLAYER1_TEAM_ID : 0, PLAYER2_NAME : "", SCOREMARGIN : "10", PLAYER2_TEAM_ABBREVIATION : "", PLAYER3_NAME : "", PLAYER1_TEAM_CITY : "", PLAYER3_TEAM_ID : 0, PLAYER3_TEAM_ABBREVIATION : "", VIDEO_AVAILABLE_FLAG : 1, PLAYER3_TEAM_CITY : "", GAME_ID : "0042100315", PLAYER1_NAME : "", EVENTMSGACTIONTYPE : 0, 
shape: (5, 34)
+-------------+--------------------+-------------------+--------------+-----+-------------------+--------------+-----------------------+--------------------+
| PERSON1TYPE | NEUTRALDESCRIPTION | PLAYER2_TEAM_CITY | EVENTMSGTYPE | ... | PLAYER3_TEAM_CITY | GAME_ID      | PLAYER1_NAME          | EVENTMSGACTIONTYPE |
| ---         | ---                | ---               | ---          |     | ---               | ---          | ---                   | ---                |
| i64         | str                | str               | i64          |     | str               | str          | str                   | i64                |
+=============+====================+===================+==============+=====+===================+==============+=======================+====================+
| 4           | ""                 | "dallas"          | 10           | ... | "golden state"    | "0042100315" | "kevon looney"        | 0                  |
+-------------+--------------------+-------------------+--------------+-----+-------------------+--------------+-----------------------+--------------------+
```

`cargo run playbyplay <game_id> -p <player_id> -k <keyword> -s <save_video>`

Example Get all plays from BOS|MIA Game 7, involving Jayson Tatum and a block
cargo run playbyplay [0042100307](https://www.nba.com/game/bos-vs-mia-0042100307/box-score#box-score) -p [1628369](https://www.nba.com/player/1628369/jayson-tatum) -k "block" -s
```
Saving exit code: 0, status: ALL_PLAYS_block_0042100307_1628369.mp4
Loading game df took 160.2435ms
Saving videos took 3.4320608s
Combining videos took 79.1803ms
```
