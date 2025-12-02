# Sikuli-D Documentation

This directory contains the VitePress documentation site for Sikuli-D.

## Setup

Install dependencies:

```bash
npm install
```

## Development

Start the development server:

```bash
npm run dev
```

The site will be available at `http://localhost:5173/Sikuli-D/`

## Building

Build the static site:

```bash
npm run build
```

Output will be in `.vitepress/dist/`

## Preview

Preview the built site:

```bash
npm run preview
```

## Structure

```
docs/
├── .vitepress/
│   └── config.ts          # VitePress configuration
├── getting-started/       # Getting started guides (English)
├── api/                   # API reference (English)
├── tutorials/             # Tutorials (English)
├── troubleshooting/       # Troubleshooting guides (English)
├── ja/                    # Japanese translations
│   ├── getting-started/
│   ├── api/
│   ├── tutorials/
│   └── troubleshooting/
└── index.md              # Home page (English)
```

## Languages

The documentation is available in:
- English (default)
- Japanese (日本語)

Use the language selector in the top navigation to switch between languages.

## Contributing

When adding new documentation:

1. Add the English version in the appropriate directory
2. Add the Japanese translation in the corresponding `ja/` directory
3. Update the sidebar configuration in `.vitepress/config.ts`

## License

Apache License 2.0
