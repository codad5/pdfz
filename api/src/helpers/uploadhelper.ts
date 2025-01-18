import multer from "multer";
import path from "path";
import fs from "fs";

const pdfUploadPath = `${process.env.SHARED_STORAGE_PATH}/upload/pdf`;
if (!fs.existsSync(pdfUploadPath)) fs.mkdirSync(pdfUploadPath, { recursive: true });

const allowedExtensions = ['.pdf']; // Define allowable extensions
const allowedMimes = ['application/pdf']; // Allowed MIME types

const storage = multer.diskStorage({
  destination: (req, file, cb) => {
    cb(null, pdfUploadPath); // Directory where files will be saved
  },
  filename: (req, file, cb) => {
    cb(null, Date.now() + path.extname(file.originalname)); // Rename file with timestamp
  },
  
});

export const upload = multer({
  storage: storage,
  fileFilter: (req, file, cb) => {
    const fileExtension = path.extname(file.originalname).toLowerCase(); // Get the file extension
    const fileMime = file.mimetype; // Get the MIME type
    console.log(fileExtension, fileMime);

    // Validate extension and MIME type
    if (!allowedExtensions.includes(fileExtension) || !allowedMimes.includes(fileMime)) {
      return cb(
        new Error(
          `Invalid file type. Only ${allowedExtensions.join(', ')} files with MIME types ${allowedMimes.join(', ')} are allowed.`
        )
      );
    }

    cb(null, true); // Accept the file
  },
});
