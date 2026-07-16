# Auth & Billing

## Clerk

1. Create a Clerk application and copy keys into `.env` / `apps/web/.env.local`.
2. Set API verification:
   - `CLERK_JWKS_URL`
   - `CLERK_ISSUER`
   - `JWT_AUDIENCE` (optional; validation can skip audience when empty)
3. With keys present, `/dashboard/*` requires sign-in and the web app sends the Clerk session JWT to the API.
4. Without keys, local development uses `Authorization: Bearer dev`.

## Stripe

1. Create Products/Prices for Pro and Team; put IDs in `NEXT_PUBLIC_STRIPE_PRICE_*`.
2. Set `STRIPE_SECRET_KEY` and `STRIPE_WEBHOOK_SECRET` for the API.
3. Point a Stripe webhook to `POST /api/v1/billing/webhook` for:
   - `customer.subscription.created`
   - `customer.subscription.updated`
   - `customer.subscription.deleted`
   - `checkout.session.completed`
4. Dashboard **Billing** creates Checkout sessions and opens the Customer Portal.
