export type NewFileProcessQueue = {
    file: string; // relative part to shared_storage
    start_page: number; // page number to start processing
    page_count: number; // number of pages to process use 0 for all
    piority?: 0 | 1 | 2; // 0 - low, 1 - medium, 2 - high
}