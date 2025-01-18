import { CustomResponse, ErrorResponse, SuccessResponse } from "@/types/response";

class ResponseHelper {

    static getResponse<T>(success: boolean, message: string, data: T): CustomResponse<T> {
        if (success) {
            return ResponseHelper.getSuccessResponse(data, message);
        }
        return ResponseHelper.getErrorResponse(message, data);
    }

    static getSuccessResponse<T>(data : T, message = 'Success') : SuccessResponse<T> {
        return {
            success: true,
            message,
            data,
        };
    }

    static getErrorResponse<T>(message: string, data?: T) : ErrorResponse<T> {
        return {
            success: false,
            message,
            data,
        };
    }

    static success(res, data, message) {
        return res.status(200).json({
        success: true,
        message,
        data,
        });
    }

    static error(res, message, status = 400) {
        return res.status(status).json({
        success: false,
        message,
        });
    }
}