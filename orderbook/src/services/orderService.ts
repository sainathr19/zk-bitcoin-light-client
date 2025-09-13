import { getPool } from '../config/database';
import { CreateOrderRequest, Order, OrderResponse } from '../types/order';

export class OrderService {
  static async createOrder(orderData: CreateOrderRequest): Promise<OrderResponse> {
    try {
      const pool = getPool();
      
      const query = `
        INSERT INTO orders (source_asset, destination_asset, amount, status)
        VALUES ($1, $2, $3, 'created')
        RETURNING id, source_asset, destination_asset, amount, status, created_at, proof_bytes, public_outputs
      `;
      
      const values = [
        orderData.source_asset,
        orderData.destination_asset,
        orderData.amount
      ];
      
      const result = await pool.query(query, values);
      const order = result.rows[0];
      
      return {
        success: true,
        data: {
          id: order.id,
          source_asset: order.source_asset,
          destination_asset: order.destination_asset,
          amount: parseInt(order.amount),
          status: order.status,
          created_at: order.created_at,
          proof_bytes: order.proof_bytes,
          public_outputs: order.public_outputs
        }
      };
    } catch (error) {
      console.error('Error creating order:', error);
      return {
        success: false,
        error: 'Failed to create order'
      };
    }
  }

  static async getOrderById(id: number): Promise<OrderResponse> {
    try {
      const pool = getPool();
      
      const query = `
        SELECT id, source_asset, destination_asset, amount, status, created_at, proof_bytes, public_outputs
        FROM orders
        WHERE id = $1
      `;
      
      const result = await pool.query(query, [id]);
      
      if (result.rows.length === 0) {
        return {
          success: false,
          error: 'Order not found'
        };
      }
      
      const order = result.rows[0];
      
      return {
        success: true,
        data: {
          id: order.id,
          source_asset: order.source_asset,
          destination_asset: order.destination_asset,
          amount: parseInt(order.amount),
          status: order.status,
          created_at: order.created_at,
          proof_bytes: order.proof_bytes,
          public_outputs: order.public_outputs
        }
      };
    } catch (error) {
      console.error('Error fetching order:', error);
      return {
        success: false,
        error: 'Failed to fetch order'
      };
    }
  }

  static async getAllOrders(): Promise<{ success: boolean; data?: Order[]; error?: string }> {
    try {
      const pool = getPool();
      
      const query = `
        SELECT id, source_asset, destination_asset, amount, status, created_at, proof_bytes, public_outputs
        FROM orders
        ORDER BY created_at DESC
      `;
      
      const result = await pool.query(query);
      
      const orders = result.rows.map(order => ({
        id: order.id,
        source_asset: order.source_asset,
        destination_asset: order.destination_asset,
        amount: parseInt(order.amount),
        status: order.status,
        created_at: order.created_at,
        proof_bytes: order.proof_bytes,
        public_outputs: order.public_outputs
      }));
      
      return {
        success: true,
        data: orders
      };
    } catch (error) {
      console.error('Error fetching all orders:', error);
      return {
        success: false,
        error: 'Failed to fetch orders'
      };
    }
  }
}
