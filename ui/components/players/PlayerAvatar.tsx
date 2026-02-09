import { User } from 'lucide-react';
import { usePlayerHead } from '../../hooks/useAssetCache';
import { AppSettings } from '../../hooks/useAppSettings';
import { useInView } from '../../hooks/useInView';

interface PlayerAvatarProps {
  name: string;
  uuid?: string;
  settings: AppSettings;
  className?: string;
}

export function PlayerAvatar({ name, uuid, settings, className = "w-12 h-12" }: PlayerAvatarProps) {
  const [ref, isInView] = useInView({ rootMargin: '100px' });
  // Determine if we should download heads
  const shouldDownload = settings.download_player_heads;

  // Decide which identifier to use for the head
  // If query_heads_by_uuid is true, prefer UUID if available. Otherwise use name.
  const headIdentifier = settings.query_heads_by_uuid ? (uuid || name) : name;

  // Only call hook if we should download and in view
  const { localUrl } = usePlayerHead(shouldDownload ? headIdentifier : null, isInView);

  if (shouldDownload && localUrl) {
    return (
      <img
        ref={ref as any}
        src={localUrl}
        alt={name}
        className={`${className} rounded-xl shadow-lg ring-1 ring-black/10 dark:ring-white/10`}
      />
    );
  }

  return (
    <div
      ref={ref as any}
      className={`${className} rounded-xl bg-black/5 dark:bg-white/5 flex items-center justify-center text-gray-400 border border-black/5 dark:border-white/5`}
    >
      <User size={Math.floor(parseInt(className.split('h-')[1] || '12') / 2)} />
    </div>
  );
}
