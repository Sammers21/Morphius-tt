document.addEventListener('DOMContentLoaded', function() {
    // DOM elements
    const connectionStatus = document.getElementById('connection-status');
    const priceDisplay = document.getElementById('price-display');
    const ctx = document.getElementById('priceChart').getContext('2d');
    
    // Data arrays for the chart
    const timestamps = [];
    const prices = [];
    const maxDataPoints = 60; // Show 1 minute of data
    
    // Initialize Chart.js
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
                fill: true
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: {
                    display: true,
                    title: {
                        display: true,
                        text: 'Time'
                    }
                },
                y: {
                    display: true,
                    title: {
                        display: true,
                        text: 'Price (USD)'
                    },
                    suggestedMin: 90000,
                    suggestedMax: 110000
                }
            },
            animation: {
                duration: 0 // Disable animation for better performance
            }
        }
    });
    
    // Format price for display
    function formatPrice(price) {
        return '$' + price.toLocaleString('en-US', {
            minimumFractionDigits: 2,
            maximumFractionDigits: 2
        });
    }
    
    // Format timestamp for display
    function formatTimestamp(timestamp) {
        const date = new Date(timestamp * 1000);
        return date.toLocaleTimeString();
    }
    
    // Update chart with new data
    function updateChart(price, timestamp) {
        // Add new data
        timestamps.push(formatTimestamp(timestamp));
        prices.push(price);
        
        // Remove old data if we have more than maxDataPoints
        if (timestamps.length > maxDataPoints) {
            timestamps.shift();
            prices.shift();
        }
        
        // Update chart
        chart.update();
        
        // Update price display
        priceDisplay.textContent = formatPrice(price);
    }
    
    // Connect to WebSocket
    function connectWebSocket() {
        // Create WebSocket connection
        const socket = new WebSocket('ws://127.0.0.1:3000/ws');
        
        // Connection opened
        socket.addEventListener('open', function(event) {
            connectionStatus.textContent = 'Connected';
            connectionStatus.classList.remove('disconnected');
            connectionStatus.classList.add('connected');
        });
        
        // Connection closed
        socket.addEventListener('close', function(event) {
            connectionStatus.textContent = 'Disconnected - Reconnecting...';
            connectionStatus.classList.remove('connected');
            connectionStatus.classList.add('disconnected');
            
            // Try to reconnect after 1 second
            setTimeout(connectWebSocket, 1000);
        });
        
        // Listen for messages
        socket.addEventListener('message', function(event) {
            try {
                const data = JSON.parse(event.data);
                updateChart(data.price, data.timestamp);
            } catch (error) {
                console.error('Error parsing message:', error);
            }
        });
        
        // Connection error
        socket.addEventListener('error', function(event) {
            console.error('WebSocket error:', event);
            socket.close();
        });
    }
    
    // Start the WebSocket connection
    connectWebSocket();
});