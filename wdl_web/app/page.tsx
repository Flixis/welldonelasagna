'use client'
import Navbar from './components/Navbar';
import CountMessagesByUsers from './components/CountMessageByUser';

export default function Home() {
  return (
    <>
      <Navbar />
      <div style={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'space-around', padding: '5px' }}>
        <div style={{ width: '50%', minWidth: '300px', padding: '5px' }}>
          <CountMessagesByUsers />
        </div>
      </div>
    </>
  )
}

