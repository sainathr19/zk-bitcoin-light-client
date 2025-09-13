export interface CreateOrderRequest {
  source_asset: string;
  destination_asset: string;
  source_address: string;
  destination_address: string;
  amount: number; // Will be stored as BIGINT in database
}

export interface Order {
  id: string;
  source_asset: string;
  destination_asset: string;
  source_address: string;
  destination_address: string;
  deposit_address: string;
  amount: number;
  status: OrderStatus;
  created_at: Date;
  proof_bytes?: any;
  public_values?: any;
}

export type OrderStatus = 'created' | 'deposit_detected' | 'minting' | 'completed';

export interface OrderResponse {
  success: boolean;
  data?: Order;
  error?: string;
}
