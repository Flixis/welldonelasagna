// In pages/api/data.ts
import { NextApiRequest, NextApiResponse } from 'next';
import connection from './connection';

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    const query = `
    SELECT
    UserId,
    Name,
    COUNT(MessageId) AS TotalMessages,
    DATE_FORMAT(Timestamp, '%Y-%m') AS Month
FROM
    wdl_database.discord_messages
WHERE
    Name IN ('theycallmeq', 'jbuwu', 'snozledozle', 'thefyreprophecy', 'joppertje','lykozen','coeus._','coeus7680')
GROUP BY
    UserId, Name, DATE_FORMAT(Timestamp, '%Y-%m')
ORDER BY
    DATE_FORMAT(Timestamp, '%Y-%m');
  `;

  connection.query(query, (err, results) => {
    if (err) {
      return res.status(500).json({ message: err.message });
    }
    res.status(200).json(results);
  });
}
