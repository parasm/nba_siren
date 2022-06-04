import requests
import matplotlib.pyplot as plt
import pandas
import seaborn as sns
from nba_api.stats.endpoints import shotchartdetail
from nba_api.stats.endpoints import shotchartlineupdetail
from nba_api.stats.endpoints import boxscoresummaryv2
from nba_api.stats.endpoints import teamdashptshots


from shotchart import ShotChartDetail
import json
from matplotlib.patches import Circle, Rectangle, Arc
from nba_api.stats.library.data import players

def draw_court(ax=None, color='black', lw=2, outer_lines=False):
    # If an axes object isn't provided to plot onto, just get current one
    if ax is None:
        ax = plt.gca()

    # Create the various parts of an NBA basketball court

    # Create the basketball hoop
    # Diameter of a hoop is 18" so it has a radius of 9", which is a value
    # 7.5 in our coordinate system
    hoop = Circle((0, 0), radius=7.5, linewidth=lw, color=color, fill=False)

    # Create backboard
    backboard = Rectangle((-30, -7.5), 60, -1, linewidth=lw, color=color)

    # The paint
    # Create the outer box 0f the paint, width=16ft, height=19ft
    outer_box = Rectangle((-80, -47.5), 160, 190, linewidth=lw, color=color,
                          fill=False)
    # Create the inner box of the paint, widt=12ft, height=19ft
    inner_box = Rectangle((-60, -47.5), 120, 190, linewidth=lw, color=color,
                          fill=False)

    # Create free throw top arc
    top_free_throw = Arc((0, 142.5), 120, 120, theta1=0, theta2=180,
                         linewidth=lw, color=color, fill=False)
    # Create free throw bottom arc
    bottom_free_throw = Arc((0, 142.5), 120, 120, theta1=180, theta2=0,
                            linewidth=lw, color=color, linestyle='dashed')
    # Restricted Zone, it is an arc with 4ft radius from center of the hoop
    restricted = Arc((0, 0), 80, 80, theta1=0, theta2=180, linewidth=lw,
                     color=color)

    # Three point line
    # Create the side 3pt lines, they are 14ft long before they begin to arc
    corner_three_a = Rectangle((-220, -47.5), 0, 140, linewidth=lw,
                               color=color)
    corner_three_b = Rectangle((220, -47.5), 0, 140, linewidth=lw, color=color)
    # 3pt arc - center of arc will be the hoop, arc is 23'9" away from hoop
    # I just played around with the theta values until they lined up with the 
    # threes
    three_arc = Arc((0, 0), 475, 475, theta1=22, theta2=158, linewidth=lw,
                    color=color)

    # Center Court
    center_outer_arc = Arc((0, 422.5), 120, 120, theta1=180, theta2=0,
                           linewidth=lw, color=color)
    center_inner_arc = Arc((0, 422.5), 40, 40, theta1=180, theta2=0,
                           linewidth=lw, color=color)

    # List of the court elements to be plotted onto the axes
    court_elements = [hoop, backboard, outer_box, inner_box, top_free_throw,
                      bottom_free_throw, restricted, corner_three_a,
                      corner_three_b, three_arc, center_outer_arc,
                      center_inner_arc]

    if outer_lines:
        # Draw the half court line, baseline and side out bound lines
        outer_lines = Rectangle((-250, -47.5), 500, 470, linewidth=lw,
                                color=color, fill=False)
        court_elements.append(outer_lines)

    # Add the court elements onto the axes
    for element in court_elements:
        ax.add_patch(element)

    return ax

STATS_HEADERS = {
    'Host': 'stats.nba.com',
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:72.0) Gecko/20100101 Firefox/72.0',
    'Accept': 'application/json, text/plain, */*',
    'Accept-Language': 'en-US,en;q=0.5',
    'Accept-Encoding': 'gzip, deflate, br',
    'x-nba-stats-origin': 'stats',
    'x-nba-stats-token': 'true',
    'Connection': 'keep-alive',
    'Referer': 'https://stats.nba.com/',
    'Pragma': 'no-cache',
    'Cache-Control': 'no-cache',
}

team_dash = teamdashptshots.TeamDashPtShots(
    league_id="00",
    last_n_games="1",
    month="0",
    period="0",
    season="2021-22",
    season_type_all_star="Playoffs",
    team_id="1610612744",
)
print(dir(team_dash))
print(team_dash.nba_response.get_url())
print(team_dash.get_json())

# s = shotchart.ShotChart(player_id="202710", game_id="0042100206",  context_measure="FGA")
# print(dir(s))
#print(s.json())

# b = boxscoresummaryv2.BoxScoreSummaryV2(game_id="0022100560")
# print(dir(b))

# plt.figure(figsize=(12,11))
# plt.axis('off')
# draw_court(outer_lines=True)
# plt.xlim(-300,300)
# plt.ylim(-100,500)
# plt.show()

# shotchartlineupdetail.ShotChartLineupDetail(
#     group_id="",
#     context_measure_detailed="FGM",
#     season="2021-22"
# )

url = "https://stats.nba.com/stats/shotchartdetail"#?AheadBehind=&ClutchTime=&ContextFilter=&ContextMeasure=FGA&DateFrom=&DateTo=&EndPeriod=&EndRange=&GameID=0022100560&GameSegment=&LastNGames=0&LeagueID=00&Location=&Month=0&OpponentTeamID=0&Outcome=&Period=0&PlayerID=1629027&PlayerPosition=&PointDiff=&Position=&RangeType=&RookieYear=&Season=&SeasonSegment=&SeasonType=Regular+Season&StartPeriod=&StartRange=&TeamID=1610612737&VsConference=&VsDivision="


# shot_chart = shotchartdetail.ShotChartDetail(
#     context_measure_simple="FGA",
#     player_id="203507",
#     team_id="1610612749",
#     season_type_all_star="Playoffs",
# )
# print(shot_chart.nba_response.get_url())
# print(shot_chart.get_json())
# j = json.loads(shot_chart.get_json())
# headers = j['resultSets'][0]['headers']
# shots = j['resultSets'][0]['rowSet']
# shot_df = pandas.DataFrame(shots, columns=headers)


# plt.figure(figsize=(12,11))
# plt.scatter(shot_df.LOC_X, shot_df.LOC_Y)
# draw_court()
# # Adjust plot limits to just fit in half court
# plt.xlim(-250,250)
# # Descending values along th y axis from bottom to top
# # in order to place the hoop by the top of plot
# plt.ylim(422.5, -47.5)
# # get rid of axis tick labels
# # plt.tick_params(labelbottom=False, labelleft=False)
# plt.show()


# r = requests.get(url.lower(),params=shot_chart.parameters, headers=STATS_HEADERS)
# print(r.text)
# print(shot_chart.shot_chart_detail.get_data_frame())
# print(dir(shot_chart))
# dfs = shot_chart.get_data_frames()
# print(len(dfs))
# print(dfs[0])