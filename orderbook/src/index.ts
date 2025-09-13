import express, { Request, Response } from 'express';
import cors from 'cors';
import { connectToDatabase, disconnectFromDatabase } from './config/database';
import { config } from './config';
import ordersRouter from './routes/orders';
import { AddressGenerator } from './services/addressGenerator';

const app = express();

// Middleware
app.use(cors());
app.use(express.json());

// Routes
app.use('/orders', ordersRouter);

// Health endpoint
app.get('/health', (req: Request, res: Response) => {
  res.status(200).send('Online');
});

// Start server
const startServer = async () => {
  try {
    // Connect to PostgreSQL
    await connectToDatabase();
    
    // Initialize address generator with mnemonic
    if (!config.mnemonic) {
      throw new Error('MNEMONIC environment variable is required');
    }
    AddressGenerator.initialize(config.mnemonic);
    
    // Start the server
    app.listen(config.port, () => {
      console.log(`✅ Server is running on port ${config.port}`);
      console.log(`✅ PostgreSQL database connection established`);
      console.log(`✅ Address generator initialized`);
      console.log(`✅ Health check available at http://localhost:${config.port}/health`);
      console.log(`✅ Orders API available at http://localhost:${config.port}/orders`);
    });
  } catch (error) {
    console.error('❌ Failed to start server:', error);
    process.exit(1);
  }
};

// Graceful shutdown
process.on('SIGINT', async () => {
  console.log('Received SIGINT, shutting down gracefully...');
  await disconnectFromDatabase();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  console.log('Received SIGTERM, shutting down gracefully...');
  await disconnectFromDatabase();
  process.exit(0);
});

// Start the server
startServer();

export default app;
