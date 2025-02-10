// src/lib/redis/ModelDownloadService.ts
import { BaseRedisService } from './BaseRedisService';

export enum ModelStatus {
    QUEUED = "queued",
    DOWNLOADING = "downloading",
    COMPLETED = "completed",
    FAILED = "failed"
}

export class ModelDownloadService extends BaseRedisService {
    constructor() {
        super('model');
    }

    async isModelDownloading(modelName: string): Promise<boolean> {
        const status = await this.getModelStatus(modelName);
        return status === ModelStatus.DOWNLOADING;
    }

    async isModelDownloadComplete(modelName: string): Promise<boolean> {
        const status = await this.getModelStatus(modelName);
        return status === ModelStatus.COMPLETED;
    }

    async getModelStatus(modelName: string): Promise<ModelStatus | null> {
        const status = await this.getStatus(modelName);
        
        switch (status) {
            case "queued":
                return ModelStatus.QUEUED;
            case "downloading":
                return ModelStatus.DOWNLOADING;
            case "completed":
                return ModelStatus.COMPLETED;
            case "failed":
                return ModelStatus.FAILED;
            default:
                return null;
        }
    }

    async startModelDownload(modelName: string, ttl: number = 7200): Promise<void> {
        await this.setWithTTL(
            `${this.prefix}:status:${modelName}`,
            ModelStatus.QUEUED,
            ttl
        );
        await this.setProgress(modelName, 0);
    }

    async markAsDownloading(modelName: string): Promise<void> {
        await this.setStatus(modelName, ModelStatus.DOWNLOADING);
    }

    async markAsCompleted(modelName: string): Promise<void> {
        await this.setStatus(modelName, ModelStatus.COMPLETED);
        await this.setProgress(modelName, 100);
    }

    async markAsFailed(modelName: string): Promise<void> {
        await this.setStatus(modelName, ModelStatus.FAILED);
    }

    async updateProgress(modelName: string, downloadedBytes: number, totalBytes: number): Promise<void> {
        const progress = Math.floor((downloadedBytes / totalBytes) * 100);
        await this.setProgress(modelName, progress);
        
        if (progress === 100) {
            await this.markAsCompleted(modelName);
        }
    }

    async getModelProgress(modelName: string): Promise<number> {
        return await this.getProgress(modelName);
    }

    async getDownloadingModels(): Promise<string[]> {
        const keys = await this.redis.keys(`${this.prefix}:status:*`);
        const downloadingModels: string[] = [];
        
        for (const key of keys) {
            const modelName = key.replace(`${this.prefix}:status:`, '');
            const status = await this.getModelStatus(modelName);
            if (status === ModelStatus.DOWNLOADING) {
                downloadingModels.push(modelName);
            }
        }
        
        return downloadingModels;
    }
}