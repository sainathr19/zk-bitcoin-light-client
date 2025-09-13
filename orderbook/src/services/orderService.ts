import { getPool } from '../config/database';
import { CreateOrderRequest, Order, OrderResponse } from '../types/order';
import { AddressGenerator } from './addressGenerator';

export class OrderService {
  private static orderCounter = 0;

  static async createOrder(orderData: CreateOrderRequest): Promise<OrderResponse> {
    try {
      const pool = getPool();
      
      // Generate deposit address from mnemonic
      const depositAddress = AddressGenerator.generateBitcoinAddress(this.orderCounter);
      this.orderCounter++;
      
      console.log(`🔑 Generated deposit address for order: ${depositAddress}`);
      
      const query = `
        INSERT INTO orders (source_asset, destination_asset, source_address, destination_address, deposit_address, amount, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'created')
        RETURNING id, source_asset, destination_asset, source_address, destination_address, deposit_address, amount, status, created_at, proof_bytes, public_values
      `;
      
      const values = [
        orderData.source_asset,
        orderData.destination_asset,
        orderData.source_address,
        orderData.destination_address,
        depositAddress,
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
          source_address: order.source_address,
          destination_address: order.destination_address,
          amount: parseInt(order.amount),
          status: order.status,
          created_at: order.created_at,
          proof_bytes: order.proof_bytes,
          public_values: order.public_values,
          deposit_address : depositAddress
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

  static async getOrderById(id: string): Promise<OrderResponse> {
    try {
      const pool = getPool();
      
      const query = `
        SELECT id, source_asset, destination_asset, source_address, destination_address, deposit_address, amount, status, created_at, proof_bytes, public_values
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
          source_address: order.source_address,
          destination_address: order.destination_address,
          amount: parseInt(order.amount),
          status: order.status,
          created_at: order.created_at,
          proof_bytes: order.proof_bytes,
          public_values: order.public_values,
          deposit_address: order.deposit_address
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
        SELECT id, source_asset, destination_asset, source_address, destination_address, deposit_address, amount, status, created_at, proof_bytes, public_values
        FROM orders
        ORDER BY created_at DESC
      `;
      
      const result = await pool.query(query);
      
      const orders = result.rows.map(order => ({
        id: order.id,
        source_asset: order.source_asset,
        destination_asset: order.destination_asset,
        source_address: order.source_address,
        destination_address: order.destination_address,
        deposit_address: order.deposit_address,
        amount: parseInt(order.amount),
        status: order.status,
        created_at: order.created_at,
        proof_bytes: order.proof_bytes,
        public_values: order.public_values
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
