import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function getFilename(path: string): string {
  return path.split(/[\\/]/).pop() || '';
}

export function getParentDir(path: string): string {
  const parts = path.split(/[\\/]/);
  parts.pop();
  return parts.join('/');
}

export function formatError(err: any): string {
  if (typeof err === 'string') return err;
  if (err && typeof err === 'object' && 'message' in err) {
    return err.message as string;
  }
  return String(err);
}
