// src/lib/redis/index.ts
import { FileProcessingService } from './FileProcessingService';
import { ModelDownloadService } from './ModelDownloadService';

// Create singleton instances
export const fileProcessingService = new FileProcessingService();
export const modelDownloadService = new ModelDownloadService();

// Re-export types
export { FileStatus } from './FileProcessingService';
export { ModelStatus } from './ModelDownloadService';

// Export all methods from both services
export const {
    isFileInProcessing,
    isProcessDone,
    startFileProcess,
    markAsDone: markFileAsDone,
    markAsFailed: markFileAsFailed,
    markProgress: markFileProgress,
    getFileProgress,
} = fileProcessingService;

export const {
    isModelDownloading,
    isModelDownloadComplete,
    startModelDownload,
    markAsDownloading,
    markAsCompleted,
    markAsFailed: markModelAsFailed,
    updateProgress: updateModelProgress,
    getModelProgress,
    getDownloadingModels,
    getModelStatus
} = modelDownloadService;