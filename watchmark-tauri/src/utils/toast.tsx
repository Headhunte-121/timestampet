import { toast as sonnerToast } from "sonner";
import { logger } from "./logger";

import { ScanCompleteToast } from "../components/ui/ScanCompleteToast";

export const toast = {
    scanComplete: (result: any) => {
        logger.toast(`Scan Complete: ${result.auto_matched_count} matched, ${result.unmatched_count} unmatched`, 'info');
        sonnerToast.custom((t) => <ScanCompleteToast result={result} t={t} />, {
            duration: Infinity, // The custom component handles its own timeout/dismiss
            className: "w-[350px] p-0 bg-transparent shadow-none border-none",
        });
    },
    success: (msg: string, data?: any) => {
        logger.toast(msg, 'success');
        sonnerToast.success(msg, data);
    },
    error: (msg: string, data?: any) => {
        logger.toast(msg, 'error');
        sonnerToast.error(msg, data);
    },
    info: (msg: string, data?: any) => {
        logger.toast(msg, 'info');
        sonnerToast.info(msg, data);
    }
};