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
      "id": 1,
      "source_asset": "BTC",
      "destination_asset": "zkBTC",
      "amount": 100000000,
      "status": "created",
      "created_at": "2024-01-01T00:00:00.000Z",
      "proof_bytes": null,
      "public_outputs": null
    },
    {
      "id": 2,
      "source_asset": "ETH",
      "destination_asset": "zkETH",
      "amount": 200000000,
      "status": "deposit_detected",
      "created_at": "2024-01-01T01:00:00.000Z",
      "proof_bytes": null,
      "public_outputs": null
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
  "amount": 100000000
}
```

Response:
```json
{
  "success": true,
  "data": {
    "id": 1,
    "source_asset": "BTC",
    "destination_asset": "zkBTC",
    "amount": 100000000,
    "status": "created",
    "created_at": "2024-01-01T00:00:00.000Z",
    "proof_bytes": null,
    "public_outputs": null
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
