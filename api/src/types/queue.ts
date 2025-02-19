    export type NewFileProcessQueue = {
        file: string; // relative part to shared_storage
        start_page: number; // page number to start processing
        page_count: number; // number of pages to process use 0 for all
        piority?: 0 | 1 | 2; // 0 - low, 1 - medium, 2 - high
        format: 'text' | 'json'; // output format
        engine: 'tesseract'| 'ollama'; // processing engine
        model?: string;
    }

    export type ProcessedFilePage = {
        page_num: number,
        text: String, 
    }

    export type ProcessedFile = ProcessedFilePage[];



    export type Status = 'queued' | 'processing' | 'completed' | 'failed'



    export type OllamaModelPull = {
        name: String;
    }