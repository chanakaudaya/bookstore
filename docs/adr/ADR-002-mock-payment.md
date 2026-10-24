# ADR-002: Mock Payment Processor for MVP

**Status:** Accepted

## Context

A real payment gateway integration (e.g. Stripe) requires PCI compliance, live test credentials, and webhook handling — not appropriate for MVP scope.

## Decision

Implement a **mock payment processor** abstraction in the checkout module:

- A `mock_process_payment(amount) -> MockPaymentResult` function always approves non-zero amounts.
- The `payment_methods` table stores only safe metadata (`last_four`, `token_reference`) — no raw card data.
- The checkout flow is structured as if a real gateway were present (idempotency key, payment status field, transaction record) so it can be replaced with a real gateway later.

## Consequences

- MVP can be developed and tested without live credentials.
- The schema and checkout flow are compatible with future real gateway integration.
- `token_reference` field is reserved for the future gateway token.
