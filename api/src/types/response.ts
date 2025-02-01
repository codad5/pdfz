import { ProcessedFile, Status } from "./queue";

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
    progress: number;
    status?: Status;
    queuedAt?: Date;
}

export interface ProgressResponse {
    id: string;
    progress: number;
    status: Status;
    message?: string;
}

export interface FinalResponse {
    id: string;
    content: ProcessedFile;
    message: string;
    status: Status;
}




// You can use these types like this:
// export type UploadPDFResponse = CustomResponse<UploadResponse>;
// export type ProcessPDFResponse = CustomResponse<ProcessResponse>;