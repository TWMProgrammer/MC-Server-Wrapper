import { describe, it, expect } from 'vitest';
import { cn, formatSize, getFilename, getParentDir } from '../../ui/utils';

describe('utils', () => {
  describe('cn', () => {
    it('merges tailwind classes correctly', () => {
      expect(cn('px-2 py-2', 'px-4')).toBe('py-2 px-4');
    });

    it('handles conditional classes', () => {
      expect(cn('px-2', true && 'py-2', false && 'm-2')).toBe('px-2 py-2');
    });

    it('handles arrays and objects', () => {
      expect(cn(['px-2', 'py-2'], { 'm-2': true, 'p-4': false })).toBe('px-2 py-2 m-2');
    });
  });

  describe('formatSize', () => {
    it('formats bytes correctly', () => {
      expect(formatSize(0)).toBe('0 Bytes');
      expect(formatSize(1024)).toBe('1 KB');
      expect(formatSize(1024 * 1024)).toBe('1 MB');
      expect(formatSize(1024 * 1024 * 1.5)).toBe('1.5 MB');
      expect(formatSize(1024 * 1024 * 1024)).toBe('1 GB');
    });
  });

  describe('getFilename', () => {
    it('extracts filename from path', () => {
      expect(getFilename('C:\\path\\to\\file.txt')).toBe('file.txt');
      expect(getFilename('/path/to/file.txt')).toBe('file.txt');
      expect(getFilename('file.txt')).toBe('file.txt');
    });

    it('handles trailing slashes', () => {
      expect(getFilename('/path/to/dir/')).toBe('');
    });
  });

  describe('getParentDir', () => {
    it('gets parent directory correctly', () => {
      expect(getParentDir('/path/to/file.txt')).toBe('/path/to');
      expect(getParentDir('C:\\path\\to\\file.txt')).toBe('C:/path/to');
    });

    it('handles paths with no parent', () => {
      expect(getParentDir('file.txt')).toBe('');
    });
  });
});
