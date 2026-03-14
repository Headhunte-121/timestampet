import { useRef, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { invokeWithTimeout } from '../utils/ipc';

export function useAsyncInvoke() {
  const isMounted = useRef(true);

  useEffect(() => {
    isMounted.current = true;
    return () => {
      isMounted.current = false;
    };
  }, []);

  const asyncInvoke = useCallback(async <T>(cmd: string, args: Record<string, unknown> = {}, timeoutMs = 15000): Promise<T | null> => {
    // Generate UUID for cancellation tracking specifically for heavy reads
    const request_id = crypto.randomUUID();

    // The backend signature changed to expect request_id,
    // we only pass it to the ones we explicitly modified
    const isCancelable = ['get_library_data', 'fetch_history', 'get_dashboard_data', 'perform_tmdb_search'].includes(cmd);

    let enhancedArgs = { ...args };
    if (isCancelable) {
      enhancedArgs = { ...args, requestId: request_id };
    }

    try {
      const result = await invokeWithTimeout<T>(cmd, enhancedArgs, timeoutMs);
      if (isMounted.current) {
        return result;
      } else {
        // Unmounted before promise resolved
        if (isCancelable) {
            invoke('cancel_task', { requestId: request_id }).catch(console.error);
        }
        return null; // Silently discard
      }
    } catch (e: any) {
      // If unmounted, we don't care about the error bubbling up to state either
      if (!isMounted.current) {
        if (isCancelable) {
          invoke('cancel_task', { requestId: request_id }).catch(console.error);
        }
        return null;
      }
      throw e;
    }
  }, []);

  return asyncInvoke;
}
