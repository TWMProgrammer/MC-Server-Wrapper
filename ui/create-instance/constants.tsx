import { Box, Send, Sparkles, Hammer, Zap, Layers, Network, Gamepad2 } from 'lucide-react'
import { ServerType } from './types'

export const SERVER_TYPES: ServerType[] = [
  {
    id: 'vanilla',
    name: 'Vanilla',
    description: 'The basic Vanilla experience without plugins.',
    category: 'Playable Server',
    icon: <Box className="text-emerald-400" size={24} />,
  },
  {
    id: 'paper',
    name: 'Paper',
    description: 'High performance fork of Spigot with many features and performance improvements.',
    category: 'Playable Server',
    icon: <Send className="text-blue-400" size={24} />,
  },
  {
    id: 'purpur',
    name: 'Purpur',
    description: 'Purpur is a drop-in replacement for Paper servers designed for configurability and new features.',
    category: 'Playable Server',
    icon: <Sparkles className="text-purple-400" size={24} />,
  },
  {
    id: 'forge',
    name: 'Forge',
    description: 'Drastically change the way how Minecraft looks and feels with mods.',
    category: 'Playable Server',
    icon: <Hammer className="text-orange-400" size={24} />,
  },
  {
    id: 'neoforge',
    name: 'NeoForge',
    description: 'A community-driven fork of Forge, designed to be more modern and open.',
    category: 'Playable Server',
    icon: <Zap className="text-amber-400" size={24} />,
  },
  {
    id: 'fabric',
    name: 'Fabric',
    description: 'Fabric is a lightweight, experimental modding toolchain for Minecraft.',
    category: 'Playable Server',
    icon: <Layers className="text-indigo-400" size={24} />,
  },
  {
    id: 'quilt',
    name: 'Quilt',
    description: 'Quilt is an open-source, community-driven modding toolchain, designed primarily for Minecraft.',
    category: 'Playable Server',
    icon: <Box className="text-pink-400" size={24} />,
  },
  {
    id: 'bungeecord',
    name: 'BungeeCord',
    description: 'Efficiently proxies, maintains connections and transport between multiple servers.',
    category: 'Network Proxy',
    icon: <Network className="text-emerald-400" size={24} />,
  },
  {
    id: 'velocity',
    name: 'Velocity',
    description: 'Modern alternative to Waterfall. Designed with performance and stability in mind.',
    category: 'Network Proxy',
    icon: <Zap className="text-blue-400" size={24} />,
  },
  {
    id: 'bedrock',
    name: 'Bedrock',
    description: 'Multi-platform versions of Minecraft for Mobile, Console & Other',
    category: 'Other',
    icon: <Gamepad2 className="text-gray-400" size={24} />,
    badge: 'preview',
    badgeColor: 'text-orange-400',
  },
];
