export interface CreateOrderRequest {
  source_asset: string;
  destination_asset: string;
  amount: number; // Will be stored as BIGINT in database
}

export interface Order {
  id: number;
  source_asset: string;
  destination_asset: string;
  amount: number;
  status: OrderStatus;
  created_at: Date;
  proof_bytes?: any;
  public_outputs?: any;
}

export type OrderStatus = 'created' | 'deposit_detected' | 'minting' | 'completed';

export interface OrderResponse {
  success: boolean;
  data?: Order;
  error?: string;
}
