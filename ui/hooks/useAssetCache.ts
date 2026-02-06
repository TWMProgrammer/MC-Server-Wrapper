import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';

/**
 * A hook that caches a remote image URL locally and returns a local URL.
 * Useful for icons, screenshots, and player heads.
 */
export function useAssetCache(url: string | null | undefined) {
  const [localUrl, setLocalUrl] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!url) {
      setLocalUrl(null);
      return;
    }

    // If it's already a local path or data URL, don't cache
    if (url.startsWith('data:') || url.startsWith('asset:') || url.startsWith('file:')) {
      setLocalUrl(url);
      return;
    }

    let isMounted = true;
    setIsLoading(true);

    const cacheImage = async () => {
      try {
        const path = await invoke<string>('cache_asset', { url });
        if (isMounted) {
          setLocalUrl(convertFileSrc(path));
          setIsLoading(false);
        }
      } catch (err) {
        console.error('Failed to cache asset:', err);
        if (isMounted) {
          setError(err instanceof Error ? err.message : String(err));
          // Fallback to original URL on error
          setLocalUrl(url);
          setIsLoading(false);
        }
      }
    };

    cacheImage();

    return () => {
      isMounted = false;
    };
  }, [url]);

  return { localUrl, isLoading, error };
}

/**
 * A hook that caches multiple remote image URLs locally and returns an array of local URLs.
 */
export function useAssetsCache(urls: string[] | null | undefined) {
  const [localUrls, setLocalUrls] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (!urls || urls.length === 0) {
      setLocalUrls([]);
      return;
    }

    let isMounted = true;
    setIsLoading(true);

    const cacheImages = async () => {
      try {
        const results = await Promise.all(
          urls.map(async (url) => {
            try {
              if (url.startsWith('data:') || url.startsWith('asset:') || url.startsWith('file:')) {
                return url;
              }
              const path = await invoke<string>('cache_asset', { url });
              return convertFileSrc(path);
            } catch (err) {
              console.error(`Failed to cache asset ${url}:`, err);
              return url; // Fallback to original URL
            }
          })
        );

        if (isMounted) {
          setLocalUrls(results);
          setIsLoading(false);
        }
      } catch (err) {
        console.error('Failed to cache assets:', err);
        if (isMounted) {
          setLocalUrls(urls);
          setIsLoading(false);
        }
      }
    };

    cacheImages();

    return () => {
      isMounted = false;
    };
  }, [urls]);

  return { localUrls, isLoading };
}

/**
 * A hook specifically for player heads.
 */
export function usePlayerHead(uuid: string | null | undefined) {
  const [localUrl, setLocalUrl] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (!uuid) {
      setLocalUrl(null);
      return;
    }

    let isMounted = true;
    setIsLoading(true);

    const getHead = async () => {
      try {
        const path = await invoke<string>('get_player_head_path', { uuid });
        if (isMounted) {
          setLocalUrl(convertFileSrc(path));
          setIsLoading(false);
        }
      } catch (err) {
        console.error('Failed to get player head:', err);
        if (isMounted) {
          // Fallback to mc-heads.net directly
          setLocalUrl(`https://mc-heads.net/avatar/${uuid}/64`);
          setIsLoading(false);
        }
      }
    };

    getHead();

    return () => {
      isMounted = false;
    };
  }, [uuid]);

  return { localUrl, isLoading };
}
