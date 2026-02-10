# Minecraft Server Software Icons Report

This report provides links to the official or most commonly used icons and logos for various Minecraft server softwares. These can be used to enhance the UI by replacing generic icons with official branding.

| Software | Category | Icon URL | Description |
| :--- | :--- | :--- | :--- |
| **Vanilla** | Playable Server | [Link](https://minecraft.wiki/images/Grass_Block_JE7_BE6.png?2bd37) | High-resolution Minecraft Grass Block icon. |
| **Paper** | Playable Server | [Link](https://docs.papermc.io/_astro/papermc_logo.256_Z28RK0e.webp) | Official PaperMC logo (256px WebP). |
| **Purpur** | Playable Server | [Link](https://purpurmc.org/images/purpur.svg) | Official PurpurMC sparkles logo (SVG). |
| **Forge** | Playable Server | [Link](https://storage.googleapis.com/replit/images/1654232400062_57239fe5995715e769a2e88f9131ee72.png) | Official Minecraft Forge anvil-only logo. |
| **NeoForge** | Playable Server | [Link](https://neoforged.net/img/authors/neoforged.png) | Official NeoForge branding (PNG). |
| **Fabric** | Playable Server | [Link](https://fabricmc.net/assets/logo.png) | Official Fabric mod loader loom logo. |
| **Quilt** | Playable Server | [Link](https://quiltmc.org/assets/img/logo.svg) | Official QuiltMC patch pattern logo (SVG). |
| **BungeeCord** | Network Proxy | [Link](https://raw.githubusercontent.com/PrismLauncher/PrismLauncher/develop/launcher/resources/multimc/64px/instances/bungeecord.png) | Official BungeeCord bridge-themed logo. |
| **Velocity** | Network Proxy | [Link](https://docs.papermc.io/_astro/velocity_logo_blue.min_ZLBWdW.webp) | Official Velocity proxy logo from PaperMC. |
| **Bedrock** | Playable Server | [Link](https://minecraft.wiki/images/Bedrock_JE2_BE2.png?5ea94) | Official Minecraft Bedrock Edition icon. |

## Usage Guidelines

- **PaperMC / Velocity**: Assets are provided via their CDN. Use of official URLs is encouraged by the PaperMC team.
- **Fabric / Quilt**: These projects are community-driven; ensure compliance with their respective brand guidelines if available.
- **Forge / NeoForge**: Standard modding API branding.
- **Vanilla / Bedrock**: Property of Mojang Studios/Microsoft. Use should follow their commercial usage guidelines.

## Implementation Suggestion

For the UI implementation in [constants.tsx](ui/create-instance/constants.tsx), these URLs can be used in an `<img>` tag or as a background image for the server type selection cards to provide a more professional look.
