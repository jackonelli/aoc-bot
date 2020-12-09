import requests


def fetch_aoc_json():
    headers = {
        "Cookie": "session=53616c7465645f5f3c5ced8a2f6dca75760bc2f8953f91ae1465d71815a8367c5f2cb85de1a2d3c479e134979834eb40"
    }
    res = requests.get(
        "https://adventofcode.com/2020/leaderboard/private/view/152507.json",
        headers=headers,
    )
    aoc_json = res.json()
    print(aoc_json)


if __name__ == "__main__":
    fetch_aoc_json()
