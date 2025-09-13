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

## Project Structure

- `src/index.ts` - Main server file
- `src/config/` - Configuration files
  - `database.ts` - PostgreSQL connection setup
  - `index.ts` - Environment configuration
- `dist/` - Compiled JavaScript output
- `package.json` - Dependencies and scripts
- `tsconfig.json` - TypeScript configuration
