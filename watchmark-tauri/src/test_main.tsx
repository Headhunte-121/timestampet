import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './TestApp'
import './index.css'

// Mock Tauri internals for testing in browser without Tauri backend
(window as any).__TAURI_INTERNALS__ = {
  invoke: async (cmd: string, args: any) => {
    console.log("Mock invoke:", cmd, args);
    if (cmd === 'get_library_data') {
      return [
        {
          id: 1,
          title: 'Mock Movie With Poster',
          poster_path: '/mock_poster.jpg',
          release_date: '2023-01-01',
          is_date_known: true,
          is_exact_date: true,
          user_rating: 8,
          total_episodes: 0,
          completed_eps: 0,
          synopsis: 'A mock movie for testing posters.',
        },
        {
          id: 2,
          title: 'Obscure French Film Without Poster',
          poster_path: null,
          release_date: '1955-06-15',
          is_date_known: true,
          is_exact_date: true,
          user_rating: 9,
          total_episodes: 0,
          completed_eps: 0,
          synopsis: 'A classic film with missing poster art to test the fallback.',
        }
      ];
    }
    return [];
  }
};

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
