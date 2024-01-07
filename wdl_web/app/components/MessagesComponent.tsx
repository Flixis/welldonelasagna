import React, { useState, useEffect } from 'react';

interface Message {
  Id: number;
  MessageId: number;
  ChannelId: number;
  UserId: number;
  Name: string;
  Content: string;
  Timestamp: string;
  PremiumType: string;
}

const MessagesComponent = () => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState<boolean>(true);

  useEffect(() => {
    fetch('/api/messages')
      .then(response => response.json())
      .then(data => {
        setMessages(data);
        setLoading(false);
      })
      .catch(error => console.error('Error fetching data: ', error));
  }, []);

  if (loading) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      {messages.map(message => (
        <div key={message.Id}>
          <p>Name: {message.Name}</p>
          <p>Message: {message.Content}</p>
          <p>Timestamp: {message.Timestamp}</p>
          {/* Display other fields as needed */}
        </div>
      ))}
    </div>
  );
};

export default MessagesComponent;
