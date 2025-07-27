# Morphius-tt

Task description:
```text
website using rust that has 2 components
backend: connects to some api that gets bitcoin price, like coingecko and scrapes it on an interval, then streams the price and timestamp etc. over websockets. Use axum
frontend: displaying a chart that receives websocket messages from backend and renders them on the chart in real time

backend should be built with docker and frontend should be a single html file with js file, no build step or compilation needed
extra credit: store data in db as you scrape then on initial load serve db stored data

doesn't need to look nice. No AI shit code
```

## Results

Demo live: http://31.220.74.111:3000/

<img width="1072" height="649" alt="image" src="https://github.com/user-attachments/assets/4681711a-d4dc-404d-ab2e-5fc3daf2b0e8" />

Highlights of the implementation:

* The app stores data in memory (for speed) using a BTreeMap while also persisting it in a PostgreSQL database
* Space-efficient data storage strategy:
  * 60 points for each second for the last minute
  * 60 points for each minute for the last hour
  * 24 points for each hour for the last day
  * 1 point per day for older data
