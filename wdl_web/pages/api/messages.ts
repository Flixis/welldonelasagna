// In pages/api/data.ts
import { NextApiRequest, NextApiResponse } from 'next';
import connection from './connection';

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  const query = 'SELECT * FROM discord_messages';

  connection.query(query, (err, results) => {
    if (err) {
      return res.status(500).json({ message: err.message });
    }
    res.status(200).json(results);
  });
}
