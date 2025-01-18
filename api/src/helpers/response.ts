import { Response, Request } from 'express';
import { CustomResponse, ErrorResponse, SuccessResponse } from '@/types/response';

export class ResponseHelper {
    private static responseObj?: Response;

    /**
     * Registers an Express response object for later use
     */
    static registerExpressResponse(req: Request, res: Response): void {
        ResponseHelper.responseObj = res;
    }

    /**
     * Ensures response object is available
     */
    private static ensureResponse(): Response {
        if (!ResponseHelper.responseObj) {
            throw new Error('Response object not registered. Call registerExpressResponse first or provide a response object.');
        }
        return ResponseHelper.responseObj;
    }

    /**
     * Creates a response object based on success status
     */
    static getResponse<T>(success: boolean, message: string, data: T): CustomResponse<T> {
        if (success) {
            return ResponseHelper.getSuccessResponse(data, message);
        }
        return ResponseHelper.getErrorResponse(message, data);
    }

    /**
     * Creates a success response object
     */
    static getSuccessResponse<T>(data: T, message = 'Success'): SuccessResponse<T> {
        return {
            success: true,
            message,
            data,
        };
    }

    /**
     * Creates an error response object
     */
    static getErrorResponse<T>(message: string, data?: T): ErrorResponse<T> {
        return {
            success: false,
            message,
            data,
        };
    }

    /**
     * Sends a response with appropriate status code using provided response object
     */
    static sendResponse<T>(res: Response, response: CustomResponse<T>): Response {
        if (response.success) {
            return res.status(200).json(response);
        }
        return res.status(400).json(response);
    }

    /**
     * Sends a success response using provided response object
     */
    static sendSuccess<T>(res: Response, data: T, message = 'Success'): Response {
        return ResponseHelper.sendResponse(res, ResponseHelper.getSuccessResponse(data, message));
    }

    /**
     * Sends an error response using provided response object
     */
    static sendError<T>(res: Response, message: string, data?: T): Response {
        return ResponseHelper.sendResponse(res, ResponseHelper.getErrorResponse(message, data));
    }

    /**
     * Sends a response using stored response object
     */
    static send<T>(response: CustomResponse<T>): Response {
        const res = ResponseHelper.ensureResponse();
        return ResponseHelper.sendResponse(res, response);
    }

    /**
     * Sends a success response using stored response object
     */
    static success<T>(data: T, message = 'Success'): Response {
        const res = ResponseHelper.ensureResponse();
        return ResponseHelper.sendSuccess(res, data, message);
    }

    /**
     * Sends an error response using stored response object
     */
    static error<T>(message: string, data?: T): Response {
        const res = ResponseHelper.ensureResponse();
        return ResponseHelper.sendError(res, message, data);
    }
}