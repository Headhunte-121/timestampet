/* WATCHMARK STANDARD PATTERN: All asynchronous data commands MUST implement requestId for cancellation support and pagination (page/limit) for UI performance. Follow this signature for all future connections to maintain Phase 1 & 2 integrity. */

import { useRef, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { invokeWithTimeout } from '../utils/ipc';
import { logger } from '../utils/logger';

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
    const isCancelable = ['get_library_data', 'fetch_history', 'get_dashboard_data', 'perform_tmdb_search', 'assign_unmatched_to_tracker'].includes(cmd);

    let enhancedArgs = { ...args };
    if (isCancelable) {
      enhancedArgs = { ...args, requestId: request_id };
    }

    // Pass raw args so logger can parse them specifically
    const rawContext = Object.keys(args).length > 0 ? args : undefined;

    logger.ipcSend(cmd, rawContext, isCancelable ? request_id : undefined);

    try {
      const result = await invokeWithTimeout<T>(cmd, enhancedArgs, timeoutMs);

      if (isMounted.current) {
        if (Array.isArray(result)) {
            logger.ipcSuccess(`Received ${result.length} items from '${cmd}'.`);
        } else if (result && typeof result === 'object') {
            logger.ipcSuccess(`Received data object from '${cmd}'.`);
        } else {
            logger.ipcSuccess(`'${cmd}' completed successfully.`);
        }
        return result;
      } else {
        // Unmounted before promise resolved
        if (isCancelable) {
            logger.ipcCancel(request_id, "User left the page");
            invoke('cancel_task', { requestId: request_id }).catch(err => logger.error(`Failed to cancel background task ${request_id}`, err));
        }
        return null; // Silently discard
      }
    } catch (e: any) {
      // If unmounted, we don't care about the error bubbling up to state either
      if (!isMounted.current) {
        if (isCancelable) {
          logger.ipcCancel(request_id, "User left the page during failure");
          invoke('cancel_task', { requestId: request_id }).catch(err => logger.error(`Failed to cancel background task ${request_id}`, err));
        }
        return null;
      }

      logger.error(`Command '${cmd}' failed`, e);
      throw e;
    }
  }, []);

  return asyncInvoke;
}
