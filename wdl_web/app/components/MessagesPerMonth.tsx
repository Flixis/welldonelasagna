import React, { useState, useEffect } from 'react';
import Barchart from './Barchart';
import { ChartData, ChartOptions } from 'chart.js';

interface Message {
    UserId: number;
    Name: string;
    TotalMessages: number;
    Month: string; // Assuming you have a Month property in your data
}

// Define a type for the structure of dataByUser
interface DataByUser {
    [key: string]: { [key: string]: number }; // Each user has a record of months with message counts
}

const MessagesPerMonth = () => {
    const [messages, setMessages] = useState<Message[]>([]);
    const [loading, setLoading] = useState<boolean>(true);

    useEffect(() => {
        fetch('/api/messagespermonth')
            .then(response => response.json())
            .then(data => {
                setMessages(data);
                setLoading(false);
            })
            .catch(error => console.error('Error fetching data: ', error));
    }, []);

    // Processing data for Chart.js with proper typing
    const dataByUser = messages.reduce<DataByUser>((acc, message) => {
        const user = acc[message.Name] || {};
        user[message.Month] = message.TotalMessages;
        acc[message.Name] = user;
        return acc;
    }, {});

    const months = [...new Set(messages.map(message => message.Month))].sort();

    // Function to generate a random color
    const getRandomColor = () => {
        const r = Math.floor(Math.random() * 256);
        const g = Math.floor(Math.random() * 256);
        const b = Math.floor(Math.random() * 256);
        return `rgba(${r}, ${g}, ${b}, 0.5)`;
    };

    // Create datasets
    const datasets = Object.keys(dataByUser).map(name => {
        return {
            label: name,
            data: months.map(month => dataByUser[name][month] || 0),
            backgroundColor: getRandomColor()
        };
    });

    const chartData: ChartData = {
        labels: months,
        datasets: datasets
    };

    const chartOptions: ChartOptions = {
        scales: {
            x: {
                stacked: true
            },
            y: {
                stacked: true
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

export default MessagesPerMonth;
