import mysql from 'mysql2';

interface DbConfig {
    host: string;
    user: string;
    password: string;
    database: string;
}
  
const config: DbConfig = {
    host: process.env.DB_HOST || '',
    user: process.env.DB_USER || '',
    password: process.env.DB_PASSWORD || '',
    database: process.env.DB_DATABASE || '',
};

const pool = mysql.createPool(config);

export default pool;

