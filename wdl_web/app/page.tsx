'use client'
import Navbar from './components/Navbar';
import MessagesComponent from './components/MessagesComponent';

const data = {
    labels: ['January', 'February', 'March', 'April', 'May', 'June', 'July'],
    datasets: [{
        label: 'My First dataset',
        backgroundColor: 'rgb(255, 99, 132)',
        borderColor: 'rgb(255, 99, 132)',
        data: [0, 10, 5, 2, 20, 30, 45],
    }]
};

const data2 = {
  labels: ['January', 'February', 'March', 'April', 'May', 'June', 'July'],
  datasets: [{
      label: 'My Second dataset',
      backgroundColor: 'rgb(255, 255, 0)',
      borderColor: 'rgb(255, 255, 0)',
      data: [0, 10, 5, 2, 20, 30, 45],
  }]
};

const options = {
    scales: {
        y: {
            beginAtZero: true
        }
    }
};

export default function Home() {
  return (
    <>
      <Navbar />
      <MessagesComponent />
    </>
  )
}

