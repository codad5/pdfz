// src/lib/redis/FileProcessingService.ts
import { BaseRedisService } from './BaseRedisService';

export enum FileStatus {
    PENDING = "pending",
    DONE = "done",
    FAILED = "failed"
}

export class FileProcessingService extends BaseRedisService {
    constructor() {
        super('processing');
    }

    async isFileInProcessing(fileId: string): Promise<boolean> {
        const status = await this.getFileStatus(fileId);
        return status === FileStatus.PENDING;
    }

    async isProcessDone(fileId: string): Promise<boolean> {
        const status = await this.getFileStatus(fileId);
        return status === FileStatus.DONE;
    }

    async getFileStatus(fileId: string): Promise<FileStatus | null> {
        const status = await this.getStatus(fileId);
        
        switch (status) {
            case "done":
                return FileStatus.DONE;
            case "failed":
                return FileStatus.FAILED;
            case 'pending':
                return FileStatus.PENDING;
            default:
                return null;
        }
    }

    async startFileProcess(fileId: string, ttl: number = 3600): Promise<void> {
        await this.setWithTTL(
            `${this.prefix}:${fileId}`,
            FileStatus.PENDING,
            ttl
        );
    }

    async markAsDone(fileId: string): Promise<void> {
        await this.setStatus(fileId, FileStatus.DONE);
    }

    async markAsFailed(fileId: string): Promise<void> {
        await this.setStatus(fileId, FileStatus.FAILED);
    }

    async markProgress(fileId: string, page: number, total: number): Promise<void> {
        const progress = Math.floor((page / total) * 100);
        await this.setProgress(fileId, progress);

        if (progress === 100) {
            await this.markAsDone(fileId);
        }
    }

    async getFileProgress(fileId: string): Promise<number> {
        return await this.getProgress(fileId);
    }
}