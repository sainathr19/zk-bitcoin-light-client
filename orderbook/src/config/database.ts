import { Pool, PoolClient } from 'pg';
import fs from 'fs';
import path from 'path';

let pool: Pool | null = null;

export const connectToDatabase = async (): Promise<void> => {
  try {
    const databaseUrl = process.env.DATABASE_URL;
    
    if (!databaseUrl) {
      throw new Error('DATABASE_URL environment variable is not set');
    }

    pool = new Pool({
      connectionString: databaseUrl,
      ssl: process.env.NODE_ENV === 'production' ? { rejectUnauthorized: false } : false,
    });

    // Test the connection
    const client = await pool.connect();
    await client.query('SELECT NOW()');
    
    // Run schema migration
    await runSchemaMigration(client);
    
    client.release();
    
    console.log('Connected to PostgreSQL successfully');
  } catch (error) {
    console.error('Error connecting to PostgreSQL:', error);
    process.exit(1);
  }
};

const runSchemaMigration = async (client: PoolClient): Promise<void> => {
  try {
    const schemaPath = path.join(__dirname, '../database/schema.sql');
    const schema = fs.readFileSync(schemaPath, 'utf8');
    
    await client.query(schema);
    console.log('Database schema migration completed');
  } catch (error) {
    console.error('Error running schema migration:', error);
    throw error;
  }
};

export const getPool = (): Pool => {
  if (!pool) {
    throw new Error('Database pool not initialized. Call connectToDatabase() first.');
  }
  return pool;
};

export const disconnectFromDatabase = async (): Promise<void> => {
  try {
    if (pool) {
      await pool.end();
      pool = null;
      console.log('Disconnected from PostgreSQL');
    }
  } catch (error) {
    console.error('Error disconnecting from PostgreSQL:', error);
  }
};
