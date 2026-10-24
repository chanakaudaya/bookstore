// ---- Catalog ----

export interface Category {
  id: string;
  tenantId: string;
  name: string;
  slug: string;
  description?: string;
  createdAt: string;
}

export interface Book {
  id: string;
  tenantId: string;
  categoryId?: string;
  title: string;
  author: string;
  isbn?: string;
  description?: string;
  price: number; // in cents
  stockQty: number;
  coverImageUrl?: string;
  publishedAt?: string;
  createdAt: string;
  updatedAt: string;
}

export interface PagedResponse<T> {
  data: T[];
  total: number;
  page: number;
  perPage: number;
}

// ---- Cart ----

export interface CartItem {
  id: string;
  cartId: string;
  bookId: string;
  title: string;
  unitPrice: number;
  quantity: number;
  createdAt: string;
}

export interface Cart {
  id: string;
  userId: string;
  tenantId: string;
  items: CartItem[];
  subtotal: number;
  createdAt: string;
  updatedAt: string;
}

// ---- User ----

export interface UserProfile {
  id: string;
  tenantId: string;
  keycloakUserId: string;
  email: string;
  firstName?: string;
  lastName?: string;
  phone?: string;
  createdAt: string;
  updatedAt: string;
}

export interface ShippingAddress {
  id: string;
  userId: string;
  tenantId: string;
  label?: string;
  line1: string;
  line2?: string;
  city: string;
  state?: string;
  postalCode: string;
  country: string;
  isDefault: boolean;
  createdAt: string;
}

export interface PaymentMethod {
  id: string;
  userId: string;
  tenantId: string;
  label: string;
  methodType: string;
  lastFour?: string;
  tokenReference?: string;
  isDefault: boolean;
  createdAt: string;
}

// ---- Orders ----

export interface OrderItem {
  id: string;
  orderId: string;
  bookId: string;
  title: string;
  unitPrice: number;
  quantity: number;
}

export interface Order {
  id: string;
  tenantId: string;
  userId: string;
  status: 'confirmed' | 'processing' | 'shipped' | 'delivered' | 'cancelled' | 'payment_failed';
  paymentStatus: 'approved' | 'declined';
  total: number;
  shippingAddressId: string;
  paymentMethodId: string;
  items?: OrderItem[];
  createdAt: string;
  updatedAt: string;
}

// ---- Auth ----

export interface AuthUser {
  sub: string;
  email?: string;
  preferredUsername?: string;
  tenantId?: string;
  roles: string[];
}
