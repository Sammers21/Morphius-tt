document.addEventListener('DOMContentLoaded', function () {
    const ctx = document.getElementById('chart').getContext('2d');
    const pointsCountDisplay = document.getElementById('points-count');
    const latestPriceDisplay = document.getElementById('latest-price');
    const latestTimeDisplay = document.getElementById('latest-time');
    const timestamps = [];
    const prices = [];
    const chart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: timestamps,
            datasets: [{
                label: 'Bitcoin Price (USD)',
                data: prices,
                borderColor: 'rgb(75, 192, 192)',
                backgroundColor: 'rgba(75, 192, 192, 0.1)',
                borderWidth: 2,
                tension: 0.3,
                fill: true,
                pointRadius: 2,
                pointHoverRadius: 4
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: {
                    type: "time",
                    display: true,
                    title: {
                        display: true,
                        text: 'Time'
                    },
                    time: {
                        displayFormats: {
                            second: 'HH:mm:ss',
                            minute: 'HH:mm',
                            hour: 'HH:mm',
                            day: 'MMM DD',
                            week: 'MMM DD',
                            month: 'MMM YYYY',
                            quarter: 'MMM YYYY',
                            year: 'YYYY'
                        }
                    }
                },
                y: {
                    display: true,
                    title: {
                        display: true,
                        text: 'Price (USD)'
                    },
                }
            },
            animation: {
                duration: 0
            }
        }
    });

    function formatPrice(price) {
        if (price === null || price === undefined || isNaN(price)) {
            return '$0.00';
        }
        return '$' + price;
    }

    function formatTimestamp(timestamp) {
        const date = new Date(timestamp * 1000);
        return date.toLocaleTimeString();
    }

    function updateChart(price, timestamp) {
        if (price === null || price === undefined || timestamp === null || timestamp === undefined) {
            return;
        }
        let tsFormatted = formatTimestamp(timestamp);
        timestamps.push(new Date(timestamp * 1000));
        prices.push(price);
        chart.update();
        pointsCountDisplay.textContent = prices.length;
        latestPriceDisplay.textContent = formatPrice(price);
        latestTimeDisplay.textContent = tsFormatted;
    }

    function connectWebSocket() {
        const socket = new WebSocket(`ws://${window.location.host}/ws`);
        socket.addEventListener('message', function (event) {
            try {
                const data = JSON.parse(event.data);
                updateChart(data.price, data.timestamp);
            } catch (error) {
                console.error('Error parsing message:', error);
            }
        });
        socket.addEventListener('close', function () {
            console.log('WebSocket closed');
        });
        socket.addEventListener('error', function (event) {
            console.error('WebSocket error:', event);
            socket.close();
        });
    }

    connectWebSocket();
});