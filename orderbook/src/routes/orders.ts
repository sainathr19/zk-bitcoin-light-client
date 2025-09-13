import { Router, Request, Response } from 'express';
import { OrderService } from '../services/orderService';
import { CreateOrderRequest } from '../types/order';

const router = Router();

// Get all orders endpoint
router.get('/', async (req: Request, res: Response) => {
  try {
    const result = await OrderService.getAllOrders();

    if (result.success) {
      res.status(200).json(result);
    } else {
      res.status(500).json(result);
    }
  } catch (error) {
    console.error('Error in get all orders endpoint:', error);
    res.status(500).json({
      success: false,
      error: 'Internal server error'
    });
  }
});

// Create order endpoint
router.post('/', async (req: Request, res: Response) => {
  try {
    const { source_asset, destination_asset, source_address, destination_address, amount }: CreateOrderRequest = req.body;

    // Validation
    if (!source_asset || !destination_asset || !source_address || !destination_address || amount === undefined) {
      return res.status(400).json({
        success: false,
        error: 'Missing required fields: source_asset, destination_asset, source_address, destination_address, amount'
      });
    }

    if (typeof amount !== 'number' || amount <= 0) {
      return res.status(400).json({
        success: false,
        error: 'Amount must be a positive number'
      });
    }

    if (typeof source_asset !== 'string' || typeof destination_asset !== 'string' || 
        typeof source_address !== 'string' || typeof destination_address !== 'string') {
      return res.status(400).json({
        success: false,
        error: 'source_asset, destination_asset, source_address, and destination_address must be strings'
      });
    }

    const result = await OrderService.createOrder({
      source_asset,
      destination_asset,
      source_address,
      destination_address,
      amount
    });

    if (result.success) {
      res.status(201).json(result);
    } else {
      res.status(500).json(result);
    }
  } catch (error) {
    console.error('Error in create order endpoint:', error);
    res.status(500).json({
      success: false,
      error: 'Internal server error'
    });
  }
});

// Get order by ID endpoint
router.get('/:id', async (req: Request, res: Response) => {
  try {
    const id = req.params.id;

    // Basic UUID format validation
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
    if (!uuidRegex.test(id)) {
      return res.status(400).json({
        success: false,
        error: 'Invalid order ID format'
      });
    }

    const result = await OrderService.getOrderById(id);

    if (result.success) {
      res.status(200).json(result);
    } else {
      res.status(404).json(result);
    }
  } catch (error) {
    console.error('Error in get order endpoint:', error);
    res.status(500).json({
      success: false,
      error: 'Internal server error'
    });
  }
});

export default router;
