import { toast as sonnerToast } from "sonner";
import { logger } from "./logger";

export const toast = {
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