// Ticket API client.
//
// This is the single boundary the UI talks to. It maps 1:1 to the Rust
// `commands::ticket_*` handlers via invoke(). Swapping transport later (e.g. a
// remote HTTP fallback) only touches this file.

import { invoke } from "@/lib/invoke";
import type { CreateTicketInput, Ticket, ValidationResult } from "@/lib/types";

export const ticketsApi = {
  list(): Promise<Ticket[]> {
    return invoke<Ticket[]>("ticket_list");
  },

  get(id: number): Promise<Ticket> {
    return invoke<Ticket>("ticket_get", { id });
  },

  create(input: CreateTicketInput): Promise<Ticket> {
    // Rust params are snake_case; Tauri maps camelCase keys, so pass camelCase.
    return invoke<Ticket>("ticket_create", {
      validDate: input.valid_date,
      invoiceId: input.invoice_id ?? null,
    });
  },

  updateStatus(id: number, status: string): Promise<Ticket> {
    return invoke<Ticket>("ticket_update", { id, status });
  },

  remove(id: number): Promise<void> {
    return invoke<void>("ticket_delete", { id });
  },

  validate(ticketCode: string): Promise<ValidationResult> {
    return invoke<ValidationResult>("ticket_validate", { ticketCode });
  },
};
