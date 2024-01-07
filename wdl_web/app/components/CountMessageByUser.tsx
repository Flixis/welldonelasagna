import React, { useState, useEffect } from 'react';
import Barchart from './Barchart';
import { ChartData, ChartOptions } from 'chart.js';

interface Message {
    Id: number;
    UserId: number;
    Name: string;
    TotalMessages: number;
}

const CountMessagesByUsers = () => {
    const [messages, setMessages] = useState<Message[]>([]);
    const [loading, setLoading] = useState<boolean>(true);

    useEffect(() => {
        fetch('/api/countmessagesbyuser')
            .then(response => response.json())
            .then(data => {
                setMessages(data);
                setLoading(false);
            })
            .catch(error => console.error('Error fetching data: ', error));
    }, []);

    const chartData: ChartData = {
        labels: messages.map(message => message.Name),
        datasets: [{
            label: 'Total Messages',
            data: messages.map(message => message.TotalMessages),
            backgroundColor: 'rgba(54, 162, 235, 0.5)',
            borderColor: 'rgba(54, 162, 235, 1)',
            borderWidth: 1
        }]
    };

    const chartOptions: ChartOptions = {
        scales: {
            y: {
                beginAtZero: true
            }
        }
    };

    if (loading) {
        return <div>Loading...</div>;
    }

    return (
        <div>
            <Barchart data={chartData} options={chartOptions} />
        </div>
    );
};

export default CountMessagesByUsers;
