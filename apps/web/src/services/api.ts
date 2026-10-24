import axios, { AxiosError } from 'axios';

const apiClient = axios.create({
  baseURL: '/api/v1',
  headers: { 'Content-Type': 'application/json' },
});

/** Set or clear the Bearer token on the shared client. Called by AuthContext on auth state change. */
export function setAuthToken(token: string | undefined) {
  if (token) {
    apiClient.defaults.headers.common['Authorization'] = 'Bearer ' + token;
  } else {
    delete apiClient.defaults.headers.common['Authorization'];
  }
}

/** Response interceptor -- surfaces API error messages from the standard error envelope. */
apiClient.interceptors.response.use(
  (response) => response,
  (error: AxiosError<{ error: string; message: string }>) => {
    const message = error.response?.data?.message ?? error.message;
    return Promise.reject(new Error(message));
  }
);

export default apiClient;

// ---- Catalog ----

import type {
  Book,
  Category,
  Cart,
  UserProfile,
  ShippingAddress,
  PaymentMethod,
  Order,
  PagedResponse,
} from '@/types';

export interface BookListParams {
  q?: string;
  categoryId?: string;
  page?: number;
  perPage?: number;
}

export interface CreateBookPayload {
  title: string;
  author: string;
  isbn?: string;
  description?: string;
  price: number;
  stock_qty: number;
  cover_image_url?: string;
  category_id?: string;
  published_at?: string;
}

export const catalogApi = {
  listBooks: (params?: BookListParams) =>
    apiClient.get<PagedResponse<Book>>('/catalog/books', { params }).then((r) => r.data),

  getBook: (id: string) =>
    apiClient.get<Book>(`/catalog/books/${id}`).then((r) => r.data),

  listCategories: () =>
    apiClient.get<Category[]>('/catalog/categories').then((r) => r.data),

  // Admin write operations
  createBook: (payload: CreateBookPayload) =>
    apiClient.post<Book>('/catalog/books', payload).then((r) => r.data),

  updateBook: (id: string, payload: Partial<CreateBookPayload>) =>
    apiClient.put<Book>(`/catalog/books/${id}`, payload).then((r) => r.data),

  deleteBook: (id: string) =>
    apiClient.delete(`/catalog/books/${id}`).then((r) => r.data),

  createCategory: (payload: { name: string; slug: string; description?: string }) =>
    apiClient.post<Category>('/catalog/categories', payload).then((r) => r.data),

  updateCategory: (id: string, payload: { name?: string; description?: string }) =>
    apiClient.put<Category>(`/catalog/categories/${id}`, payload).then((r) => r.data),

  deleteCategory: (id: string) =>
    apiClient.delete(`/catalog/categories/${id}`).then((r) => r.data),
};

// ---- Cart ----

export const cartApi = {
  getCart: () => apiClient.get<Cart>('/cart').then((r) => r.data),

  upsertItem: (payload: { bookId: string; quantity: number }) =>
    apiClient.post<Cart>('/cart', payload).then((r) => r.data),

  clearCart: () => apiClient.delete('/cart').then((r) => r.data),
};

// ---- Checkout ----

export interface CheckoutPayload {
  idempotencyKey: string;
  shippingAddressId: string;
  paymentMethodId: string;
}

export interface CheckoutResult {
  orderId: string;
  status: string;
  paymentStatus: string;
  total: number;
}

export const checkoutApi = {
  checkout: (payload: CheckoutPayload) =>
    apiClient.post<CheckoutResult>('/checkout', payload).then((r) => r.data),
};

// ---- Users / Account ----

export const usersApi = {
  getProfile: () => apiClient.get<UserProfile>('/users/me').then((r) => r.data),

  updateProfile: (payload: Partial<Pick<UserProfile, 'firstName' | 'lastName' | 'phone'>>) =>
    apiClient.put<UserProfile>('/users/me', payload).then((r) => r.data),

  listAddresses: () =>
    apiClient.get<ShippingAddress[]>('/users/me/addresses').then((r) => r.data),

  createAddress: (payload: Omit<ShippingAddress, 'id' | 'userId' | 'tenantId' | 'createdAt'>) =>
    apiClient.post<ShippingAddress>('/users/me/addresses', payload).then((r) => r.data),

  deleteAddress: (id: string) => apiClient.delete(`/users/me/addresses/${id}`),

  listPaymentMethods: () =>
    apiClient.get<PaymentMethod[]>('/users/me/payment-methods').then((r) => r.data),

  createPaymentMethod: (payload: Omit<PaymentMethod, 'id' | 'userId' | 'tenantId' | 'createdAt'>) =>
    apiClient.post<PaymentMethod>('/users/me/payment-methods', payload).then((r) => r.data),

  deletePaymentMethod: (id: string) => apiClient.delete(`/users/me/payment-methods/${id}`),
};

// ---- Orders ----

export const ordersApi = {
  listMyOrders: (params?: { page?: number; perPage?: number }) =>
    apiClient.get<PagedResponse<Order>>('/orders/me', { params }).then((r) => r.data),

  getMyOrder: (id: string) =>
    apiClient.get<Order>(`/orders/me/${id}`).then((r) => r.data),

  // Admin
  adminListOrders: (params?: { page?: number; perPage?: number }) =>
    apiClient.get<PagedResponse<Order>>('/orders/admin', { params }).then((r) => r.data),

  adminUpdateStatus: (id: string, status: string) =>
    apiClient.put<Order>(`/orders/admin/${id}/status`, { status }).then((r) => r.data),
};
