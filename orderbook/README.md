# Orderbook Backend

A TypeScript Node.js backend with Express.js and PostgreSQL integration.

## Setup

1. Install dependencies:
```bash
npm install
```

2. Set up environment variables:
Create a `.env` file in the root directory with:
```
PORT=3000
DATABASE_URL=postgresql://username:password@localhost:5432/orderbook
NODE_ENV=development
MNEMONIC=your twelve word mnemonic phrase here for address generation
```

3. Build the project:
```bash
npm run build
```

4. Start the server:
```bash
npm start
```

## Development

For development with auto-reload:
```bash
npm run dev
```

## Environment Variables

- `PORT` - Server port (default: 3000)
- `DATABASE_URL` - PostgreSQL connection string (required)
- `NODE_ENV` - Environment (development/production)
- `MNEMONIC` - 12-word mnemonic phrase for address generation (required)

## API Endpoints

- `GET /health` - Returns "Online" status
- `GET /orders` - Get all orders
- `POST /orders` - Create a new order
- `GET /orders/:id` - Get order by ID

### Get All Orders

**GET** `/orders`

Response:
```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "source_asset": "BTC",
      "destination_asset": "zkBTC",
      "source_address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
      "destination_address": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
      "deposit_address": "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
      "amount": 100000000,
      "status": "created",
      "created_at": "2024-01-01T00:00:00.000Z",
      "proof_bytes": null,
      "public_values": null
    },
    {
      "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
      "source_asset": "ETH",
      "destination_asset": "zkETH",
      "source_address": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
      "destination_address": "0x8ba1f109551bD432803012645Hac136c",
      "deposit_address": "1CvBMSEYstWetqTFn5Au4m4GFg7xJaNVN3",
      "amount": 200000000,
      "status": "deposit_detected",
      "created_at": "2024-01-01T01:00:00.000Z",
      "proof_bytes": null,
      "public_values": null
    }
  ]
}
```

### Create Order

**POST** `/orders`

Request body:
```json
{
  "source_asset": "BTC",
  "destination_asset": "zkBTC",
  "source_address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
  "destination_address": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
  "amount": 100000000
}
```

**Note**: A unique `deposit_address` is automatically generated from the mnemonic for each order.

Response:
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "source_asset": "BTC",
    "destination_asset": "zkBTC",
    "source_address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
    "destination_address": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
    "deposit_address": "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
    "amount": 100000000,
    "status": "created",
    "created_at": "2024-01-01T00:00:00.000Z",
    "proof_bytes": null,
    "public_values": null
  }
}
```

## Project Structure

- `src/index.ts` - Main server file
- `src/config/` - Configuration files
  - `database.ts` - PostgreSQL connection setup
  - `index.ts` - Environment configuration
- `src/routes/` - API routes
  - `orders.ts` - Order management endpoints
- `src/services/` - Business logic
  - `orderService.ts` - Order operations
- `src/types/` - TypeScript type definitions
  - `order.ts` - Order-related types
- `src/database/` - Database schema
  - `schema.sql` - Orders table schema
- `dist/` - Compiled JavaScript output
- `package.json` - Dependencies and scripts
- `tsconfig.json` - TypeScript configuration
