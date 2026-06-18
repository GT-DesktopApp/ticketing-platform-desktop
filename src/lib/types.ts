// Shared domain types, mirrored from the Rust domain layer
// (src-tauri/src/domain). Keep these in sync with the Rust structs.

export interface Ticket {
  id: number;
  ticket_code: string;
  invoice_id: number | null;
  valid_date: string; // "YYYY-MM-DD"
  status: TicketStatus;
  used_at: string | null;
  created_at: string;
}

export type TicketStatus = "active" | "used" | "cancelled";

export interface ValidationResult {
  valid: boolean;
  reason: string;
  ticket_code: string | null;
  valid_date: string | null;
}

export interface CreateTicketInput {
  valid_date: string;
  invoice_id?: number | null;
}
