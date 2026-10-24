import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useParams, useNavigate } from 'react-router-dom';
import { catalogApi, cartApi } from '@/services/api';

export default function BookDetailPage() {
  const { bookId } = useParams<{ bookId: string }>();
  const navigate = useNavigate();
  const qc = useQueryClient();

  const { data: book, isLoading, isError } = useQuery({
    queryKey: ['book', bookId],
    queryFn: () => catalogApi.getBook(bookId!),
    enabled: !!bookId,
  });

  const addToCart = useMutation({
    mutationFn: () => cartApi.upsertItem({ bookId: bookId!, quantity: 1 }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['cart'] });
      navigate('/cart');
    },
  });

  if (isLoading) return <p>Loading...</p>;
  if (isError || !book) return <p>Book not found.</p>;

  return (
    <div style={{ padding: '1rem', maxWidth: 600 }}>
      <button onClick={() => navigate(-1)}>&larr; Back</button>
      {book.coverImageUrl && (
        <img src={book.coverImageUrl} alt={book.title} style={{ width: '100%', maxHeight: 400, objectFit: 'contain', marginTop: '1rem' }} />
      )}
      <h1>{book.title}</h1>
      <p style={{ color: '#666' }}>by {book.author}</p>
      {book.isbn && <p>ISBN: {book.isbn}</p>}
      {book.description && <p>{book.description}</p>}
      <p style={{ fontSize: '1.5rem', fontWeight: 'bold' }}>${(book.price / 100).toFixed(2)}</p>
      <p style={{ color: book.stockQty > 0 ? 'green' : 'red' }}>
        {book.stockQty > 0 ? `${book.stockQty} in stock` : 'Out of stock'}
      </p>
      <button
        disabled={book.stockQty === 0 || addToCart.isPending}
        onClick={() => addToCart.mutate()}
        style={{ padding: '0.75rem 2rem', fontSize: '1rem' }}
      >
        {addToCart.isPending ? 'Adding...' : 'Add to Cart'}
      </button>
    </div>
  );
}
