import { Routes, Route, Navigate } from 'react-router-dom';

import { useAuth } from '@/contexts/AuthContext';
import { Layout } from '@/components/common/Layout';
import { ProtectedRoute } from '@/components/common/ProtectedRoute';

// Customer pages
import CatalogPage from '@/pages/customer/CatalogPage';
import BookDetailPage from '@/pages/customer/BookDetailPage';
import CartPage from '@/pages/customer/CartPage';
import CheckoutPage from '@/pages/customer/CheckoutPage';
import OrdersPage from '@/pages/customer/OrdersPage';
import OrderDetailPage from '@/pages/customer/OrderDetailPage';
import ProfilePage from '@/pages/customer/ProfilePage';

// Admin pages
import AdminCatalogPage from '@/pages/admin/AdminCatalogPage';
import AdminCategoriesPage from '@/pages/admin/AdminCategoriesPage';
import AdminOrdersPage from '@/pages/admin/AdminOrdersPage';

// Shared
import LoginPage from '@/pages/LoginPage';

export default function App() {
  const { isLoading } = useAuth();

  if (isLoading) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100vh' }}>
        Loading…
      </div>
    );
  }

  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />

      <Route element={<Layout />}>
        {/* Customer routes */}
        <Route element={<ProtectedRoute />}>
          <Route path="/" element={<CatalogPage />} />
          <Route path="/books/:bookId" element={<BookDetailPage />} />
          <Route path="/cart" element={<CartPage />} />
          <Route path="/checkout" element={<CheckoutPage />} />
          <Route path="/orders" element={<OrdersPage />} />
          <Route path="/orders/:orderId" element={<OrderDetailPage />} />
          <Route path="/profile" element={<ProfilePage />} />
        </Route>

        {/* Admin routes */}
        <Route element={<ProtectedRoute requiredRole="tenant_admin" />}>
          <Route path="/admin/catalog" element={<AdminCatalogPage />} />
          <Route path="/admin/categories" element={<AdminCategoriesPage />} />
          <Route path="/admin/orders" element={<AdminOrdersPage />} />
        </Route>
      </Route>

      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
