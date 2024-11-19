import {useCallback, useEffect, useState} from "react";
import { useDropzone, FileWithPath } from "react-dropzone";
import { UploadIcon } from "lucide-react";
import { Button } from "@/components/ui/button.tsx";

type FileUploaderProps = {
    fieldChange: (files: File[]) => void;
    existingFiles?: { url: string; fileName: string }[];

};

const FileUploader = ({ fieldChange, existingFiles = [] }: FileUploaderProps) => {
    const [, setFiles] = useState<File[]>([]);
    const [fileUrls, setFileUrls] = useState<string[]>([]);

    const onDrop = useCallback((acceptedFiles: FileWithPath[]) => {
        setFiles(acceptedFiles);
        fieldChange(acceptedFiles);

        const urls = acceptedFiles.map((file) => URL.createObjectURL(file));
        setFileUrls(urls);
    }, [fieldChange]);

    useEffect(() => {
        if (existingFiles.length > 0) {
            const urls = existingFiles.map((file) => file.url);
            setFileUrls(urls);
        }
    }, [existingFiles]);

    const { getRootProps, getInputProps } = useDropzone({
        onDrop,
        accept: {
            "image/*": [".png", ".jpeg", ".jpg", ".svg", ".gif"],
        },
        multiple: true,
    });

    return (
        <div {...getRootProps()} className="flex flex-center flex-col bg-dark-3 rounded-xl cursor-pointer">
            <input {...getInputProps()} className="cursor-pointer" />

            {fileUrls.length > 0 ? (
                <div className="w-full p-4">
                    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
                        {fileUrls.map((url, index) => (
                            <div key={index} className="relative">
                                <img
                                    src={url}
                                    alt={`Preview ${index}`}
                                    className="w-full h-full object-cover rounded-md"
                                />
                            </div>
                        ))}
                    </div>
                </div>
            ) : (
                <div className="flex flex-col items-center justify-center p-7 h-70 lg:h-[400px]">
                    <UploadIcon width={96} height={77} />
                    <h3 className="base-medium text-light-2 mb-2 mt-6">Drag photos here</h3>
                    <h3 className="text-light-4 small-regular mb-6">All image formats, except WebP</h3>
                    <Button className="shad-button_dark_4">Select from computer</Button>
                </div>
            )}
        </div>
    );
};

export default FileUploader;
