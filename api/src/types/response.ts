export interface SuccessResponse<T> {
    success: true;
    message: string;
    data: T;
}

export interface ErrorResponse<RequestData> {
    success: false;
    message: string;
    data?: RequestData;
}

export type CustomResponse<T> = SuccessResponse<T> | ErrorResponse<T>;

export type UploadPDFResponse = CustomResponse<Express.Multer.File>;
