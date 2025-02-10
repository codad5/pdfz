export interface ProcessOptions {
    startPage?: number;
    pageCount?: number;
    priority?: 0 | 1 | 2;
    engine: 'tesseract' | 'ollama', 
    model ?: string,
}