import { test, expect } from '@playwright/test';

test('history displays cross-day binge label correctly', async ({ page }) => {
  // We need to inject mock tauri IPC before the page loads.
  await page.addInitScript(() => {
    window.__TAURI_INTERNALS__ = {
      invoke: async (cmd, args) => {
        if (cmd === 'fetch_history') {
          // Provide some mock history that simulates a crossed day boundary
          return [
            {
              type: "binge_block",
              episode_count: 5,
              total_runtime: 250,
              entries: [
                { timestamp: 1709400000, media_id: 1, is_legacy: 0, session_id: "test" }, // Latest
                { timestamp: 1709350000, media_id: 1, is_legacy: 0, session_id: "test" }, // Earliest
              ],
              main_entry: {
                show_title: "Test Show",
                media_type: "TV",
                poster_path: "",
                timestamp: 1709400000,
                is_legacy: 0,
                completion_ratio: 1.0,
              }
            }
          ];
        }
        return [];
      }
    };
  });

  await page.goto('http://localhost:1420/#/history');

  // Wait for the history items to render
  await expect(page.getByText('Watch History')).toBeVisible();

  // Give the page a moment for data loading
  await page.waitForTimeout(1000);

  // Screenshot the history page
  await page.screenshot({ path: '/home/jules/verification/history_screenshot.png', fullPage: true });
});
