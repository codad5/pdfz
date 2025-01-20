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

export interface UploadResponse {
    id: string;
    filename: string;
    path: string;
    size: number;
}

export interface ProcessResponse {
    id: string;
    file: string;
    message: string;
    options: {
        startPage: number;
        pageCount: number;
        priority: 0 | 1 | 2;
    };
    status?: 'queued' | 'processing' | 'completed' | 'failed';
    queuedAt?: Date;
}

// You can use these types like this:
// export type UploadPDFResponse = CustomResponse<UploadResponse>;
// export type ProcessPDFResponse = CustomResponse<ProcessResponse>;