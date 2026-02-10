import React from 'react';
import { Box, FileCode, Zap, Flame, Hammer, Network, Share2, Globe, Database, Settings, ShieldCheck } from 'lucide-react';
import { ServerType } from './types';

// Local icon imports
import vanillaIcon from '../assets/software/vanilla.png';
import bedrockIcon from '../assets/software/bedrock.png';
import paperIcon from '../assets/software/paper.webp';
import purpurIcon from '../assets/software/purpur.svg';
import forgeIcon from '../assets/software/forge.png';
import neoforgeIcon from '../assets/software/neoforge.png';
import fabricIcon from '../assets/software/fabric.png';
import quiltIcon from '../assets/software/quilt.svg';
import velocityIcon from '../assets/software/velocity.webp';

export const SERVER_TYPES: ServerType[] = [
  {
    id: 'vanilla',
    name: 'Vanilla',
    description: 'The official Minecraft server software from Mojang. Simple, clean, and always up-to-date.',
    category: 'Official',
    icon: <Box className="text-emerald-400" size={24} />,
    imageUrl: vanillaIcon,
  },
  {
    id: 'paper',
    name: 'Paper',
    description: 'The most popular high-performance Spigot fork. Extensive plugin support and optimizations.',
    category: 'Plugins',
    icon: <Zap className="text-emerald-400" size={24} />,
    imageUrl: paperIcon,
  },
  {
    id: 'purpur',
    name: 'Purpur',
    description: 'A drop-in replacement for Paper designed for configurability and new gameplay features.',
    category: 'Plugins',
    icon: <Flame className="text-emerald-400" size={24} />,
    imageUrl: purpurIcon,
  },
  {
    id: 'forge',
    name: 'Forge',
    description: 'The classic modding platform. Required for many popular large-scale modpacks.',
    category: 'Mods',
    icon: <Hammer className="text-emerald-400" size={24} />,
    imageUrl: forgeIcon,
  },
  {
    id: 'neoforge',
    name: 'NeoForge',
    description: 'A modern, community-driven fork of Forge. Better performance and cleaner modding API.',
    category: 'Mods',
    icon: <Zap className="text-emerald-400" size={24} />,
    imageUrl: neoforgeIcon,
  },
  {
    id: 'fabric',
    name: 'Fabric',
    description: 'A lightweight, modular modding toolchain. Known for fast updates and great performance.',
    category: 'Mods',
    icon: <FileCode className="text-emerald-400" size={24} />,
    imageUrl: fabricIcon,
  },
  {
    id: 'quilt',
    name: 'Quilt',
    description: 'A community-first modloader compatible with most Fabric mods. Focuses on modularity.',
    category: 'Mods',
    icon: <Globe className="text-emerald-400" size={24} />,
    imageUrl: quiltIcon,
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
    description: 'The next-generation high-performance proxy. Unmatched scalability and security.',
    category: 'Network Proxy',
    icon: <Zap className="text-emerald-400" size={24} />,
    imageUrl: velocityIcon,
  },
  {
    id: 'bedrock',
    name: 'Bedrock',
    description: 'Official Bedrock Edition server. Play with friends on mobile, console, and Windows 10.',
    category: 'Official',
    icon: <Box className="text-emerald-400" size={24} />,
    imageUrl: bedrockIcon,
  },
];
