import { Monaco } from '@monaco-editor/react';

export const registerCustomLanguages = (monaco: Monaco) => {
  registerSkriptLanguage(monaco);
  registerTomlLanguage(monaco);
};

export const registerSkriptLanguage = (monaco: Monaco) => {
  // Register Skript language
  monaco.languages.register({ id: 'skript' });

  // Define tokens for Skript
  monaco.languages.setMonarchTokensProvider('skript', {
    defaultToken: '',
    tokenPostfix: '.sk',

    keywords: [
      'on', 'command', 'trigger', 'function', 'if', 'else', 'loop', 'every', 
      'while', 'stop', 'exit', 'return', 'cancel', 'event', 'message', 
      'broadcast', 'send', 'set', 'add', 'remove', 'give', 'teleport', 
      'kill', 'spawn', 'execute', 'wait', 'chance', 'random', 'player', 
      'victim', 'attacker', 'target', 'location', 'world', 'metadata', 
      'variable', 'list', 'index', 'is', 'isn\'t', 'not', 'and', 'or', 
      'has', 'contains', 'greater', 'less', 'than', 'equal', 'to'
    ],

    operators: [
      '=', '<', '>', '!', ':', '.'
    ],

    // Symbols
    symbols: /[=><!~?:&|+\-*\/\^%]+/,

    tokenizer: {
      root: [
        // Keywords and identifiers
        [/[a-zA-Z_]\w*/, {
          cases: {
            '@keywords': 'keyword',
            '@default': 'identifier'
          }
        }],

        // Comments
        [/#.*$/, 'comment'],

        // Strings
        [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],

        // Variables
        [/\{[^\s\}]+\}/, 'variable'],

        // Numbers
        [/\d+(\.\d+)?/, 'number'],

        // Whitespace
        { include: '@whitespace' },

        // Delimiters
        [/[{}()\[\]]/, '@brackets'],
        [/[<>](?!@symbols)/, '@brackets'],
        [/@symbols/, {
          cases: {
            '@operators': 'operator',
            '@default': ''
          }
        }],
      ],

      whitespace: [
        [/[ \t\r\n]+/, 'white'],
      ],

      string: [
        [/[^\\"]+/, 'string'],
        [/\\./, 'string.escape'],
        [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }],
      ],
    },
  });

  // Define configuration for Skript
  monaco.languages.setLanguageConfiguration('skript', {
    comments: {
      lineComment: '#',
    },
    brackets: [
      ['{', '}'],
      ['[', ']'],
      ['(', ')'],
    ],
    autoClosingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
    ],
    surroundingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
    ],
  });
};

export const registerTomlLanguage = (monaco: Monaco) => {
  // Register TOML language if it's not already registered
  // Monaco usually has TOML built-in, but just in case or for custom themes
  monaco.languages.register({ id: 'toml' });

  monaco.languages.setMonarchTokensProvider('toml', {
    defaultToken: '',
    tokenPostfix: '.toml',

    tokenizer: {
      root: [
        // Sections
        [/^\[.*\]/, 'keyword'],
        
        // Keys
        [/[a-zA-Z0-9_-]+(?=\s*=)/, 'variable'],
        
        // Strings
        [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],
        [/'/, { token: 'string.quote', bracket: '@open', next: '@string_single' }],
        
        // Numbers
        [/\d+(\.\d+)?/, 'number'],
        
        // Booleans
        [/\b(true|false)\b/, 'keyword'],
        
        // Comments
        [/#.*$/, 'comment'],
        
        // Whitespace
        { include: '@whitespace' },
      ],

      whitespace: [
        [/[ \t\r\n]+/, 'white'],
      ],

      string: [
        [/[^\\"]+/, 'string'],
        [/\\./, 'string.escape'],
        [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }],
      ],

      string_single: [
        [/[^\\']+/, 'string'],
        [/\\./, 'string.escape'],
        [/'/, { token: 'string.quote', bracket: '@close', next: '@pop' }],
      ],
    },
  });

  monaco.languages.setLanguageConfiguration('toml', {
    comments: {
      lineComment: '#',
    },
    brackets: [
      ['{', '}'],
      ['[', ']'],
      ['(', ')'],
    ],
    autoClosingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
    ],
  });
};

export const getLanguageFromExtension = (filename: string | null): string => {
  if (!filename) return 'yaml';
  const ext = filename.split('.').pop()?.toLowerCase();
  switch (ext) {
    case 'json': return 'json';
    case 'yml':
    case 'yaml': return 'yaml';
    case 'sk': return 'skript';
    case 'toml': return 'toml';
    case 'lua': return 'lua';
    case 'md': return 'markdown';
    case 'properties': return 'ini'; // Monaco uses ini for properties usually
    case 'sh': return 'shell';
    case 'js': return 'javascript';
    case 'xml': return 'xml';
    case 'html': return 'html';
    case 'css': return 'css';
    case 'txt':
    case 'log': return 'plaintext';
    default: return 'yaml';
  }
};
