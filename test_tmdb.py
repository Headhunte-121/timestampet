import requests

api_key = "dummy_key"
url = "https://api.themoviedb.org/3/search/multi"
params = {"api_key": api_key, "query": "matrix"}
headers = {"accept": "application/json"}

try:
    response = requests.get(url, params=params, headers=headers)
    response.raise_for_status()
    print("Success")
except Exception as e:
    print(f"Error: type={type(e)}, {e}")
