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
    const { source_asset, destination_asset, amount }: CreateOrderRequest = req.body;

    // Validation
    if (!source_asset || !destination_asset || amount === undefined) {
      return res.status(400).json({
        success: false,
        error: 'Missing required fields: source_asset, destination_asset, amount'
      });
    }

    if (typeof amount !== 'number' || amount <= 0) {
      return res.status(400).json({
        success: false,
        error: 'Amount must be a positive number'
      });
    }

    if (typeof source_asset !== 'string' || typeof destination_asset !== 'string') {
      return res.status(400).json({
        success: false,
        error: 'source_asset and destination_asset must be strings'
      });
    }

    const result = await OrderService.createOrder({
      source_asset,
      destination_asset,
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
    const id = parseInt(req.params.id);

    if (isNaN(id)) {
      return res.status(400).json({
        success: false,
        error: 'Invalid order ID'
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
