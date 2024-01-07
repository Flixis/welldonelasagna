'use client'
import Homepage from "@/components/Homepage"
import Navbar from '@/components/Navbar'
import Barchart from '@/components/Barchart';
import Linechart from '@/components/Linechart';

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
      <div style={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'space-around', padding: '5px' }}>
        <div style={{ width: '50%', minWidth: '300px', padding: '5px' }}>
          <Barchart data={data} options={{...options, maintainAspectRatio: true, responsive: true }} />
        </div>
        <div style={{ width: '50%', minWidth: '300px', padding: '5px' }}>
          <Linechart data={data} options={{...options, maintainAspectRatio: true, responsive: true }} />
        </div>
        <div style={{ width: '50%', minWidth: '300px', padding: '5px' }}>
          <Barchart data={data2} options={{...options, maintainAspectRatio: true, responsive: true }} />
        </div>
        <div style={{ width: '50%', minWidth: '300px', padding: '5px' }}>
          <Linechart data={data2} options={{...options, maintainAspectRatio: true, responsive: true }} />
        </div>
      </div>
    </>
  )
}

