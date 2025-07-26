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

<img width="2331" height="1398" alt="image" src="https://github.com/user-attachments/assets/d775b1a9-4515-4257-917b-1c9778ff38c1" />

