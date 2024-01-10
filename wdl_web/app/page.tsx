'use client'
import Navbar from './components/Navbar';
import CountMessagesByUsers from './components/CountMessageByUser';
import MessagesPerMonth from './components/MessagesPerMonth';

export default function Home() {
  return (
    <>
      <Navbar />
      <div style={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'space-between' }}>
        <div style={{ flex: '1 1 30%' }}> {/* Ensure that each child div takes up to 30% */}
          <CountMessagesByUsers />
        </div>
        <div style={{ flex: '1 1 30%' }}>
          <MessagesPerMonth />
        </div>
        {/* Add another div for the third component */}
        <div style={{ flex: '1 1 30%' }}>
          {/* Your third component */}
        </div>
      </div>
      <MessagesPerMonth />
    </>
  )
}


